//! Minimal PDF writer: pages with JPEG images, filled shapes and Helvetica text.
//! Coordinates are in points with the origin at the top-left of the page.

use image::RgbImage;
use std::fmt::Write as _;

/// A4 portrait in points.
pub const A4_W: f32 = 595.28;
pub const A4_H: f32 = 841.89;
/// JPEG quality of embedded pictures.
pub const JPEG_QUALITY: u8 = 90;

/// Helvetica advance widths (1/1000 em) for ASCII 32..=126.
const HELVETICA: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556, 556, 556, 556, 556, 556,
    556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778,
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556,
    556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
];
/// Helvetica-Bold advance widths (1/1000 em) for ASCII 32..=126.
const HELVETICA_BOLD: [u16; 95] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556, 556, 556, 556, 556, 556,
    556, 556, 333, 333, 584, 584, 584, 611, 975, 722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611, 833, 722, 778,
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 333, 278, 333, 584, 556, 333, 556, 611, 556, 611, 556, 333, 611,
    611, 278, 278, 556, 278, 889, 611, 611, 611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389, 280, 389, 584,
];

struct Img {
    w: u32,
    h: u32,
    jpeg: Vec<u8>,
}

pub struct Doc {
    pub w: f32,
    pub h: f32,
    pages: Vec<String>,
    images: Vec<Img>,
}

/// Character to a WinAnsi byte; letters outside the set lose their accent.
fn win_ansi(c: char) -> u8 {
    match c {
        ' '..='~' => c as u8,
        '\u{a0}'..='\u{ff}' => c as u32 as u8,
        'Š' => 0x8A,
        'š' => 0x9A,
        'Ž' => 0x8E,
        'ž' => 0x9E,
        'Œ' => 0x8C,
        'œ' => 0x9C,
        '–' => 0x96,
        '—' => 0x97,
        '•' => 0x95,
        '‘' => 0x91,
        '’' => 0x92,
        '“' => 0x93,
        '”' => 0x94,
        '…' => 0x85,
        '€' => 0x80,
        'Č' | 'Ć' => b'C',
        'č' | 'ć' => b'c',
        'Đ' => 0xD0,
        'đ' => b'd',
        _ => b'?',
    }
}

fn rgb(c: [u8; 3]) -> String {
    format!("{:.3} {:.3} {:.3}", c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0)
}

pub fn text_width(s: &str, size: f32, bold: bool) -> f32 {
    let table = if bold { &HELVETICA_BOLD } else { &HELVETICA };
    let units: u32 = s
        .chars()
        .map(|c| match c {
            ' '..='~' => table[c as usize - 32] as u32,
            _ => 556,
        })
        .sum();
    units as f32 * size / 1000.0
}

/// Shorten `s` with "..." so it fits `max_w`.
pub fn fit(s: &str, size: f32, bold: bool, max_w: f32) -> String {
    if text_width(s, size, bold) <= max_w {
        return s.to_string();
    }
    let mut t: String = s.to_string();
    while !t.is_empty() && text_width(&format!("{t}..."), size, bold) > max_w {
        t.pop();
    }
    format!("{t}...")
}

