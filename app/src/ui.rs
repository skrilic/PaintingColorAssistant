//! Small immediate-mode UI on top of macroquad: buttons, sliders, text fields, dropdowns.
//! Every widget records its rectangle by id so the self-test can click it like a person.

use macroquad::prelude::*;
use std::collections::{HashMap, HashSet};

pub const BG: Color = Color::new(0.105, 0.11, 0.125, 1.0);
pub const PANEL: Color = Color::new(0.145, 0.15, 0.17, 1.0);
pub const PANEL2: Color = Color::new(0.185, 0.19, 0.215, 1.0);
pub const PANEL3: Color = Color::new(0.235, 0.24, 0.27, 1.0);
pub const BORDER: Color = Color::new(0.30, 0.31, 0.35, 1.0);
pub const TEXT: Color = Color::new(0.92, 0.92, 0.94, 1.0);
pub const MUTED: Color = Color::new(0.62, 0.63, 0.68, 1.0);
pub const ACCENT: Color = Color::new(0.93, 0.62, 0.27, 1.0);
pub const ACCENT_DIM: Color = Color::new(0.42, 0.30, 0.17, 1.0);
pub const GOOD: Color = Color::new(0.45, 0.80, 0.52, 1.0);
pub const WARN: Color = Color::new(0.95, 0.72, 0.35, 1.0);
pub const BAD: Color = Color::new(0.93, 0.45, 0.42, 1.0);

#[derive(Default, Clone, Debug)]
pub struct Input {
    pub mouse: Vec2,
    pub down: bool,
    pub pressed: bool,
    pub wheel: f32,
    pub chars: Vec<char>,
    pub backspace: bool,
    pub enter: bool,
    pub escape: bool,
}

impl Input {
    pub fn from_system() -> Self {
        let (mx, my) = mouse_position();
        let mut chars = Vec::new();
        while let Some(c) = get_char_pressed() {
            if !c.is_control() {
                chars.push(c);
            }
        }
        let (_, wy) = mouse_wheel();
        Input {
            mouse: vec2(mx, my),
            down: is_mouse_button_down(MouseButton::Left),
            pressed: is_mouse_button_pressed(MouseButton::Left),
            wheel: wy.clamp(-3.0, 3.0),
            chars,
            backspace: is_key_pressed(KeyCode::Backspace),
            enter: is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter),
            escape: is_key_pressed(KeyCode::Escape),
        }
    }
}

pub struct Popup {
    pub id: String,
    pub anchor: Rect,
    pub items: Vec<String>,
}

pub struct Ui {
    pub input: Input,
    pub font: Option<Font>,
    pub active: Option<String>,
    pub focus: Option<String>,
    pub rects: HashMap<String, Rect>,
    pub prev_rects: HashMap<String, Rect>,
    /// Names of on-screen information drawn this frame (read by the self-test).
    pub hud: HashSet<String>,
    pub prev_hud: HashSet<String>,
    popup: Option<Popup>,
    popup_result: Option<(String, usize)>,
    swallow: bool,
    block: Option<Rect>,
    pub time: f32,
}

impl Ui {
    pub fn new(font: Option<Font>) -> Self {
        Ui {
            input: Input::default(),
            font,
            active: None,
            focus: None,
            rects: HashMap::new(),
            prev_rects: HashMap::new(),
            hud: HashSet::new(),
            prev_hud: HashSet::new(),
            popup: None,
            popup_result: None,
            swallow: false,
            block: None,
            time: 0.0,
        }
    }

    pub fn begin(&mut self, input: Input, dt: f32) {
        self.input = input;
        self.time += dt;
        self.prev_rects = std::mem::take(&mut self.rects);
        self.prev_hud = std::mem::take(&mut self.hud);
        self.block = self.popup.as_ref().map(|p| popup_rect(p));
        // While a popup is open, a press goes to the popup only.
        self.swallow = self.popup.is_some() && self.input.pressed;
        if !self.input.down && !self.input.pressed {
            self.active = None;
        }
    }

