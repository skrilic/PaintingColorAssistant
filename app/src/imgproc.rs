//! Image work: loading, cropping, value (greyscale) levels and colour levels (k-means in Lab).

use crate::colour::{delta_e, grey_for_lightness, srgb8_to_lab};
use image::RgbImage;

/// Longest side of the working copy used for display and analysis.
pub const WORK_MAX_SIDE: u32 = 1400;
/// Pixels sampled for k-means.
pub const KMEANS_SAMPLES: usize = 60_000;
pub const KMEANS_ITERATIONS: usize = 12;

pub fn load_path(path: &std::path::Path) -> Result<RgbImage, String> {
    image::open(path).map(|i| i.to_rgb8()).map_err(|e| e.to_string())
}

pub fn load_bytes(bytes: &[u8]) -> Result<RgbImage, String> {
    image::load_from_memory(bytes).map(|i| i.to_rgb8()).map_err(|e| e.to_string())
}

/// Crop rectangle in pixels of the original: (x, y, w, h).
pub fn crop(img: &RgbImage, r: (u32, u32, u32, u32)) -> RgbImage {
    image::imageops::crop_imm(img, r.0, r.1, r.2.max(1), r.3.max(1)).to_image()
}

pub fn fit_working(img: &RgbImage) -> RgbImage {
    let (w, h) = img.dimensions();
    let m = w.max(h);
    if m <= WORK_MAX_SIDE {
        return img.clone();
    }
    let s = WORK_MAX_SIDE as f32 / m as f32;
    let nw = ((w as f32 * s).round() as u32).max(1);
    let nh = ((h as f32 * s).round() as u32).max(1);
    image::imageops::resize(img, nw, nh, image::imageops::FilterType::Triangle)
}

pub struct Analysis {
    pub lab: Vec<[f32; 3]>,
    pub l_min: f32,
    pub l_max: f32,
}

pub fn analyse(img: &RgbImage) -> Analysis {
    let lab: Vec<[f32; 3]> = img.pixels().map(|p| srgb8_to_lab(p.0)).collect();
    // Use 0.5 / 99.5 percentiles so a few specks do not stretch the value range.
    let mut ls: Vec<f32> = lab.iter().step_by(7).map(|l| l[0]).collect();
    ls.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pick = |q: f32| ls[((ls.len() - 1) as f32 * q) as usize];
    let (l_min, l_max) = if ls.is_empty() { (0.0, 100.0) } else { (pick(0.005), pick(0.995)) };
    Analysis { lab, l_min, l_max: l_max.max(l_min + 1.0) }
}

pub fn greyscale(a: &Analysis) -> Vec<u8> {
    let mut out = Vec::with_capacity(a.lab.len() * 4);
    for l in &a.lab {
        let g = grey_for_lightness(l[0]);
        out.extend_from_slice(&[g, g, g, 255]);
    }
    out
}

/// Value levels: L* range split in `n` equal steps.
pub struct ValueLevels {
    pub index: Vec<u8>,
    /// L* drawn for each level, dark to light.
    pub lightness: Vec<f32>,
    pub share: Vec<f32>,
}

pub fn value_level_of(l: f32, a: &Analysis, n: usize) -> usize {
    let t = ((l - a.l_min) / (a.l_max - a.l_min)).clamp(0.0, 0.9999);
    (t * n as f32) as usize
}

pub fn value_levels(a: &Analysis, n: usize) -> ValueLevels {
    let n = n.max(2);
    let mut index = Vec::with_capacity(a.lab.len());
    let mut count = vec![0usize; n];
    for l in &a.lab {
        let i = value_level_of(l[0], a, n);
        count[i] += 1;
        index.push(i as u8);
    }
    // Display levels as evenly spaced greys from black to white (a printed value scale).
    let lightness = (0..n).map(|i| 100.0 * i as f32 / (n - 1) as f32).collect();
    let total = a.lab.len().max(1) as f32;
    ValueLevels { index, lightness, share: count.iter().map(|&c| c as f32 / total).collect() }
}

pub struct ColourLevels {
    pub index: Vec<u8>,
    pub lab: Vec<[f32; 3]>,
    pub srgb: Vec<[u8; 3]>,
    pub share: Vec<f32>,
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn unit(&mut self) -> f32 {
        (self.next() % 1_000_000) as f32 / 1_000_000.0
    }
}

fn nearest(c: &[[f32; 3]], p: [f32; 3]) -> usize {
    let mut best = 0;
    let mut bd = f32::MAX;
    for (i, q) in c.iter().enumerate() {
        let d = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
        if d < bd {
            bd = d;
            best = i;
        }
    }
    best
}

