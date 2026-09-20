//! Embeds Windows resources.

fn main() {
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=packaging/windows/magicspot.ico");
        println!("cargo:rerun-if-env-changed=MAGICSPOT_DISPLAY_NAME");
        let display_name =
            std::env::var("MAGICSPOT_DISPLAY_NAME").unwrap_or_else(|_| "MagicSpot".to_string());
        let mut resource = winresource::WindowsResource::new();
        resource
            .set_icon("packaging/windows/magicspot.ico")
            .set("ProductName", &display_name)
            .set("FileDescription", &display_name);
        if let Err(error) = resource.compile() {
            println!("cargo:warning=Windows resources not embedded: {error}");
        }
    }
}