    pub fn end(&mut self) {
        if let Some(p) = &self.popup {
            let r = popup_rect(p);
            draw_rectangle(r.x + 3.0, r.y + 4.0, r.w, r.h, Color::new(0.0, 0.0, 0.0, 0.35));
            draw_rectangle(r.x, r.y, r.w, r.h, PANEL2);
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, BORDER);
            let mut chosen = None;
            for (i, item) in p.items.iter().enumerate() {
                let ir = Rect::new(r.x, r.y + 4.0 + i as f32 * 28.0, r.w, 28.0);
                self.rects.insert(format!("{}#{}", p.id, i), ir);
                if ir.contains(self.input.mouse) {
                    draw_rectangle(ir.x + 2.0, ir.y, ir.w - 4.0, ir.h, PANEL3);
                    if self.input.pressed {
                        chosen = Some(i);
                    }
                }
                self.text(item, ir.x + 10.0, ir.y + 19.0, 15.0, TEXT);
            }
            if self.input.pressed {
                if let Some(i) = chosen {
                    self.popup_result = Some((p.id.clone(), i));
                }
                self.popup = None;
            }
        }
    }

    pub fn mark(&mut self, id: &str, r: Rect) {
        self.rects.insert(id.to_string(), r);
    }

    pub fn hud(&mut self, name: &str) {
        self.hud.insert(name.to_string());
    }

    /// Mouse is over `r` and not covered by a popup.
    pub fn hovered(&self, r: Rect) -> bool {
        r.contains(self.input.mouse) && !self.block.is_some_and(|b| b.contains(self.input.mouse))
    }

    pub fn clicked(&self, r: Rect) -> bool {
        !self.swallow && self.input.pressed && self.hovered(r)
    }

    // ---------- drawing helpers ----------

    pub fn text(&self, s: &str, x: f32, y: f32, size: f32, color: Color) {
        draw_text_ex(
            s,
            x.round(),
            y.round(),
            TextParams { font: self.font.as_ref(), font_size: size as u16, color, ..Default::default() },
        );
    }

    pub fn measure(&self, s: &str, size: f32) -> f32 {
        measure_text(s, self.font.as_ref(), size as u16, 1.0).width
    }

    pub fn text_centered(&self, s: &str, r: Rect, size: f32, color: Color) {
        let w = self.measure(s, size);
        self.text(s, r.x + (r.w - w) / 2.0, r.y + r.h / 2.0 + size * 0.35, size, color);
    }

    /// Text cut with "..." to fit `max_w`.
    pub fn text_fit(&self, s: &str, x: f32, y: f32, size: f32, color: Color, max_w: f32) {
        if self.measure(s, size) <= max_w {
            self.text(s, x, y, size, color);
            return;
        }
        let mut t: String = s.to_string();
        while !t.is_empty() && self.measure(&format!("{t}..."), size) > max_w {
            t.pop();
        }
        self.text(&format!("{t}..."), x, y, size, color);
    }

    pub fn header(&self, s: &str, x: f32, y: f32) {
        self.text(s, x, y, 13.0, MUTED);
    }

    // ---------- widgets ----------

    pub fn button(&mut self, id: &str, r: Rect, label: &str) -> bool {
        self.button_ex(id, r, label, false)
    }

    pub fn button_ex(&mut self, id: &str, r: Rect, label: &str, selected: bool) -> bool {
        self.mark(id, r);
        let hov = self.hovered(r);
        let bg = if selected {
            ACCENT_DIM
        } else if hov {
            PANEL3
        } else {
            PANEL2
        };
        rrect(r, 5.0, bg);
        if selected {
            rrect_lines(r, 5.0, ACCENT);
        }
        self.text_centered(label, r, 15.0, if selected { Color::new(1.0, 0.86, 0.66, 1.0) } else { TEXT });
        self.clicked(r)
    }

    /// Row of mutually exclusive options; returns the newly picked index.
    pub fn segmented(&mut self, id: &str, r: Rect, labels: &[&str], selected: usize) -> Option<usize> {
        let w = r.w / labels.len() as f32;
        let mut out = None;
        for (i, l) in labels.iter().enumerate() {
            let br = Rect::new(r.x + i as f32 * w, r.y, w - 3.0, r.h);
            if self.button_ex(&format!("{id}_{i}"), br, l, i == selected) {
                out = Some(i);
            }
        }
        out
    }

    pub fn checkbox(&mut self, id: &str, r: Rect, label: &str, value: &mut bool) -> bool {
        self.mark(id, r);
        let b = Rect::new(r.x, r.y + (r.h - 16.0) / 2.0, 16.0, 16.0);
        rrect(b, 3.0, if *value { ACCENT } else { PANEL3 });
        if *value {
            draw_line(b.x + 3.5, b.y + 8.5, b.x + 7.0, b.y + 12.0, 2.0, BG);
            draw_line(b.x + 7.0, b.y + 12.0, b.x + 13.0, b.y + 4.0, 2.0, BG);
        }
        self.text(label, r.x + 24.0, r.y + r.h / 2.0 + 5.0, 15.0, TEXT);
        if self.clicked(r) {
            *value = !*value;
            return true;
        }
        false
    }

    pub fn slider_int(&mut self, id: &str, r: Rect, value: &mut i32, min: i32, max: i32) -> bool {
        let mut f = *value as f32;
        let old = *value;
        self.slider_core(id, r, &mut f, min as f32, max as f32, (max - min) as usize);
        *value = (f.round() as i32).clamp(min, max);
        *value != old
    }

    pub fn slider_f(&mut self, id: &str, r: Rect, value: &mut f32, min: f32, max: f32) -> bool {
        let old = *value;
        self.slider_core(id, r, value, min, max, 0);
        (*value - old).abs() > f32::EPSILON
    }

    fn slider_core(&mut self, id: &str, r: Rect, value: &mut f32, min: f32, max: f32, ticks: usize) {
        self.mark(id, r);
        let track = Rect::new(r.x + 8.0, r.y + r.h / 2.0 - 2.0, r.w - 16.0, 4.0);
        if self.clicked(r) {
            self.active = Some(id.to_string());
        }
        if self.active.as_deref() == Some(id) && (self.input.down || self.input.pressed) {
            let t = ((self.input.mouse.x - track.x) / track.w).clamp(0.0, 1.0);
            *value = min + t * (max - min);
        }
        let t = ((*value - min) / (max - min)).clamp(0.0, 1.0);
        rrect(track, 2.0, PANEL3);
        rrect(Rect::new(track.x, track.y, track.w * t, track.h), 2.0, ACCENT);
        if ticks > 0 && ticks <= 30 {
            for i in 0..=ticks {
                let x = track.x + track.w * i as f32 / ticks as f32;
                draw_line(x, track.y + 7.0, x, track.y + 10.0, 1.0, BORDER);
            }
        }
        let hx = track.x + track.w * t;
        let hov = self.hovered(r) || self.active.as_deref() == Some(id);
        draw_circle(hx, track.y + 2.0, if hov { 8.0 } else { 7.0 }, TEXT);
        draw_circle(hx, track.y + 2.0, 4.0, ACCENT);
    }

    pub fn text_field(&mut self, id: &str, r: Rect, value: &mut String, placeholder: &str) -> bool {
        self.mark(id, r);
        let focused = self.focus.as_deref() == Some(id);
        let mut changed = false;
        if self.clicked(r) {
            self.focus = Some(id.to_string());
        } else if focused && self.input.pressed && !self.hovered(r) {
            self.focus = None;
        }
        let focused = self.focus.as_deref() == Some(id);
        if focused {
            for c in self.input.chars.clone() {
                if value.chars().count() < 60 {
                    value.push(c);
                    changed = true;
                }
            }
            if self.input.backspace && value.pop().is_some() {
                changed = true;
            }
            if self.input.enter || self.input.escape {
                self.focus = None;
            }
        }
        rrect(r, 4.0, if focused { BG } else { PANEL2 });
        rrect_lines(r, 4.0, if focused { ACCENT } else { BORDER });
        let ty = r.y + r.h / 2.0 + 5.0;
        if value.is_empty() && !focused {
            self.text(placeholder, r.x + 8.0, ty, 15.0, MUTED);
        } else {
            self.text_fit(value, r.x + 8.0, ty, 15.0, TEXT, r.w - 18.0);
        }
        if focused && (self.time * 2.0).fract() < 0.6 {
            let w = self.measure(value, 15.0).min(r.w - 18.0);
            draw_line(r.x + 9.0 + w, r.y + 6.0, r.x + 9.0 + w, r.y + r.h - 6.0, 1.5, ACCENT);
        }
        changed
    }

    /// Button that opens a list; returns the picked index once chosen.
    pub fn dropdown(&mut self, id: &str, r: Rect, label: &str, items: &[String]) -> Option<usize> {
        self.mark(id, r);
        let hov = self.hovered(r);
        rrect(r, 5.0, if hov { PANEL3 } else { PANEL2 });
        rrect_lines(r, 5.0, BORDER);
        self.text_fit(label, r.x + 10.0, r.y + r.h / 2.0 + 5.0, 15.0, TEXT, r.w - 34.0);
        let cx = r.x + r.w - 16.0;
        let cy = r.y + r.h / 2.0;
        draw_triangle(vec2(cx - 5.0, cy - 2.5), vec2(cx + 5.0, cy - 2.5), vec2(cx, cy + 3.5), MUTED);
        if self.clicked(r) && !items.is_empty() {
            self.popup = Some(Popup { id: id.to_string(), anchor: r, items: items.to_vec() });
            // Do not let this same press close the popup again in end().
            self.swallow = true;
            self.input.pressed = false;
        }
        if let Some((pid, i)) = &self.popup_result {
            if pid == id {
                let i = *i;
                self.popup_result = None;
                return Some(i);
            }
        }
        None
    }
}

