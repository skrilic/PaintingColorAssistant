//! Painting Colour Assistant: value and colour studies of a picture, composition overlays,
//! and Kubelka-Munk mixing recipes from the artist's own paint palettes.

mod app;
mod colour;
mod grid;
mod imgproc;
mod km;
mod palette;
mod selftest;
mod ui;

use app::{App, View};
use grid::Overlay;
use macroquad::prelude::*;
use std::path::PathBuf;
use ui::{Input, Ui};

#[derive(Default)]
struct Args {
    screenshot: Option<String>,
    self_test: Option<String>,
    image: Option<PathBuf>,
    palettes: bool,
}

impl Args {
    fn parse() -> Self {
        let mut a = Args::default();
        let mut it = std::env::args().skip(1);
        while let Some(s) = it.next() {
            match s.as_str() {
                "--screenshot" => a.screenshot = it.next(),
                "--self-test" => a.self_test = it.next(),
                "--palettes" => a.palettes = true,
                _ if !s.starts_with("--") => a.image = Some(PathBuf::from(s)),
                _ => {}
            }
        }
        a
    }

    fn background(&self) -> bool {
        self.screenshot.is_some() || self.self_test.is_some()
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Painting Colour Assistant".to_owned(),
        window_width: 1400,
        window_height: 880,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

fn load_font() -> Option<Font> {
    let candidates = [
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/Library/Fonts/Arial.ttf",
        "/System/Library/Fonts/Geneva.ttf",
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
    ];
    candidates.iter().find_map(|p| std::fs::read(p).ok().and_then(|b| load_ttf_font_from_bytes(&b).ok()))
}

fn save_screenshot(path: &str) {
    let shot = get_screen_data();
    let (w, h) = (shot.width as u32, shot.height as u32);
    let mut img = image::RgbaImage::from_raw(w, h, shot.bytes).expect("screen buffer");
    // OpenGL hands the framebuffer over bottom row first.
    image::imageops::flip_vertical_in_place(&mut img);
    for p in img.pixels_mut() {
        p.0[3] = 255;
    }
    if let Some(dir) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = img.save(path) {
        eprintln!("could not save screenshot: {e}");
    }
}

async fn run(args: Args) {
    let mut ui = Ui::new(load_font());
    let (store, first_start) = if args.background() { (palette::default_store(), false) } else { palette::load_store() };
    let mut app = App::new(store, !args.background(), first_start);
    if let Some(p) = &args.image {
        app.open_path(p);
        app.screen = app::Screen::Painting;
    } else if args.background() {
        app.set_image("Sample landscape".into(), imgproc::sample_landscape(1200, 800));
    }
    if args.screenshot.is_some() {
        app.screen = app::Screen::Painting;
        app.view = View::Colours;
        app.overlay.kind = Overlay::Fibonacci;
        app.pick_at(vec2(0.3, 0.86));
    }
    if args.palettes {
        app.screen = app::Screen::Palettes;
        app.edit_palette = Some(0);
        app.select_paint(Some(0));
    }
    let mut test = args.self_test.as_ref().map(|_| selftest::SelfTest::new());
    let mut frames = 0u32;
    loop {
        let input = if let Some(t) = &mut test {
            match t.input(&app, &ui) {
                Some(i) => i,
                None => {
                    t.write(args.self_test.as_deref().unwrap());
                    break;
                }
            }
        } else if args.screenshot.is_some() {
            Input { mouse: vec2(-100.0, -100.0), ..Default::default() }
        } else {
            for f in get_dropped_files() {
                if let Some(p) = &f.path {
                    app.open_path(p);
                } else if let Some(b) = &f.bytes {
                    app.open_bytes(b);
                }
                app.screen = app::Screen::Painting;
            }
            Input::from_system()
        };
        ui.begin(input, get_frame_time());
        app.frame(&mut ui);
        frames += 1;
        if let Some(path) = &args.screenshot {
            if (frames >= 120 && app.recipes_done()) || frames >= 900 {
                save_screenshot(path);
                break;
            }
        }
        if test.as_ref().is_some_and(|t| t.frames > 20_000) {
            break;
        }
        next_frame().await;
    }
}

fn main() {
    let args = Args::parse();
    if args.background() {
        // Read by the patched window library: open the window without taking focus.
        unsafe { std::env::set_var("MINIQUAD_NO_ACTIVATE", "1") };
    }
    macroquad::Window::from_config(window_conf(), run(args));
}
