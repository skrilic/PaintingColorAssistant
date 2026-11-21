//! "About" dialog, opened from the app's name in the top bar: the version, and whether a
//! newer one is out (asked from GitHub when the dialog opens).

use super::App;
use crate::ui::{self, BORDER, MUTED, PANEL, TEXT, Ui};
use crate::update::{self, State};
use macroquad::prelude::*;

impl App {
    /// Opens the dialog and starts asking GitHub for the latest version.
    pub(super) fn open_about(&mut self) {
        self.about_open = true;
        self.update.start();
    }

    /// The dialog, over everything else (the rest of the frame is drawn covered).
    /// `was_open`: open before this frame (the click that opened it is not one outside it).
    pub(super) fn about_dialog(&mut self, ui: &mut Ui, was_open: bool) {
        self.update.poll();
        if !self.about_open {
            return;
        }
        let (sw, sh) = (screen_width(), screen_height());
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.5));
        let w = (sw - 24.0).min(420.0);
        let h = 196.0;
        let r = Rect::new(((sw - w) / 2.0).round(), ((sh - h) / 2.0).round(), w, h);
        ui.mark("about_dialog", r);
        ui.hud("about_dialog");
        draw_rectangle(r.x + 3.0, r.y + 4.0, r.w, r.h, Color::new(0.0, 0.0, 0.0, 0.35));
        ui::rrect(r, 8.0, PANEL);
        ui::rrect_lines(r, 8.0, BORDER);

        let x = r.x + 20.0;
        let tw = r.w - 40.0;
        ui.text_fit("PaletteCraft", x, r.y + 36.0, 18.0, TEXT, tw);
        ui.text_fit(&format!("Version {}", update::VERSION), x, r.y + 58.0, 14.0, MUTED, tw);
        let (line, note, colour) = match &self.update.state {
            State::Idle | State::Checking => ("Checking for a newer version...".to_string(), String::new(), MUTED),
            State::Latest => ("You have the latest version.".to_string(), String::new(), ui::GOOD),
            State::Newer(v) => (format!("Version {v} is available."), String::new(), ui::ACCENT),
            State::Failed(e) => ("Could not check for a newer version.".to_string(), e.clone(), ui::WARN),
        };
        ui.text_fit(&line, x, r.y + 98.0, 15.0, colour, tw);
        if !note.is_empty() {
            ui.text_fit(&note, x, r.y + 120.0, 13.0, MUTED, tw);
        }

        let bw = ((tw - 10.0) / 2.0).min(150.0);
        let by = r.y + r.h - 54.0;
        let close = Rect::new(r.x + r.w - 20.0 - bw, by, bw, 34.0);
        let main = Rect::new(close.x - 10.0 - bw, by, bw, 34.0);
        match self.update.state {
            // the web app updates itself: it only needs to load the new files
            State::Newer(_) if ui.button("about_download", main, if cfg!(target_arch = "wasm32") { "Update now" } else { "Download" }) => {
                self.get_update();
            }
            State::Failed(_) if ui.button("about_retry", main, "Try again") => self.update.start(),
            _ => {}
        }
        let outside = was_open && ui.clicked(Rect::new(0.0, 0.0, sw, sh)) && !r.contains(ui.input.mouse);
        if ui.button("about_close", close, "Close") || ui.input.escape || outside {
            self.about_open = false;
        }
    }

    /// "Download" (desktop): the website with the latest downloads; "Update now" (web): reload.
    fn get_update(&mut self) {
        #[cfg(target_arch = "wasm32")]
        crate::web::reload_app();
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.about_open = false;
            match crate::links::open_url(update::DOWNLOAD_URL) {
                Ok(()) => self.say("The download page opens in your browser"),
                Err(e) => self.say(format!("Could not open the browser: {e}. The page is {}", update::DOWNLOAD_URL)),
            }
        }
    }
}
