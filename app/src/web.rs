//! Web build only (wasm32): the browser side of opening a picture, saving a file and
//! keeping data, implemented in `web/pca.js` (a miniquad plugin).

/// Must equal `version` in web/pca.js; miniquad logs a mismatch in the console.
const PLUGIN_VERSION: u32 = 1;

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn pca_open_picture();
    fn pca_picture_len() -> i32;
    fn pca_picture_name_len() -> i32;
    fn pca_picture_take(data: *mut u8, name: *mut u8);
    fn pca_save_file(name: *const u8, name_len: usize, mime: *const u8, mime_len: usize, data: *const u8, data_len: usize);
    fn pca_storage_len(key: *const u8, key_len: usize) -> i32;
    fn pca_storage_get(key: *const u8, key_len: usize, out: *mut u8);
    fn pca_storage_set(key: *const u8, key_len: usize, value: *const u8, value_len: usize) -> i32;
}

/// Read by miniquad's plugin loader to check that web/pca.js matches this build.
#[unsafe(no_mangle)]
pub extern "C" fn pca_crate_version() -> u32 {
    PLUGIN_VERSION
}

/// Shows the browser's file chooser (on phones: photos, camera or files). The chosen
/// picture arrives later through `take_picture`.
pub fn open_picture() {
    unsafe { pca_open_picture() }
}

/// The picture chosen with `open_picture`, once the browser has read it: (file name, bytes).
pub fn take_picture() -> Option<(String, Vec<u8>)> {
    let len = unsafe { pca_picture_len() };
    if len < 0 {
        return None;
    }
    let mut data = vec![0u8; len as usize];
    let mut name = vec![0u8; unsafe { pca_picture_name_len() }.max(0) as usize];
    unsafe { pca_picture_take(data.as_mut_ptr(), name.as_mut_ptr()) };
    Some((String::from_utf8_lossy(&name).into_owned(), data))
}

/// Hands `data` to the browser as a download named `name`.
pub fn save_file(name: &str, mime: &str, data: &[u8]) {
    unsafe { pca_save_file(name.as_ptr(), name.len(), mime.as_ptr(), mime.len(), data.as_ptr(), data.len()) }
}

/// Text kept in the browser's local storage under `key`.
pub fn storage_get(key: &str) -> Option<String> {
    let len = unsafe { pca_storage_len(key.as_ptr(), key.len()) };
    if len < 0 {
        return None;
    }
    let mut out = vec![0u8; len as usize];
    unsafe { pca_storage_get(key.as_ptr(), key.len(), out.as_mut_ptr()) };
    String::from_utf8(out).ok()
}

pub fn storage_set(key: &str, value: &str) -> Result<(), String> {
    match unsafe { pca_storage_set(key.as_ptr(), key.len(), value.as_ptr(), value.len()) } {
        1 => Ok(()),
        _ => Err("the browser refused to store it (storage full or turned off)".into()),
    }
}
