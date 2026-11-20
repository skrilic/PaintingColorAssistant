//! Web build only (wasm32): the browser side of opening a picture, saving a file and
//! keeping data, implemented in `web/pca.js` (a miniquad plugin).

/// Must equal `version` in web/pca.js; miniquad logs a mismatch in the console.
const PLUGIN_VERSION: u32 = 4;

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn pca_open_picture();
    fn pca_picture_button(x: f32, y: f32, w: f32, h: f32);
    fn pca_picture_len() -> i32;
    fn pca_picture_name_len() -> i32;
    fn pca_picture_take(data: *mut u8, name: *mut u8);
    fn pca_save_file(name: *const u8, name_len: usize, mime: *const u8, mime_len: usize, data: *const u8, data_len: usize);
    fn pca_storage_len(key: *const u8, key_len: usize) -> i32;
    fn pca_storage_get(key: *const u8, key_len: usize, out: *mut u8);
    fn pca_storage_set(key: *const u8, key_len: usize, value: *const u8, value_len: usize) -> i32;
    #[allow(clippy::too_many_arguments)]
    fn pca_text_field(
        id: *const u8,
        id_len: usize,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        value: *const u8,
        value_len: usize,
        placeholder: *const u8,
        placeholder_len: usize,
    ) -> i32;
    fn pca_text_field_take(out: *mut u8);
    fn pca_frame_end();
    fn pca_open_url(url: *const u8, url_len: usize);
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

/// Places the browser's file input over the "Open picture" button at `r` (logical px = CSS
/// px) for this frame. Safari and every browser on iOS open a file chooser only from the
/// click or tap itself, so the button's own reaction a frame later would come too late.
pub fn picture_button(r: macroquad::math::Rect) {
    unsafe { pca_picture_button(r.x, r.y, r.w, r.h) }
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

/// Places the browser's input for text field `id` over `r` (logical px = CSS px) and shows
/// `value` in it. Returns whether it has the keyboard focus, and the text typed since the
/// last frame, if any.
pub fn text_field(id: &str, r: macroquad::math::Rect, value: &str, placeholder: &str) -> (bool, Option<String>) {
    let code = unsafe {
        pca_text_field(id.as_ptr(), id.len(), r.x, r.y, r.w, r.h, value.as_ptr(), value.len(), placeholder.as_ptr(), placeholder.len())
    };
    let focused = code & 1 == 1;
    let typed = (code >= 2).then(|| {
        let mut out = vec![0u8; (code >> 1) as usize - 1];
        unsafe { pca_text_field_take(out.as_mut_ptr()) };
        String::from_utf8_lossy(&out).into_owned()
    });
    (focused, typed)
}

/// Opens `url` in a new browser tab.
pub fn open_url(url: &str) {
    unsafe { pca_open_url(url.as_ptr(), url.len()) }
}

/// End of a frame: hides the inputs of text fields and buttons that were not drawn in it.
pub fn frame_end() {
    unsafe { pca_frame_end() }
}