fn popup_rect(p: &Popup) -> Rect {
    let h = p.items.len() as f32 * 28.0 + 8.0;
    let y = if p.anchor.y + p.anchor.h + h > screen_height() - 4.0 {
        (p.anchor.y - h).max(4.0)
    } else {
        p.anchor.y + p.anchor.h + 2.0
    };
    Rect::new(p.anchor.x, y, p.anchor.w.max(180.0), h)
}

pub fn rrect(r: Rect, rad: f32, c: Color) {
    let rad = rad.min(r.w / 2.0).min(r.h / 2.0);
    if rad < 0.5 {
        draw_rectangle(r.x, r.y, r.w, r.h, c);
        return;
    }
    draw_rectangle(r.x + rad, r.y, r.w - 2.0 * rad, r.h, c);
    draw_rectangle(r.x, r.y + rad, rad, r.h - 2.0 * rad, c);
    draw_rectangle(r.x + r.w - rad, r.y + rad, rad, r.h - 2.0 * rad, c);
    for (x, y) in [
        (r.x + rad, r.y + rad),
        (r.x + r.w - rad, r.y + rad),
        (r.x + rad, r.y + r.h - rad),
        (r.x + r.w - rad, r.y + r.h - rad),
    ] {
        draw_circle(x, y, rad, c);
    }
}

