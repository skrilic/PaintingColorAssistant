// Painting Colour Assistant, web build: the browser side of src/web.rs, as a miniquad
// plugin. Load after gl.js and before load(...).
"use strict";

(function () {
    const encoder = new TextEncoder();
    const decoder = new TextDecoder();
    const bytes = (ptr, len) => new Uint8Array(wasm_memory.buffer, ptr, len);
    const text = (ptr, len) => decoder.decode(bytes(ptr, len).slice());

    // Browsers (Safari on iOS most strictly) open a file chooser or start a download only
    // during a click or tap. App buttons react when pressed, and the app sees that a frame
    // later, so the action runs when the finger or mouse button is lifted; if that already
    // happened, it runs straight away and relies on the browser's short grace period.
    let pointerDown = false;
    let afterRelease = [];
    function runInGesture(action) {
        if (pointerDown) {
            afterRelease.push(action);
        } else {
            action();
        }
    }
    function released() {
        pointerDown = false;
        const actions = afterRelease;
        afterRelease = [];
        actions.forEach((a) => a());
    }
    window.addEventListener("mousedown", () => { pointerDown = true; }, true);
    window.addEventListener("touchstart", () => { pointerDown = true; }, true);
    window.addEventListener("mouseup", released, true);
    window.addEventListener("touchend", released, true);
    window.addEventListener("touchcancel", released, true);

    // ---------- open a picture ----------

    let input = null;
    let picture = null; // { name: Uint8Array, data: Uint8Array } until the app takes it

    function chooser() {
        if (input == null) {
            input = document.createElement("input");
            input.type = "file";
            input.accept = "image/*";
            input.style.display = "none";
            input.addEventListener("change", async () => {
                const file = input.files[0];
                input.value = "";
                if (file) {
                    const data = new Uint8Array(await file.arrayBuffer());
                    picture = { name: encoder.encode(file.name), data };
                }
            });
            document.body.appendChild(input);
        }
        return input;
    }

    // ---------- local storage ----------

    // pca_storage_len encodes the value; pca_storage_get then copies that same encoding.
    let stored = null;

    function register_plugin(importObject) {
        const env = importObject.env;

        env.pca_open_picture = () => runInGesture(() => chooser().click());
        env.pca_picture_len = () => (picture == null ? -1 : picture.data.length);
        env.pca_picture_name_len = () => (picture == null ? 0 : picture.name.length);
        env.pca_picture_take = (dataPtr, namePtr) => {
            if (picture == null) return;
            bytes(dataPtr, picture.data.length).set(picture.data);
            bytes(namePtr, picture.name.length).set(picture.name);
            picture = null;
        };

        env.pca_save_file = (namePtr, nameLen, mimePtr, mimeLen, dataPtr, dataLen) => {
            const name = text(namePtr, nameLen);
            const blob = new Blob([bytes(dataPtr, dataLen).slice()], { type: text(mimePtr, mimeLen) });
            runInGesture(() => {
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
    }

    miniquad_add_plugin({ register_plugin, on_init, name: "pca", version: 1 });
})();
