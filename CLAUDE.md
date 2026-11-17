# CLAUDE.md

Painting Colour Assistant: a Rust + macroquad desktop app that turns a reference picture
into value/colour studies, composition overlays and Kubelka-Munk mixing recipes from the
user's paint palettes. See README.md for the feature overview.

## Layout

- All source lives in `app/` (a single binary crate, edition 2024). Run cargo from `app/`.
- `app/vendor/miniquad/` is a patched miniquad 0.4.11 used via `[patch.crates-io]`.
- Root files `craft.json`, `feed.json`, `investigation.cany/`, `checks/`, `capture/` are
  state from the "Craft Anything" tool that generated the project. Do not edit them by hand.

## Commands (from `app/`)

```sh
cargo build
cargo test                                    # km.rs unit tests
cargo run -- --self-test shots/selftest.json  # scripted end-to-end test
cargo run -- --screenshot shots/now.png       # render sample landscape to PNG
cargo run -- --palettes                       # open on the Palettes screen
cargo run -- <image path>                     # open with a picture
```

- The self-test always exits 0. Read the JSON: every step must have `"passed": true`.
- `--screenshot` / `--self-test` open a real window and use `palette::default_store()`;
  they never read or write the user's `palettes.json`.
- After UI changes, run both the self-test and a screenshot and look at the PNG.

## Architecture

- `main.rs` – arg parsing, window config, main loop, screenshot capture. Sets
  `MINIQUAD_NO_ACTIVATE=1` in background modes.
- `app.rs` – `App` state (`Screen::{Palettes, Painting}`, `View`, crop, picked colour,
  incremental recipe search). Screens are split into `app/painting_view.rs` and
  `app/palettes_view.rs` as `impl App` blocks.
- `ui.rs` – own immediate-mode widgets. Every widget registers its rect under a string id;
  `selftest.rs` clicks widgets by those ids. **Renaming a widget id breaks the self-test.**
- `selftest.rs` – queue of `Action`s (click, drag, type, wait, check) producing synthetic
  `Input` per frame; checks read `App` state directly.
- `km.rs` – two-constant Kubelka-Munk per linear RGB channel. Watercolor = film over paper
  (`PAPER`), lightened by `WATER_LOADS`; acrylic/oil = `r_inf`, lightened with white
  (`WHITE_PARTS`). Recipe search is time-sliced (`RECIPE_BUDGET_MS` per frame).
- `imgproc.rs` – working copy capped at `WORK_MAX_SIDE`; crops apply to the full original.
  Colour levels = seeded k-means++ in Lab (deterministic).
- `colour.rs` – sRGB/linear/Lab, CIE76 ΔE. `palette.rs` – `Paint`, `Palette`, `Store`,
  45-pigment `LIBRARY`, JSON persistence in `dirs::config_dir()`.
- `grid.rs` – rule of thirds, Fibonacci spiral, square grid overlays.

## Conventions

- Model and UI constants are documented in `app/MECHANICS.md` and `app/mechanics.json`
  with a source (request or estimated). When you change a constant, update both files.
- Use British "colour" in code and UI text (the crate name keeps `color`).
- Keep the build warning-free, including the vendored miniquad.

## Vendored miniquad

Local changes vs upstream 0.4.11 (keep this list current when touching `vendor/`):

1. `src/native/macos.rs`: honour `MINIQUAD_NO_ACTIVATE` (don't activate/focus the window).
2. `src/lib.rs`: `set_or_replace_display` is `#[cfg(target_os = "android")]` (dead code elsewhere).
3. `src/native/linux_x11/clipboard.rs`: `MESSAGE` is a `Mutex<Option<String>>` instead of
   `static mut` (fixes the `static_mut_refs` warning).
