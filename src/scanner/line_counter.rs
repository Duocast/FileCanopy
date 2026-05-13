use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Result;
use crate::scanner::metadata::{EntryKind, FileEntry};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LineCountReport {
    pub per_file: Vec<FileLineCount>,
    pub total_lines: u64,
    pub total_code_lines: u64,
    pub monolithic: Vec<FileLineCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLineCount {
    pub path: std::path::PathBuf,
    pub lines: u64,
    pub code_lines: u64,
    pub bytes: u64,
}

/// Per-language comment syntax used when classifying code vs. comment lines.
#[derive(Debug, Clone, Copy)]
enum CommentSyntax {
    /// `//` line comments and `/* ... */` block comments (C-family).
    Slash,
    /// `#` line comments (Python and similar).
    Hash,
    /// No known comment syntax — every non-blank line counts as code.
    None,
}

fn comment_syntax_for(ext: &str) -> CommentSyntax {
    match ext {
        "rs" | "ts" | "tsx" | "js" | "jsx" | "go" | "java" | "c" | "cpp" | "cc" | "cxx"
        | "h" | "hpp" | "hh" | "hxx" | "swift" | "kt" | "kts" | "scala" | "cs" | "m"
        | "mm" => CommentSyntax::Slash,
        "py" | "rb" | "sh" | "bash" | "zsh" | "pl" | "yaml" | "yml" | "toml" => {
            CommentSyntax::Hash
        }
        _ => CommentSyntax::None,
    }
}

/// Count lines across `entries` whose extension matches `extensions`
/// (case-insensitive, no leading dot). Files with a NUL byte in their first
/// 8 KiB are treated as binary and skipped.
pub fn count_entries(
    entries: &[FileEntry],
    extensions: &[String],
    monolith_threshold: usize,
) -> Result<LineCountReport> {
    let normalized: Vec<String> = extensions
        .iter()
        .map(|e| e.trim_start_matches('.').to_ascii_lowercase())
        .collect();

    let per_file: Vec<FileLineCount> = entries
        .par_iter()
        .filter(|e| e.kind == EntryKind::File)
        .filter_map(|e| {
            let ext = file_extension(&e.path)?;
            if !normalized.iter().any(|n| n == &ext) {
                return None;
            }
            let syntax = comment_syntax_for(&ext);
            count_file(&e.path, syntax).map(|(lines, code_lines, bytes)| FileLineCount {
                path: e.path.clone(),
                lines,
                code_lines,
                bytes,
            })
        })
        .collect();

    let total_lines = per_file.iter().map(|f| f.lines).sum();
    let total_code_lines = per_file.iter().map(|f| f.code_lines).sum();
    let threshold = monolith_threshold as u64;
    let mut monolithic: Vec<FileLineCount> = per_file
        .iter()
        .filter(|f| f.lines >= threshold)
        .cloned()
        .collect();
    monolithic.sort_by(|a, b| b.lines.cmp(&a.lines));

    Ok(LineCountReport {
        per_file,
        total_lines,
        total_code_lines,
        monolithic,
    })
}

/// Convenience wrapper that walks `root` and counts matching files.
pub fn count(
    root: &Path,
    extensions: &[String],
    monolith_threshold: usize,
) -> Result<LineCountReport> {
    let opts = crate::scanner::ScanOptions {
        roots: vec![root.to_path_buf()],
        ..Default::default()
    };
    let report = crate::scanner::walker::scan(&opts)?;
    count_entries(&report.entries, extensions, monolith_threshold)
}

fn file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
}

fn count_file(path: &Path, syntax: CommentSyntax) -> Option<(u64, u64, u64)> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut lines: u64 = 0;
    let mut code_lines: u64 = 0;
    let mut bytes: u64 = 0;
    let mut buf: Vec<u8> = Vec::with_capacity(256);
    let mut checked_binary = false;
    let mut in_block_comment = false;

    loop {
        buf.clear();
        let n = reader.read_until(b'\n', &mut buf).ok()?;
        if n == 0 {
            break;
        }
        if !checked_binary {
            let probe_end = buf.len().min(8192);
            if buf[..probe_end].contains(&0) {
                return None;
            }
            checked_binary = true;
        }
        bytes += n as u64;
        lines += 1;

        let content_end = trim_trailing_newline(&buf);
        let line = &buf[..content_end];
        if is_code_line(line, syntax, &mut in_block_comment) {
            code_lines += 1;
        }
    }

    Some((lines, code_lines, bytes))
}

