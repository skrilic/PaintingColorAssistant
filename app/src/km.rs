//! Kubelka-Munk paint mixing and recipe search.
//!
//! Each paint gets an absorption K and scattering S per RGB channel (linear light).
//! Mixtures add K and S by concentration (two-constant Kubelka-Munk).
//! * Watercolor: the mixed film of thickness `load` sits on white paper, so
//!   adding water (lower load) lets the paper show through.
//! * Acrylic / oil: an opaque body (infinite thickness, R = R_inf(K/S)); lighter
//!   values come from adding white paint.
//!
//! Transparency sets the scattering power, tinting strength sets how much a
//! part of the paint counts in a mixture.

use crate::colour::{delta_e, lin_to_lab, lin_to_srgb8, srgb8_to_lin};
use crate::palette::{Medium, Paint, Palette, Transparency};

/// Watercolor paper colour (sRGB).
pub const PAPER: [u8; 3] = [247, 244, 236];
/// Default white used for acrylic/oil when the palette has none (Titanium White PW6).
pub const DEFAULT_WHITE: [u8; 3] = [246, 246, 242];
/// Water dilution steps: fraction of full pigment load.
pub const WATER_LOADS: [f32; 12] = [1.0, 0.75, 0.55, 0.4, 0.3, 0.22, 0.16, 0.11, 0.075, 0.05, 0.033, 0.02];
/// White parts tried for acrylic/oil.
pub const WHITE_PARTS: [u32; 11] = [0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 40];
/// Ratio grid: pairs use 1..=PAIR_MAX parts each, triples 1..=TRIPLE_MAX.
pub const PAIR_MAX: u32 = 6;
pub const TRIPLE_MAX: u32 = 4;
/// Score penalty (in delta E) per extra paint, so simpler mixes win ties.
pub const EXTRA_PAINT_PENALTY: f32 = 0.8;

pub fn scatter_for(t: Transparency) -> f32 {
    match t {
        Transparency::Transparent => 0.12,
        Transparency::HalfTransparent => 0.45,
        Transparency::Opaque => 1.6,
    }
}

#[inline]
pub fn ks_from_r(r: f32) -> f32 {
    let r = r.clamp(0.001, 0.999);
    (1.0 - r) * (1.0 - r) / (2.0 * r)
}

#[inline]
pub fn r_inf(ks: f32) -> f32 {
    1.0 + ks - (ks * ks + 2.0 * ks).sqrt()
}

/// Reflectance of a layer (absorption k, scattering s, thickness x) over a ground of reflectance rg.
#[inline]
pub fn r_layer(k: f32, s: f32, x: f32, rg: f32) -> f32 {
    let sx = s * x;
    if sx < 1e-5 {
        return rg * (-2.0 * k * x).exp();
    }
    if k < 1e-6 {
        let t = sx * (1.0 - rg);
        return (t + rg) / (1.0 + t);
    }
    let a = 1.0 + k / s;
    let b = (a * a - 1.0).sqrt();
    let bsx = b * sx;
    let coth = if bsx > 15.0 { 1.0 } else { 1.0 / bsx.tanh() };
    (1.0 - rg * (a - b * coth)) / (a - rg + b * coth)
}

