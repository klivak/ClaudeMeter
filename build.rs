fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon_app.ico");
        // Embed tray icons as resources (IDs 101-104)
        res.set_icon_with_id("assets/icon_green.ico", "101");
        res.set_icon_with_id("assets/icon_yellow.ico", "102");
        res.set_icon_with_id("assets/icon_red.ico", "103");
        res.set_icon_with_id("assets/icon_gray.ico", "104");
        res.set("ProductName", "ClaudeMeter");
        res.set("FileDescription", "ClaudeMeter");
        res.set("LegalCopyright", "MIT License — klivak");
        res.set_manifest_file("app.manifest");
        if let Err(e) = res.compile() {
            eprintln!("Warning: could not compile Windows resources: {e}");
            eprintln!("This is expected on non-Windows or without Windows SDK.");
        }
    }
}