fn trim_trailing_newline(buf: &[u8]) -> usize {
    let mut end = buf.len();
    if end > 0 && buf[end - 1] == b'\n' {
        end -= 1;
        if end > 0 && buf[end - 1] == b'\r' {
            end -= 1;
        }
    }
    end
}

fn trim_ascii(s: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = s.len();
    while start < end && matches!(s[start], b' ' | b'\t' | b'\r') {
        start += 1;
    }
    while end > start && matches!(s[end - 1], b' ' | b'\t' | b'\r') {
        end -= 1;
    }
    &s[start..end]
}

/// Returns `true` if the line contains at least one character of source code,
/// after stripping leading/trailing whitespace and any leading comment that
/// occupies the entire line. Block-comment state (`/* ... */`) is carried
/// across calls via `in_block`.
fn is_code_line(line: &[u8], syntax: CommentSyntax, in_block: &mut bool) -> bool {
    let mut rest = trim_ascii(line);

    if *in_block {
        match find(rest, b"*/") {
            Some(idx) => {
                *in_block = false;
                rest = trim_ascii(&rest[idx + 2..]);
            }
            None => return false,
        }
    }

    loop {
        if rest.is_empty() {
            return false;
        }
        match syntax {
            CommentSyntax::Slash => {
                if rest.starts_with(b"//") {
                    return false;
                }
                if rest.starts_with(b"/*") {
                    let after_open = &rest[2..];
                    match find(after_open, b"*/") {
                        Some(idx) => {
                            rest = trim_ascii(&after_open[idx + 2..]);
                            continue;
                        }
                        None => {
                            *in_block = true;
                            return false;
                        }
                    }
                }
                return true;
            }
            CommentSyntax::Hash => {
                return !rest.starts_with(b"#");
            }
            CommentSyntax::None => return true,
        }
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_tmp(contents: &str, ext: &str) -> NamedTempFile {
        let mut f = tempfile::Builder::new()
            .suffix(&format!(".{ext}"))
            .tempfile()
            .unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        f.flush().unwrap();
        f
    }

    #[test]
    fn counts_total_and_code_lines_for_rust() {
        let src = "// header\nfn main() {\n    // inline\n    let x = 1; // trailing\n\n    /* block\n       still block */\n    println!(\"{}\", x);\n}\n";
        let f = write_tmp(src, "rs");
        let (total, code, _bytes) =
            count_file(f.path(), comment_syntax_for("rs")).expect("count");
        assert_eq!(total, 9);
        // Lines 2, 4, 8, 9 are code. Line 5 is blank, 1/3/6/7 are comments.
        assert_eq!(code, 4);
    }

    #[test]
    fn counts_python_hash_comments() {
        let src = "# comment\nx = 1\n\n  # indented comment\ny = 2  # trailing\n";
        let f = write_tmp(src, "py");
        let (total, code, _bytes) =
            count_file(f.path(), comment_syntax_for("py")).expect("count");
        assert_eq!(total, 5);
        assert_eq!(code, 2);
    }

    #[test]
    fn unknown_extension_treats_every_nonblank_line_as_code() {
        let src = "hello\n\nworld\n";
        let f = write_tmp(src, "txt");
        let (total, code, _bytes) =
            count_file(f.path(), comment_syntax_for("txt")).expect("count");
        assert_eq!(total, 3);
        assert_eq!(code, 2);
    }

    #[test]
    fn single_line_block_comment_is_not_code() {
        let src = "/* one-liner */\nlet a = 1;\n";
        let f = write_tmp(src, "rs");
        let (_total, code, _bytes) =
            count_file(f.path(), comment_syntax_for("rs")).expect("count");
        assert_eq!(code, 1);
    }
}