/// Split text into lines no wider than `max_w`.
pub fn wrap(s: &str, size: f32, bold: bool, max_w: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        let next = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
        if text_width(&next, size, bold) > max_w && !cur.is_empty() {
            lines.push(std::mem::take(&mut cur));
            cur = word.to_string();
        } else {
            cur = next;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

impl Doc {
    pub fn new(w: f32, h: f32) -> Self {
        Doc { w, h, pages: Vec::new(), images: Vec::new() }
    }

    pub fn new_page(&mut self) {
        self.pages.push(String::new());
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    fn ops(&mut self) -> &mut String {
        if self.pages.is_empty() {
            self.pages.push(String::new());
        }
        self.pages.last_mut().unwrap()
    }

    /// Embed a picture (as JPEG); returns its handle.
    pub fn add_image(&mut self, img: &RgbImage) -> usize {
        let mut jpeg = Vec::new();
        let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, JPEG_QUALITY);
        img.write_with_encoder(enc).expect("jpeg encoding");
        self.images.push(Img { w: img.width(), h: img.height(), jpeg });
        self.images.len() - 1
    }

    pub fn image(&mut self, handle: usize, x: f32, y: f32, w: f32, h: f32) {
        let py = self.h - y - h;
        let _ = write!(self.ops(), "q {w:.2} 0 0 {h:.2} {x:.2} {py:.2} cm /Im{handle} Do Q\n");
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, fill: [u8; 3]) {
        let py = self.h - y - h;
        let c = rgb(fill);
        let _ = write!(self.ops(), "{c} rg {x:.2} {py:.2} {w:.2} {h:.2} re f\n");
    }

    pub fn rect_stroke(&mut self, x: f32, y: f32, w: f32, h: f32, color: [u8; 3], lw: f32) {
        let py = self.h - y - h;
        let c = rgb(color);
        let _ = write!(self.ops(), "{c} RG {lw:.2} w {x:.2} {py:.2} {w:.2} {h:.2} re S\n");
    }

    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: [u8; 3], lw: f32) {
        let (a, b) = (self.h - y0, self.h - y1);
        let c = rgb(color);
        let _ = write!(self.ops(), "{c} RG {lw:.2} w {x0:.2} {a:.2} m {x1:.2} {b:.2} l S\n");
    }

    fn circle_path(&mut self, cx: f32, cy: f32, r: f32) {
        let cy = self.h - cy;
        let k = 0.552_284_8 * r;
        let _ = write!(
            self.ops(),
            "{:.2} {:.2} m {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c \
             {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c ",
            cx + r, cy,
            cx + r, cy + k, cx + k, cy + r, cx, cy + r,
            cx - k, cy + r, cx - r, cy + k, cx - r, cy,
            cx - r, cy - k, cx - k, cy - r, cx, cy - r,
            cx + k, cy - r, cx + r, cy - k, cx + r, cy
        );
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, fill: [u8; 3]) {
        let c = rgb(fill);
        let _ = write!(self.ops(), "{c} rg ");
        self.circle_path(cx, cy, r);
        self.ops().push_str("f\n");
    }

    pub fn circle_stroke(&mut self, cx: f32, cy: f32, r: f32, color: [u8; 3], lw: f32) {
        let c = rgb(color);
        let _ = write!(self.ops(), "{c} RG {lw:.2} w ");
        self.circle_path(cx, cy, r);
        self.ops().push_str("S\n");
    }

    /// Text with its baseline at `y`.
    pub fn text(&mut self, x: f32, y: f32, size: f32, bold: bool, color: [u8; 3], s: &str) {
        let mut lit = String::with_capacity(s.len() + 2);
        for c in s.chars() {
            match win_ansi(c) {
                b'(' => lit.push_str("\\("),
                b')' => lit.push_str("\\)"),
                b'\\' => lit.push_str("\\\\"),
                b if b < 0x80 => lit.push(b as char),
                b => {
                    let _ = write!(lit, "\\{b:03o}");
                }
            }
        }
        let py = self.h - y;
        let font = if bold { "F2" } else { "F1" };
        let c = rgb(color);
        let _ = write!(self.ops(), "BT {c} rg /{font} {size:.1} Tf {x:.2} {py:.2} Td ({lit}) Tj ET\n");
    }

    pub fn text_right(&mut self, right: f32, y: f32, size: f32, bold: bool, color: [u8; 3], s: &str) {
        let w = text_width(s, size, bold);
        self.text(right - w, y, size, bold, color, s);
    }

    /// The finished file.
    pub fn finish(self) -> Vec<u8> {
        // Object numbers: 1 catalog, 2 page tree, 3-4 fonts, then images, then page + content pairs.
        let first_img = 5;
        let first_page = first_img + self.images.len();
        let page_ids: Vec<usize> = (0..self.pages.len()).map(|i| first_page + 2 * i).collect();
        let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
        let mut offsets = Vec::new();
        let obj = |out: &mut Vec<u8>, offsets: &mut Vec<usize>, body: &[u8]| {
            offsets.push(out.len());
            let id = offsets.len();
            out.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
            out.extend_from_slice(body);
            out.extend_from_slice(b"\nendobj\n");
        };
        obj(&mut out, &mut offsets, b"<< /Type /Catalog /Pages 2 0 R >>");
        let kids: Vec<String> = page_ids.iter().map(|id| format!("{id} 0 R")).collect();
        obj(&mut out, &mut offsets, format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), page_ids.len()).as_bytes());
        obj(&mut out, &mut offsets, b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>");
        obj(&mut out, &mut offsets, b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>");
        for img in &self.images {
            let mut body = format!(
                "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n",
                img.w,
                img.h,
                img.jpeg.len()
            )
            .into_bytes();
            body.extend_from_slice(&img.jpeg);
            body.extend_from_slice(b"\nendstream");
            obj(&mut out, &mut offsets, &body);
        }
        let xobjects: String = (0..self.images.len()).map(|i| format!("/Im{i} {} 0 R ", first_img + i)).collect();
        for (i, ops) in self.pages.iter().enumerate() {
            let content_id = page_ids[i] + 1;
            let page = format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> /XObject << {xobjects}>> >> /Contents {content_id} 0 R >>",
                self.w, self.h
            );
            obj(&mut out, &mut offsets, page.as_bytes());
            let mut body = format!("<< /Length {} >>\nstream\n", ops.len()).into_bytes();
            body.extend_from_slice(ops.as_bytes());
            body.extend_from_slice(b"\nendstream");
            obj(&mut out, &mut offsets, &body);
        }
        let xref = out.len();
        let mut tail = format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1);
        for o in &offsets {
            let _ = writeln!(tail, "{o:010} 00000 n ");
        }
        let _ = write!(tail, "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", offsets.len() + 1);
        out.extend_from_slice(tail.as_bytes());
        out
    }
}