#[derive(Clone, Copy, Debug)]
pub struct Optics {
    pub k: [f32; 3],
    pub s: [f32; 3],
    pub weight: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Lighten {
    None,
    /// Watercolor pigment load (1 = full strength).
    Water(f32),
    /// Parts of white paint.
    White(u32),
}

#[derive(Clone, Debug)]
pub struct Recipe {
    /// (palette paint index, parts)
    pub parts: Vec<(usize, u32)>,
    pub lighten: Lighten,
    pub srgb: [u8; 3],
    pub de: f32,
}

pub struct Mixer {
    pub medium: Medium,
    pub optics: Vec<Optics>,
    pub names: Vec<String>,
    /// Palette indices that take part in mixtures (the white is handled separately for body paints).
    pub candidates: Vec<usize>,
    pub white: Optics,
    pub white_name: String,
    paper: [f32; 3],
    pairs: Vec<[u32; 2]>,
    triples: Vec<[u32; 3]>,
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn solve_watercolor_k(target: f32, s: f32, rg: f32) -> f32 {
    if r_layer(0.0, s, 1.0, rg) <= target {
        return 0.0;
    }
    let (mut lo, mut hi) = (0.0f32, 80.0f32);
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if r_layer(mid, s, 1.0, rg) > target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

pub fn optics_for(medium: Medium, paint: &Paint) -> Optics {
    optics_for_colour(medium, paint.color, paint.transparency, paint.strength)
}

pub fn optics_for_colour(medium: Medium, color: [u8; 3], transparency: Transparency, strength: f32) -> Optics {
    let lin = srgb8_to_lin(color);
    let s0 = scatter_for(transparency);
    let mut k = [0.0; 3];
    let s = [s0; 3];
    if medium.uses_water() {
        let paper = srgb8_to_lin(PAPER);
        for c in 0..3 {
            k[c] = solve_watercolor_k(lin[c], s0, paper[c]);
        }
    } else {
        for c in 0..3 {
            k[c] = s0 * ks_from_r(lin[c]);
        }
    }
    Optics { k, s, weight: strength.max(0.05) }
}

impl Mixer {
    pub fn new(palette: &Palette) -> Self {
        let medium = palette.medium;
        let optics: Vec<Optics> = palette.paints.iter().map(|p| optics_for(medium, p)).collect();
        let names = palette.paints.iter().map(|p| p.name.clone()).collect();
        let mut white_idx = None;
        if !medium.uses_water() {
            white_idx = palette.paints.iter().position(|p| p.is_white());
        }
        let candidates = (0..palette.paints.len()).filter(|i| Some(*i) != white_idx).collect();
        let (white, white_name) = match white_idx {
            Some(i) => (optics[i], palette.paints[i].name.clone()),
            None => (
                optics_for_colour(medium, DEFAULT_WHITE, Transparency::Opaque, 1.0),
                "Titanium White".to_string(),
            ),
        };
        let mut pairs = Vec::new();
        for a in 1..=PAIR_MAX {
            for b in 1..=PAIR_MAX {
                if gcd(a, b) == 1 {
                    pairs.push([a, b]);
                }
            }
        }
        let mut triples = Vec::new();
        for a in 1..=TRIPLE_MAX {
            for b in 1..=TRIPLE_MAX {
                for c in 1..=TRIPLE_MAX {
                    if gcd(gcd(a, b), c) == 1 {
                        triples.push([a, b, c]);
                    }
                }
            }
        }
        Mixer {
            medium,
            optics,
            names,
            candidates,
            white,
            white_name,
            paper: srgb8_to_lin(PAPER),
            pairs,
            triples,
        }
    }

    pub fn paper_srgb(&self) -> [u8; 3] {
        PAPER
    }

    /// Weighted K and S sums plus total weight of a set of paints.
    fn sums(&self, parts: &[(usize, u32)]) -> ([f32; 3], [f32; 3], f32) {
        let mut k = [0.0; 3];
        let mut s = [0.0; 3];
        let mut w = 0.0;
        for &(i, n) in parts {
            let o = &self.optics[i];
            let wi = n as f32 * o.weight;
            for c in 0..3 {
                k[c] += wi * o.k[c];
                s[c] += wi * o.s[c];
            }
            w += wi;
        }
        (k, s, w)
    }

    #[inline]
    fn finish(&self, k: [f32; 3], s: [f32; 3], w: f32, lighten: Lighten) -> [f32; 3] {
        let mut out = [0.0; 3];
        match lighten {
            Lighten::Water(_) | Lighten::None if self.medium.uses_water() => {
                let load = if let Lighten::Water(l) = lighten { l } else { 1.0 };
                for c in 0..3 {
                    out[c] = r_layer(k[c] / w, s[c] / w, load, self.paper[c]);
                }
            }
            _ => {
                let wp = if let Lighten::White(n) = lighten { n as f32 * self.white.weight } else { 0.0 };
                for c in 0..3 {
                    let kk = k[c] + wp * self.white.k[c];
                    let ss = s[c] + wp * self.white.s[c];
                    out[c] = r_inf(kk / ss.max(1e-6));
                }
            }
        }
        out
    }

    /// Linear RGB reflectance of a mixture.
    pub fn mix(&self, parts: &[(usize, u32)], lighten: Lighten) -> [f32; 3] {
        let (k, s, w) = self.sums(parts);
        if w <= 0.0 {
            return match lighten {
                Lighten::White(n) if n > 0 => self.finish(self.white.k, self.white.s, 0.0, Lighten::White(n)),
                _ => self.paper,
            };
        }
        self.finish(k, s, w, lighten)
    }

    pub fn mix_srgb(&self, parts: &[(usize, u32)], lighten: Lighten) -> [u8; 3] {
        lin_to_srgb8(self.mix(parts, lighten))
    }

    /// Best recipe for a target colour (Lab), using up to `max_paints` palette paints.
    pub fn find(&self, target: [f32; 3], max_paints: usize, allow_lighten: bool) -> Option<Recipe> {
        if self.candidates.is_empty() {
            return None;
        }
        let water = self.medium.uses_water();
        let lightens: Vec<Lighten> = if !allow_lighten {
            vec![Lighten::None]
        } else if water {
            WATER_LOADS.iter().map(|&l| Lighten::Water(l)).collect()
        } else {
            WHITE_PARTS.iter().map(|&n| if n == 0 { Lighten::None } else { Lighten::White(n) }).collect()
        };
        let mut best: Option<(f32, Recipe)> = None;
        let consider = |parts: &[(usize, u32)], best: &mut Option<(f32, Recipe)>| {
            let (k, s, w) = self.sums(parts);
            let penalty = EXTRA_PAINT_PENALTY * (parts.len() as f32 - 1.0);
            for &l in &lightens {
                let lin = self.finish(k, s, w, l);
                let de = delta_e(lin_to_lab(lin), target);
                let score = de + penalty;
                if best.as_ref().is_none_or(|b| score < b.0) {
                    *best = Some((
                        score,
                        Recipe { parts: parts.to_vec(), lighten: l, srgb: lin_to_srgb8(lin), de },
                    ));
                }
            }
        };
        let c = &self.candidates;
        for &i in c {
            consider(&[(i, 1)], &mut best);
        }
        if max_paints >= 2 {
            for a in 0..c.len() {
                for b in a + 1..c.len() {
                    for p in &self.pairs {
                        consider(&[(c[a], p[0]), (c[b], p[1])], &mut best);
                    }
                }
            }
        }
        if max_paints >= 3 {
            for a in 0..c.len() {
                for b in a + 1..c.len() {
                    for d in b + 1..c.len() {
                        for p in &self.triples {
                            consider(&[(c[a], p[0]), (c[b], p[1]), (c[d], p[2])], &mut best);
                        }
                    }
                }
            }
        }
        best.map(|b| b.1)
    }

    pub fn describe_parts(&self, r: &Recipe) -> String {
        let mut parts = r.parts.clone();
        parts.sort_by(|a, b| b.1.cmp(&a.1));
        parts
            .iter()
            .map(|&(i, n)| format!("{} {}", n, self.names[i]))
            .collect::<Vec<_>>()
            .join(" + ")
    }

    pub fn describe_lighten(&self, r: &Recipe) -> String {
        match r.lighten {
            Lighten::None => {
                if self.medium.uses_water() {
                    "water: full strength, very little water".into()
                } else {
                    "no white".into()
                }
            }
            Lighten::Water(load) if load >= 0.9 => "water: full strength, very little water".into(),
            Lighten::Water(load) => format!("water: {} ({:.0}% strength)", water_label(load), load * 100.0),
            Lighten::White(n) => format!("+ {} part{} {}", n, if n == 1 { "" } else { "s" }, self.white_name),
        }
    }
}

pub fn water_label(load: f32) -> &'static str {
    if load >= 0.9 {
        "full strength"
    } else if load >= 0.65 {
        "creamy"
    } else if load >= 0.45 {
        "like milk"
    } else if load >= 0.28 {
        "like coffee"
    } else if load >= 0.15 {
        "like tea"
    } else if load >= 0.07 {
        "light wash"
    } else {
        "very pale wash"
    }
}

pub fn match_label(de: f32) -> &'static str {
    if de < 2.5 {
        "excellent"
    } else if de < 5.0 {
        "good"
    } else if de < 10.0 {
        "close"
    } else {
        "rough"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colour::{hue_deg, srgb8_to_lab};
    use crate::palette::default_store;

    #[test]
    fn single_paint_reproduces_masstone() {
        for pal in default_store().palettes {
            let m = Mixer::new(&pal);
            for (i, p) in pal.paints.iter().enumerate() {
                let got = m.mix_srgb(&[(i, 1)], Lighten::None);
                let de = delta_e(srgb8_to_lab(got), srgb8_to_lab(p.color));
                assert!(de < 2.0, "{} {:?} vs {:?}", p.name, got, p.color);
            }
        }
    }

    #[test]
    fn blue_and_yellow_make_green() {
        let pal = &default_store().palettes[0];
        let m = Mixer::new(pal);
        let mix = m.mix_srgb(&[(1, 1), (3, 1)], Lighten::None);
        let h = hue_deg(srgb8_to_lab(mix));
        assert!((100.0..200.0).contains(&h), "hue {h} {:?}", mix);
    }
}
