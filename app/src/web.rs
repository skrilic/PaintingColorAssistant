//! Web build only (wasm32): the browser side of opening a picture, saving a file, keeping
//! data and checking for updates, implemented in `web/palettecraft.js` (a miniquad plugin).

/// Must equal `version` in web/palettecraft.js; miniquad logs a mismatch in the console.
const PLUGIN_VERSION: u32 = 6;

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn pc_open_picture();
    fn pc_picture_button(x: f32, y: f32, w: f32, h: f32);
    fn pc_picture_len() -> i32;
    fn pc_picture_name_len() -> i32;
    fn pc_picture_take(data: *mut u8, name: *mut u8);
    fn pc_save_file(name: *const u8, name_len: usize, mime: *const u8, mime_len: usize, data: *const u8, data_len: usize);
    fn pc_storage_len(key: *const u8, key_len: usize) -> i32;
    fn pc_storage_get(key: *const u8, key_len: usize, out: *mut u8);
    fn pc_storage_set(key: *const u8, key_len: usize, value: *const u8, value_len: usize) -> i32;
    #[allow(clippy::too_many_arguments)]
    fn pc_text_field(
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
    fn pc_text_field_take(out: *mut u8);
    fn pc_frame_end();
    fn pc_open_url(url: *const u8, url_len: usize);
    fn pc_check_update(url: *const u8, url_len: usize);
    fn pc_update_len() -> i32;
    fn pc_update_take(out: *mut u8);
    fn pc_reload_app();
    fn pc_picture_store(key: *const u8, key_len: usize, data: *const u8, data_len: usize);
    fn pc_pictures_keep(keys: *const u8, keys_len: usize);
    fn pc_picture_load(key: *const u8, key_len: usize, name: *const u8, name_len: usize);
}

/// Read by miniquad's plugin loader to check that web/palettecraft.js matches this build.
#[unsafe(no_mangle)]
pub extern "C" fn pc_crate_version() -> u32 {
    PLUGIN_VERSION
}

/// Shows the browser's file chooser (on phones: photos, camera or files). The chosen
/// picture arrives later through `take_picture`.
pub fn open_picture() {
    unsafe { pc_open_picture() }
}

/// Places the browser's file input over the "Open picture" button at `r` (logical px = CSS
/// px) for this frame. Safari and every browser on iOS open a file chooser only from the
/// click or tap itself, so the button's own reaction a frame later would come too late.
pub fn picture_button(r: macroquad::math::Rect) {
    unsafe { pc_picture_button(r.x, r.y, r.w, r.h) }
}

/// The picture chosen with `open_picture`, once the browser has read it: (file name, bytes).
pub fn take_picture() -> Option<(String, Vec<u8>)> {
    let len = unsafe { pc_picture_len() };
    if len < 0 {
        return None;
    }
    let mut data = vec![0u8; len as usize];
    let mut name = vec![0u8; unsafe { pc_picture_name_len() }.max(0) as usize];
    unsafe { pc_picture_take(data.as_mut_ptr(), name.as_mut_ptr()) };
    Some((String::from_utf8_lossy(&name).into_owned(), data))
}

/// Keeps a copy of a picture in the browser (IndexedDB) under `key`, its key in the recent
/// list, so it can be opened from there again.
pub fn store_picture(key: &str, data: &[u8]) {
    unsafe { pc_picture_store(key.as_ptr(), key.len(), data.as_ptr(), data.len()) }
}

/// Deletes the kept copies of all pictures but those under `keys`.
pub fn keep_pictures(keys: &[String]) {
    let keys = keys.join("\n");
    unsafe { pc_pictures_keep(keys.as_ptr(), keys.len()) }
}

/// Reads the copy kept under `key`; it arrives through `take_picture` named `name`, with no
/// bytes when the browser no longer has it.
pub fn load_picture(key: &str, name: &str) {
    unsafe { pc_picture_load(key.as_ptr(), key.len(), name.as_ptr(), name.len()) }
}

/// Hands `data` to the browser as a download named `name`.
pub fn save_file(name: &str, mime: &str, data: &[u8]) {
    unsafe { pc_save_file(name.as_ptr(), name.len(), mime.as_ptr(), mime.len(), data.as_ptr(), data.len()) }
}

/// Text kept in the browser's local storage under `key`.
pub fn storage_get(key: &str) -> Option<String> {
    let len = unsafe { pc_storage_len(key.as_ptr(), key.len()) };
    if len < 0 {
        return None;
    }
    let mut out = vec![0u8; len as usize];
    unsafe { pc_storage_get(key.as_ptr(), key.len(), out.as_mut_ptr()) };
    String::from_utf8(out).ok()
}

pub fn storage_set(key: &str, value: &str) -> Result<(), String> {
    match unsafe { pc_storage_set(key.as_ptr(), key.len(), value.as_ptr(), value.len()) } {
        1 => Ok(()),
        _ => Err("the browser refused to store it (storage full or turned off)".into()),
    }
}

/// Places the browser's input for text field `id` over `r` (logical px = CSS px) and shows
/// `value` in it. Returns whether it has the keyboard focus, and the text typed since the
/// last frame, if any.
pub fn text_field(id: &str, r: macroquad::math::Rect, value: &str, placeholder: &str) -> (bool, Option<String>) {
    let code = unsafe {
        pc_text_field(id.as_ptr(), id.len(), r.x, r.y, r.w, r.h, value.as_ptr(), value.len(), placeholder.as_ptr(), placeholder.len())
    };
    let focused = code & 1 == 1;
    let typed = (code >= 2).then(|| {
        let mut out = vec![0u8; (code >> 1) as usize - 1];
        unsafe { pc_text_field_take(out.as_mut_ptr()) };
        String::from_utf8_lossy(&out).into_owned()
    });
    (focused, typed)
}

/// Opens `url` in a new browser tab.
pub fn open_url(url: &str) {
    unsafe { pc_open_url(url.as_ptr(), url.len()) }
}

/// Asks `url` (GitHub's latest release) for its `tag_name`; the answer arrives through
/// `take_update`.
pub fn check_update(url: &str) {
    unsafe { pc_check_update(url.as_ptr(), url.len()) }
}

/// The answer to `check_update` once it has come: the release's tag, or why it failed.
pub fn take_update() -> Option<Result<String, String>> {
    let len = unsafe { pc_update_len() };
    if len < 0 {
        return None;
    }
    let mut out = vec![0u8; len as usize];
    unsafe { pc_update_take(out.as_mut_ptr()) };
    let text = String::from_utf8_lossy(&out).into_owned();
    Some(match text.split_once('\t') {
        Some(("ok", tag)) => Ok(tag.to_string()),
        Some((_, e)) => Err(e.to_string()),
        None => Err(text),
    })
}

/// Loads the app again, with its newest files (a waiting service worker takes over first).
pub fn reload_app() {
    unsafe { pc_reload_app() }
}

/// End of a frame: hides the inputs of text fields and buttons that were not drawn in it.
pub fn frame_end() {
    unsafe { pc_frame_end() }
}
