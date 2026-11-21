//! Where the app keeps its JSON data (palettes, recent pictures): a file in the settings
//! folder on the desktop, the browser's local storage in the web build.

/// Folder name inside the settings folder; also the key prefix in local storage.
const APP_DIR: &str = "PaletteCraft";

#[cfg(not(target_arch = "wasm32"))]
fn path(name: &str) -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join(APP_DIR).join(name))
}

/// The saved text called `name` (e.g. "palettes.json"), if there is one.
pub fn read(name: &str) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    return std::fs::read_to_string(path(name)?).ok();
    #[cfg(target_arch = "wasm32")]
    return crate::web::storage_get(&format!("{APP_DIR}/{name}"));
}

pub fn write(name: &str, text: &str) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = path(name).ok_or("no settings folder")?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, text).map_err(|e| e.to_string())
    }
    #[cfg(target_arch = "wasm32")]
    crate::web::storage_set(&format!("{APP_DIR}/{name}"), text)
}
