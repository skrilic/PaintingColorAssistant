//! Colour space helpers: sRGB <-> linear RGB, CIE L*a*b* and colour difference.

use std::sync::OnceLock;

pub fn srgb_to_lin(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn lin_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn lin_lut() -> &'static [f32; 256] {
    static LUT: OnceLock<[f32; 256]> = OnceLock::new();
    LUT.get_or_init(|| {
        let mut t = [0.0; 256];
        for (i, v) in t.iter_mut().enumerate() {
            *v = srgb_to_lin(i as f32 / 255.0);
        }
        t
    })
}

pub fn srgb8_to_lin(c: [u8; 3]) -> [f32; 3] {
    let l = lin_lut();
    [l[c[0] as usize], l[c[1] as usize], l[c[2] as usize]]
}

pub fn lin_to_srgb8(c: [f32; 3]) -> [u8; 3] {
    [
        (lin_to_srgb(c[0]) * 255.0 + 0.5) as u8,
        (lin_to_srgb(c[1]) * 255.0 + 0.5) as u8,
        (lin_to_srgb(c[2]) * 255.0 + 0.5) as u8,
    ]
}

#[inline]
fn lab_f(t: f32) -> f32 {
    if t > 0.008_856 {
        t.cbrt()
    } else {
        7.787 * t + 16.0 / 116.0
    }
}

/// Linear sRGB (D65) to CIE L*a*b*.
pub fn lin_to_lab(c: [f32; 3]) -> [f32; 3] {
    let x = (0.412_456_4 * c[0] + 0.357_576_1 * c[1] + 0.180_437_5 * c[2]) / 0.950_47;
    let y = 0.212_672_9 * c[0] + 0.715_152_2 * c[1] + 0.072_175 * c[2];
    let z = (0.019_333_9 * c[0] + 0.119_192 * c[1] + 0.950_304_1 * c[2]) / 1.088_83;
    let (fx, fy, fz) = (lab_f(x), lab_f(y), lab_f(z));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

pub fn srgb8_to_lab(c: [u8; 3]) -> [f32; 3] {
    lin_to_lab(srgb8_to_lin(c))
}

/// Neutral grey with the given lightness L* (0..100).
pub fn grey_for_lightness(l: f32) -> u8 {
    let fy = (l + 16.0) / 116.0;
    let y = if l > 8.0 { fy * fy * fy } else { l / 903.3 };
    (lin_to_srgb(y) * 255.0 + 0.5) as u8
}

/// CIE76 colour difference.
pub fn delta_e(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d0 = a[0] - b[0];
    let d1 = a[1] - b[1];
    let d2 = a[2] - b[2];
    (d0 * d0 + d1 * d1 + d2 * d2).sqrt()
}

/// Hue angle in degrees (0..360) of a Lab colour.
pub fn hue_deg(lab: [f32; 3]) -> f32 {
    let h = lab[2].atan2(lab[1]).to_degrees();
    if h < 0.0 { h + 360.0 } else { h }
}

pub fn to_hex(c: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

pub fn from_hex(s: &str) -> Option<[u8; 3]> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(s, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}