pub fn rrect_lines(r: Rect, rad: f32, c: Color) {
    let rad = rad.min(r.w / 2.0).min(r.h / 2.0);
    draw_line(r.x + rad, r.y + 0.5, r.x + r.w - rad, r.y + 0.5, 1.0, c);
    draw_line(r.x + rad, r.y + r.h - 0.5, r.x + r.w - rad, r.y + r.h - 0.5, 1.0, c);
    draw_line(r.x + 0.5, r.y + rad, r.x + 0.5, r.y + r.h - rad, 1.0, c);
    draw_line(r.x + r.w - 0.5, r.y + rad, r.x + r.w - 0.5, r.y + r.h - rad, 1.0, c);
    let seg = 6;
    for (cx, cy, a0) in [
        (r.x + rad, r.y + rad, std::f32::consts::PI),
        (r.x + r.w - rad, r.y + rad, 1.5 * std::f32::consts::PI),
        (r.x + r.w - rad, r.y + r.h - rad, 0.0),
        (r.x + rad, r.y + r.h - rad, 0.5 * std::f32::consts::PI),
    ] {
        for i in 0..seg {
            let a = a0 + std::f32::consts::FRAC_PI_2 * i as f32 / seg as f32;
            let b = a0 + std::f32::consts::FRAC_PI_2 * (i + 1) as f32 / seg as f32;
            let r0 = rad - 0.5;
            draw_line(cx + r0 * a.cos(), cy + r0 * a.sin(), cx + r0 * b.cos(), cy + r0 * b.sin(), 1.0, c);
        }
    }
}

pub fn col(c: [u8; 3]) -> Color {
    Color::from_rgba(c[0], c[1], c[2], 255)
}

/// Colour swatch with a thin outline.
pub fn swatch(r: Rect, c: [u8; 3]) {
    rrect(r, 4.0, col(c));
    rrect_lines(r, 4.0, Color::new(1.0, 1.0, 1.0, 0.18));
}