/// Reduce the picture to `k` colours with k-means++ in Lab; levels sorted dark to light.
pub fn colour_levels(img: &RgbImage, a: &Analysis, k: usize) -> ColourLevels {
    let n = a.lab.len();
    let k = k.clamp(2, 32).min(n.max(2));
    let step = (n / KMEANS_SAMPLES).max(1);
    let samples: Vec<[f32; 3]> = a.lab.iter().step_by(step).copied().collect();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    // k-means++ seeding
    let mut centres = vec![samples[(rng.next() as usize) % samples.len()]];
    let mut dist: Vec<f32> = samples.iter().map(|p| delta_e(*p, centres[0]).powi(2)).collect();
    while centres.len() < k {
        let total: f32 = dist.iter().sum();
        let mut pick = rng.unit() * total;
        let mut chosen = samples.len() - 1;
        for (i, d) in dist.iter().enumerate() {
            pick -= d;
            if pick <= 0.0 {
                chosen = i;
                break;
            }
        }
        let c = samples[chosen];
        centres.push(c);
        for (i, p) in samples.iter().enumerate() {
            dist[i] = dist[i].min(delta_e(*p, c).powi(2));
        }
    }
    for _ in 0..KMEANS_ITERATIONS {
        let mut sum = vec![[0.0f64; 3]; k];
        let mut cnt = vec![0usize; k];
        for p in &samples {
            let i = nearest(&centres, *p);
            for c in 0..3 {
                sum[i][c] += p[c] as f64;
            }
            cnt[i] += 1;
        }
        for i in 0..k {
            if cnt[i] > 0 {
                centres[i] = [
                    (sum[i][0] / cnt[i] as f64) as f32,
                    (sum[i][1] / cnt[i] as f64) as f32,
                    (sum[i][2] / cnt[i] as f64) as f32,
                ];
            }
        }
    }
    centres.sort_by(|x, y| x[0].partial_cmp(&y[0]).unwrap());
    let mut index = Vec::with_capacity(n);
    let mut rgb_sum = vec![[0u64; 3]; k];
    let mut cnt = vec![0usize; k];
    for (p, px) in a.lab.iter().zip(img.pixels()) {
        let i = nearest(&centres, *p);
        index.push(i as u8);
        for c in 0..3 {
            rgb_sum[i][c] += px.0[c] as u64;
        }
        cnt[i] += 1;
    }
    let srgb: Vec<[u8; 3]> = (0..k)
        .map(|i| {
            let c = cnt[i].max(1) as u64;
            [(rgb_sum[i][0] / c) as u8, (rgb_sum[i][1] / c) as u8, (rgb_sum[i][2] / c) as u8]
        })
        .collect();
    let lab = srgb.iter().map(|c| srgb8_to_lab(*c)).collect();
    let total = n.max(1) as f32;
    ColourLevels { index, lab, srgb, share: cnt.iter().map(|&c| c as f32 / total).collect() }
}

/// RGBA pixels for a level map; levels other than `highlight` are faded when one is chosen.
pub fn render_levels(index: &[u8], colours: &[[u8; 3]], highlight: Option<usize>) -> Vec<u8> {
    let mut out = Vec::with_capacity(index.len() * 4);
    for &i in index {
        let c = colours[i as usize];
        match highlight {
            Some(h) if h != i as usize => {
                let g = ((c[0] as u32 + c[1] as u32 + c[2] as u32) / 3) as u8;
                let f = |v: u8| ((v as u32 + g as u32 * 2 + 255 * 3) / 6) as u8;
                out.extend_from_slice(&[f(c[0]), f(c[1]), f(c[2]), 255]);
            }
            _ => out.extend_from_slice(&[c[0], c[1], c[2], 255]),
        }
    }
    out
}

pub fn rgba(img: &RgbImage) -> Vec<u8> {
    let mut out = Vec::with_capacity((img.width() * img.height() * 4) as usize);
    for p in img.pixels() {
        out.extend_from_slice(&[p.0[0], p.0[1], p.0[2], 255]);
    }
    out
}

/// Average colour of a small square around (x, y).
pub fn sample_avg(img: &RgbImage, x: u32, y: u32, r: u32) -> [u8; 3] {
    let (w, h) = img.dimensions();
    let mut s = [0u32; 3];
    let mut n = 0;
    for yy in y.saturating_sub(r)..(y + r + 1).min(h) {
        for xx in x.saturating_sub(r)..(x + r + 1).min(w) {
            let p = img.get_pixel(xx, yy).0;
            for c in 0..3 {
                s[c] += p[c] as u32;
            }
            n += 1;
        }
    }
    let n = n.max(1);
    [(s[0] / n) as u8, (s[1] / n) as u8, (s[2] / n) as u8]
}

// ---------- procedural sample picture ----------

fn hash(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) & 0xFFFF) as f32 / 65535.0
}

fn noise(x: f32, y: f32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - xi as f32, y - yi as f32);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = hash(xi, yi) + (hash(xi + 1, yi) - hash(xi, yi)) * sx;
    let b = hash(xi, yi + 1) + (hash(xi + 1, yi + 1) - hash(xi, yi + 1)) * sx;
    a + (b - a) * sy
}

