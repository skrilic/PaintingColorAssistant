//! Opening a web page in the user's browser (the "Support on Ko-fi" button).

/// The author's Ko-fi page, for tips and donations.
pub const KOFI_URL: &str = "https://ko-fi.com/Q6X32AWI41";

/// Opens `url` in the default browser; in the web build, in a new tab.
pub fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        crate::web::open_url(url);
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::process::Command;
        #[cfg(target_os = "macos")]
        let mut cmd = Command::new("open");
        #[cfg(target_os = "windows")]
        let mut cmd = {
            // `start` is built into cmd; the empty string is the window title
            let mut c = Command::new("cmd");
            c.args(["/C", "start", ""]);
            c
        };
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let mut cmd = Command::new("xdg-open");
        cmd.arg(url).spawn().map(|_| ()).map_err(|e| e.to_string())
    }
}
