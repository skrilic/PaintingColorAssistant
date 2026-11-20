// Painting Colour Assistant, web build: the browser side of src/web.rs, as a miniquad
// plugin. Load after gl.js and before load(...).
"use strict";

(function () {
    const encoder = new TextEncoder();
    const decoder = new TextDecoder();
    const bytes = (ptr, len) => new Uint8Array(wasm_memory.buffer, ptr, len);
    const text = (ptr, len) => decoder.decode(bytes(ptr, len).slice());

    // Browsers open a file chooser or start a download only shortly after a click or tap.
    // App buttons react when released, a frame later, which is normally still in time; when
    // the browser says it is not (navigator.userActivation), a small sheet with a real
    // button asks for one more tap. WebKit (Safari, and every browser on iOS) reports the
    // click as recent but still refuses anything not done inside its event handler, so
    // there the sheet is always shown.
    const webkit = /^Apple/.test(navigator.vendor || "");

    function inTime() {
        if (webkit) return false;
        const ua = navigator.userActivation;
        return ua ? ua.isActive : !window.matchMedia("(pointer: coarse)").matches;
    }

    function runInGesture(label, action) {
        if (inTime()) {
            action();
            return;
        }
        const sheet = document.createElement("div");
        sheet.className = "pca-sheet";
        const box = document.createElement("div");
        const go = document.createElement("button");
        go.textContent = label;
        go.className = "pca-go";
        const cancel = document.createElement("button");
        cancel.textContent = "Cancel";
        const close = () => sheet.remove();
        go.addEventListener("click", () => { close(); action(); });
        cancel.addEventListener("click", close);
        sheet.addEventListener("click", (e) => { if (e.target === sheet) close(); });
        box.append(go, cancel);
        sheet.append(box);
        document.body.appendChild(sheet);
    }

    // ---------- open a picture ----------

    // A see-through label for the file input lies over "Open picture..." (pca_picture_button),
    // so the click or tap itself opens the chooser, as WebKit requires. pca_open_picture is
    // for the app asking at another time; it goes through the sheet when needed.
    let input = null;
    let pick = null;
    let pickSeen = false;
    let picture = null; // { name: Uint8Array, data: Uint8Array } until the app takes it

    function chooser() {
        if (input == null) {
            input = document.createElement("input");
            input.type = "file";
            input.accept = "image/*";
            input.id = "pca-file";
            // hidden from view but not display: none, which some WebKit versions won't open
            input.className = "pca-file";
            input.addEventListener("change", async () => {
                const file = input.files[0];
                input.value = "";
                const canvas = document.getElementById("glcanvas");
                if (canvas) canvas.focus();
                if (file) {
                    const data = new Uint8Array(await file.arrayBuffer());
                    picture = { name: encoder.encode(file.name), data };
                }
            });
            pick = document.createElement("label");
            pick.htmlFor = input.id;
            pick.className = "pca-pick";
            document.body.append(input, pick);
        }
        return input;
    }

    // ---------- text fields ----------

    // Each text field of the app gets a real input placed over it, see-through until it has
    // the focus, so that tapping a field brings up the phone's or tablet's keyboard.
    const fields = new Map(); // id -> { el, typed: string | null }
    const seen = new Set();
    let typedOut = null;

    function field(id) {
        let f = fields.get(id);
        if (f) return f;
        const el = document.createElement("input");
        el.type = "text";
        el.className = "pca-field";
        el.maxLength = 60;
        el.autocomplete = "off";
        el.spellcheck = false;
        f = { el, typed: null };
        el.addEventListener("input", () => { f.typed = el.value; });
        el.addEventListener("keydown", (e) => {
            if (e.key === "Enter" || e.key === "Escape") el.blur();
        });
        el.addEventListener("blur", () => {
            const canvas = document.getElementById("glcanvas");
            if (canvas) canvas.focus();
        });
        document.body.appendChild(el);
        fields.set(id, f);
        return f;
    }

    // ---------- local storage ----------

    // pca_storage_len encodes the value; pca_storage_get then copies that same encoding.
    let stored = null;

    function register_plugin(importObject) {
        const env = importObject.env;

        env.pca_text_field = (idPtr, idLen, x, y, w, h, valuePtr, valueLen, phPtr, phLen) => {
            const id = text(idPtr, idLen);
            const f = field(id);
            const el = f.el;
            seen.add(id);
            el.style.display = "block";
            el.style.left = x + "px";
            el.style.top = y + "px";
            el.style.width = w + "px";
            el.style.height = h + "px";
            el.placeholder = text(phPtr, phLen);
            const focused = document.activeElement === el;
            if (f.typed !== null) {
                typedOut = encoder.encode(f.typed);
                f.typed = null;
                return ((typedOut.length + 1) << 1) | (focused ? 1 : 0);
            }
            const value = text(valuePtr, valueLen);
            if (!focused && el.value !== value) el.value = value;
            return focused ? 1 : 0;
        };
        env.pca_text_field_take = (outPtr) => {
            if (typedOut != null) bytes(outPtr, typedOut.length).set(typedOut);
            typedOut = null;
        };
        env.pca_frame_end = () => {
            for (const [id, f] of fields) {
                if (!seen.has(id) && f.el.style.display !== "none") {
                    if (document.activeElement === f.el) f.el.blur();
                    f.el.style.display = "none";
                }
            }
            seen.clear();
            if (!pickSeen && pick != null) pick.style.display = "none";
            pickSeen = false;
        };

        env.pca_picture_button = (x, y, w, h) => {
            chooser();
            pickSeen = true;
            pick.style.display = "block";
            pick.style.left = x + "px";
            pick.style.top = y + "px";
            pick.style.width = w + "px";
            pick.style.height = h + "px";
        };
        env.pca_open_picture = () => runInGesture("Choose a picture", () => chooser().click());
        env.pca_picture_len = () => (picture == null ? -1 : picture.data.length);
        env.pca_picture_name_len = () => (picture == null ? 0 : picture.name.length);
        env.pca_picture_take = (dataPtr, namePtr) => {
            if (picture == null) return;
            bytes(dataPtr, picture.data.length).set(picture.data);
            bytes(namePtr, picture.name.length).set(picture.name);
            picture = null;
        };

        env.pca_open_url = (urlPtr, urlLen) => {
            const url = text(urlPtr, urlLen);
            runInGesture("Open " + new URL(url).host, () => window.open(url, "_blank", "noopener"));
        };

        env.pca_save_file = (namePtr, nameLen, mimePtr, mimeLen, dataPtr, dataLen) => {
            const name = text(namePtr, nameLen);
            const blob = new Blob([bytes(dataPtr, dataLen).slice()], { type: text(mimePtr, mimeLen) });
            runInGesture("Save " + name, () => {
                const url = URL.createObjectURL(blob);
                const a = document.createElement("a");
                a.href = url;
                a.download = name;
                document.body.appendChild(a);
                a.click();
                a.remove();
                setTimeout(() => URL.revokeObjectURL(url), 60000);
            });
        };

        env.pca_storage_len = (keyPtr, keyLen) => {
            let value = null;
            try {
                value = window.localStorage.getItem(text(keyPtr, keyLen));
            } catch (e) {
                // storage turned off (private mode in some browsers)
            }
            stored = value == null ? null : encoder.encode(value);
            return stored == null ? -1 : stored.length;
        };
        env.pca_storage_get = (keyPtr, keyLen, outPtr) => {
            if (stored != null) bytes(outPtr, stored.length).set(stored);
            stored = null;
        };
        env.pca_storage_set = (keyPtr, keyLen, valuePtr, valueLen) => {
            try {
                window.localStorage.setItem(text(keyPtr, keyLen), text(valuePtr, valueLen));
                return 1;
            } catch (e) {
                return 0;
            }
        };
    }

    function on_init() {
        const loading = document.getElementById("loading");
        if (loading) loading.remove();
        // Installed as an app: ask the browser not to clear the saved palettes when it
        // runs short of space (some browsers would ask the user on a plain web page).
        if (window.matchMedia("(display-mode: standalone)").matches && navigator.storage && navigator.storage.persist) {
            navigator.storage.persist();
        }
    }

    miniquad_add_plugin({ register_plugin, on_init, name: "pca", version: 4 });
})();