fn fbm(x: f32, y: f32) -> f32 {
    let mut v = 0.0;
    let mut amp = 0.5;
    let mut f = 1.0;
    for _ in 0..5 {
        v += amp * noise(x * f, y * f);
        f *= 2.0;
        amp *= 0.5;
    }
    v
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// A painted-looking landscape at sunset, drawn from shapes and noise.
pub fn sample_landscape(w: u32, h: u32) -> RgbImage {
    let mut img = RgbImage::new(w, h);
    let horizon = 0.58;
    for py in 0..h {
        for px in 0..w {
            let x = px as f32 / w as f32;
            let y = py as f32 / h as f32;
            let n = fbm(x * 6.0, y * 6.0);
            // sky
            let mut c = if y < 0.3 {
                lerp3([52.0, 82.0, 150.0], [120.0, 140.0, 190.0], y / 0.3)
            } else {
                lerp3([120.0, 140.0, 190.0], [244.0, 178.0, 110.0], (y - 0.3) / (horizon - 0.3))
            };
            // clouds
            let cl = fbm(x * 4.0 + 3.0, y * 10.0);
            if y < 0.45 && cl > 0.55 {
                c = lerp3(c, [236.0, 196.0, 190.0], (cl - 0.55) * 4.0);
            }
            // sun
            let (sx, sy) = (0.68, 0.47);
            let d = ((x - sx) * (x - sx) * (w as f32 / h as f32).powi(2) + (y - sy) * (y - sy)).sqrt();
            if d < 0.05 {
                c = [255.0, 236.0, 170.0];
            } else if d < 0.2 {
                c = lerp3(c, [255.0, 214.0, 140.0], (0.2 - d) / 0.15 * 0.6);
            }
            // far mountains
            let m1 = 0.43 + 0.06 * (x * 7.0).sin() * 0.5 + 0.05 * fbm(x * 5.0, 1.0);
            if y > m1 && y < horizon {
                c = lerp3([118.0, 106.0, 150.0], [150.0, 130.0, 160.0], (y - m1) * 8.0);
            }
            // near hills
            let m2 = 0.5 + 0.05 * (x * 3.0 + 1.0).sin() + 0.04 * fbm(x * 8.0, 2.0);
            if y > m2 && y < horizon {
                c = lerp3([60.0, 88.0, 70.0], [40.0, 66.0, 54.0], n);
            }
            // lake with reflection
            if y >= horizon && y < 0.74 {
                let ry = horizon - (y - horizon) * 1.4;
                let base = lerp3([120.0, 140.0, 190.0], [244.0, 178.0, 110.0], (ry - 0.3) / (horizon - 0.3));
                let ripple = 0.85 + 0.15 * (y * 300.0 + n * 6.0).sin();
                c = [base[0] * 0.78 * ripple, base[1] * 0.8 * ripple, base[2] * 0.9 * ripple];
                let sd = (x - sx).abs();
                if sd < 0.035 + (y - horizon) * 0.3 && (y * 260.0 + n * 9.0).sin() > 0.2 {
                    c = lerp3(c, [255.0, 220.0, 150.0], 0.7);
                }
            }
            // meadow
            let shore = 0.74 + 0.03 * (x * 9.0).sin() * fbm(x * 3.0, 4.0);
            if y >= shore {
                let g = fbm(x * 30.0, y * 30.0);
                c = lerp3([112.0, 138.0, 52.0], [62.0, 90.0, 36.0], (y - shore) * 3.0 + g * 0.5);
                if g > 0.62 {
                    c = lerp3(c, [214.0, 86.0, 60.0], (g - 0.62) * 6.0); // poppies
                }
            }
            img.put_pixel(px, py, image::Rgb([c[0].clamp(0.0, 255.0) as u8, c[1].clamp(0.0, 255.0) as u8, c[2].clamp(0.0, 255.0) as u8]));
        }
    }
    // trees: dark triangles on the left shore
    let trees = [(0.08, 0.80, 0.30), (0.14, 0.78, 0.24), (0.2, 0.81, 0.2), (0.88, 0.79, 0.22)];
    for (tx, base, height) in trees {
        let top = base - height;
        for py in ((top * h as f32) as u32)..((base * h as f32) as u32).min(h) {
            let y = py as f32 / h as f32;
            let half = (y - top) / height * 0.045;
            for px in (((tx - half) * w as f32).max(0.0) as u32)..(((tx + half) * w as f32) as u32).min(w) {
                let x = px as f32 / w as f32;
                let shade = if x < tx { 0.8 } else { 1.0 } * (0.85 + 0.3 * fbm(x * 40.0, y * 40.0));
                img.put_pixel(px, py, image::Rgb([(28.0 * shade) as u8, (58.0 * shade) as u8, (40.0 * shade) as u8]));
            }
        }
    }
    img
}
