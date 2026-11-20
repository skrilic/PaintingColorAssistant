//! Composition overlays drawn over the picture: rule of thirds, golden ratio grid,
//! Fibonacci spiral, square grid.

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Overlay {
    None,
    Thirds,
    Fibonacci,
    Grid,
    /// Phi grid: lines at 1/phi^2 and 1/phi of the width and height.
    Golden,
}

impl Overlay {
    /// Order of the overlay buttons.
    pub const ALL: [Overlay; 5] = [Overlay::None, Overlay::Thirds, Overlay::Golden, Overlay::Fibonacci, Overlay::Grid];
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct OverlaySettings {
    pub kind: Overlay,
    pub cells: i32,
    pub diagonals: bool,
    pub flip_x: bool,
    pub flip_y: bool,
    pub colour: usize,
    pub opacity: f32,
}

pub const LINE_COLOURS: [(&str, Color); 3] = [
    ("White", Color::new(1.0, 1.0, 1.0, 1.0)),
    ("Black", Color::new(0.0, 0.0, 0.0, 1.0)),
    ("Red", Color::new(0.95, 0.2, 0.2, 1.0)),
];

/// The golden ratio used to cut the Fibonacci rectangles.
pub const PHI: f32 = 1.618_034;

/// Draws the overlay inside `r`; returns how many line segments were drawn.
pub fn draw_overlay(r: Rect, s: &OverlaySettings, label: &dyn Fn(&str, f32, f32, Color)) -> usize {
    let mut c = LINE_COLOURS[s.colour.min(2)].1;
    c.a = s.opacity;
    let mut faint = c;
    faint.a = s.opacity * 0.55;
    let mut n = 0;
    let mut line = |x0: f32, y0: f32, x1: f32, y1: f32, w: f32, col: Color| {
        draw_line(x0, y0, x1, y1, w, col);
        n += 1;
    };
    match s.kind {
        Overlay::None => {}
        Overlay::Thirds => {
            for i in 1..3 {
                let x = r.x + r.w * i as f32 / 3.0;
                let y = r.y + r.h * i as f32 / 3.0;
                line(x, r.y, x, r.y + r.h, 1.5, c);
                line(r.x, y, r.x + r.w, y, 1.5, c);
            }
            for i in 1..3 {
                for j in 1..3 {
                    let p = vec2(r.x + r.w * i as f32 / 3.0, r.y + r.h * j as f32 / 3.0);
                    draw_circle_lines(p.x, p.y, 7.0, 1.5, c);
                }
            }
        }
        Overlay::Golden => {
            // 0.382 and 0.618: the picture split so the larger part relates to the smaller as the whole to the larger
            let cuts = [1.0 - 1.0 / PHI, 1.0 / PHI];
            for t in cuts {
                let x = r.x + r.w * t;
                let y = r.y + r.h * t;
                line(x, r.y, x, r.y + r.h, 1.5, c);
                line(r.x, y, r.x + r.w, y, 1.5, c);
            }
            for tx in cuts {
                for ty in cuts {
                    draw_circle_lines(r.x + r.w * tx, r.y + r.h * ty, 7.0, 1.5, c);
                }
            }
        }
        Overlay::Grid => {
            let cells = s.cells.max(2) as f32;
            // Square cells: the longer side gets `cells` divisions.
            let size = r.w.max(r.h) / cells;
            let cols = (r.w / size - 0.01).ceil() as i32;
            let rows = (r.h / size - 0.01).ceil() as i32;
            for i in 1..cols {
                let x = r.x + i as f32 * size;
                line(x, r.y, x, r.y + r.h, 1.0, c);
            }
            for j in 1..rows {
                let y = r.y + j as f32 * size;
                line(r.x, y, r.x + r.w, y, 1.0, c);
            }
            if s.diagonals {
                for i in 0..cols {
                    for j in 0..rows {
                        let x0 = r.x + i as f32 * size;
                        let y0 = r.y + j as f32 * size;
                        let x1 = (x0 + size).min(r.x + r.w);
                        let y1 = (y0 + size).min(r.y + r.h);
                        line(x0, y0, x1, y1, 1.0, faint);
                        line(x1, y0, x0, y1, 1.0, faint);
                    }
                }
            }
            // Cell labels (A, B, C... across, 1, 2, 3... down) to help transfer the drawing.
            if size > 18.0 {
                for i in 0..cols {
                    let ch = (b'A' + (i as u8 % 26)) as char;
                    label(&ch.to_string(), r.x + i as f32 * size + 4.0, r.y + 13.0, c);
                }
                for j in 1..rows {
                    label(&(j + 1).to_string(), r.x + 4.0, r.y + j as f32 * size + 13.0, c);
                }
            }
        }
        Overlay::Fibonacci => {
            let mut rect = (0.0f32, 0.0f32, 1.0f32, 1.0f32); // normalised x, y, w, h
            // Portrait pictures start with a horizontal cut so the first piece is the largest.
            let start = if r.h > r.w { 1 } else { 0 };
            let map = |x: f32, y: f32| -> Vec2 {
                let x = if s.flip_x { 1.0 - x } else { x };
                let y = if s.flip_y { 1.0 - y } else { y };
                vec2(r.x + x * r.w, r.y + y * r.h)
            };
            for step in 0..12 {
                let (x, y, w, h) = rect;
                let dir = (step + start) % 4;
                let cut = 1.0 / PHI;
                // piece: the "square" cut off; arc: centre and the two radii in normalised units
                let (piece, rest, centre, a0) = match dir {
                    0 => {
                        let pw = w * cut;
                        ((x, y, pw, h), (x + pw, y, w - pw, h), (x + pw, y + h), 180.0f32)
                    }
                    1 => {
                        let ph = h * cut;
                        ((x, y, w, ph), (x, y + ph, w, h - ph), (x, y + ph), 270.0)
                    }
                    2 => {
                        let pw = w * cut;
                        ((x + w - pw, y, pw, h), (x, y, w - pw, h), (x + w - pw, y), 0.0)
                    }
                    _ => {
                        let ph = h * cut;
                        ((x, y + h - ph, w, ph), (x, y, w, h - ph), (x + w, y + h - ph), 90.0)
                    }
                };
                let (px, py, pw, ph) = piece;
                // rectangle outline of the piece
                let p0 = map(px, py);
                let p1 = map(px + pw, py + ph);
                let (lx, ly) = (p0.x.min(p1.x), p0.y.min(p1.y));
                let (hx, hy) = (p0.x.max(p1.x), p0.y.max(p1.y));
                line(lx, ly, hx, ly, 1.0, faint);
                line(lx, hy, hx, hy, 1.0, faint);
                line(lx, ly, lx, hy, 1.0, faint);
                line(hx, ly, hx, hy, 1.0, faint);
                // quarter ellipse arc
                let segs = 24;
                let mut prev = None;
                for k in 0..=segs {
                    let a = (a0 + 90.0 * k as f32 / segs as f32).to_radians();
                    let q = map(centre.0 + pw * a.cos(), centre.1 + ph * a.sin());
                    if let Some(p) = prev {
                        let p: Vec2 = p;
                        line(p.x, p.y, q.x, q.y, 2.2, c);
                    }
                    prev = Some(q);
                }
                rect = rest;
                if (rect.2 * r.w).min(rect.3 * r.h) < 2.0 {
                    break;
                }
            }
        }
    }
    n
}
