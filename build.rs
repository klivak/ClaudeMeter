fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon_app.ico");
        res.set("ProductName", "ClaudeMeter");
        res.set(
            "FileDescription",
            "AI Usage Monitor for Windows — ultra-lightweight, built in Rust",
        );
        res.set("LegalCopyright", "MIT License — klivak");
        res.set_manifest_file("app.manifest");
        if let Err(e) = res.compile() {
            eprintln!("Warning: could not compile Windows resources: {e}");
            eprintln!("This is expected on non-Windows or without Windows SDK.");
        }
    }
}
