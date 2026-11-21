//! Windows only: embed the app icon and version info in the .exe.

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "PaletteCraft");
        res.set("FileDescription", "PaletteCraft");
        res.compile().expect("could not embed the Windows icon");
    }
}
