fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Assets/filecanopy.ico");

    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("windows") {
        return;
    }

    let mut res = winresource::WindowsResource::new();
    res.set_icon("Assets/filecanopy.ico");
    res.compile().expect("compile windows resource");
}
