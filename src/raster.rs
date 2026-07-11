use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use image::{ImageFormat, RgbaImage};
use usvg::{Color, FillRule};

use crate::path::{Line, Path, Point, point};

pub struct Canvas {
    pub image: RgbaImage,
    pub scale: f32,
    pub offset: Point,
    profile: Option<ScanlineProfile>,
}

#[derive(Debug, Default, Clone)]
pub struct ScanlineProfile {
    pub fills: u64,
    pub break_into_lines: Duration,
    pub active_segment_list: Duration,
    pub active_segment_list_removal: Duration,
    pub covarage_table_generation: Duration,
    pub resolve_pass: Duration,
    pub active_segment_counts: Vec<usize>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        let image = RgbaImage::new(width, height);

        Self {
            image,
            scale: 1.0,
            offset: point(0.0, 0.0),
            profile: Some(ScanlineProfile::default()),
        }
    }

    pub fn save(&self, filename: &str) {
        self.image
            .save_with_format(filename, ImageFormat::Png)
            .unwrap();
    }

    pub fn profile(&self) -> Option<&ScanlineProfile> {
        self.profile.as_ref()
    }

    pub fn dump_profile(&mut self) -> Option<ScanlineProfile> {
        if let Some(profile) = self.profile.take() {
            let (active_min, active_max, active_avg) = if profile.active_segment_counts.is_empty() {
                (0, 0, 0.0)
            } else {
                let mut min = usize::MAX;
                let mut max = 0usize;
                let mut sum = 0usize;

                for count in &profile.active_segment_counts {
                    min = min.min(*count);
                    max = max.max(*count);
                    sum += *count;
                }

                (
                    min,
                    max,
                    sum as f64 / profile.active_segment_counts.len() as f64,
                )
            };

            eprintln!(
                "fill_scanline profile: fills={}, break_into_lines={:?}, active_segment_list={:?}, active_segment_list_removal={:?}, covarage_table_generation={:?}, resolve_pass={:?}, active_segment_count[min={}, max={}, avg={:.2}, samples={}]",
                profile.fills,
                profile.break_into_lines,
                profile.active_segment_list,
                profile.active_segment_list_removal,
                profile.covarage_table_generation,
                profile.resolve_pass,
                active_min,
                active_max,
                active_avg,
                profile.active_segment_counts.len(),
            );

            Some(profile)
        } else {
            None
        }
    }

    pub fn fill_scanline(&mut self, path: &Path, color: &Color, color_alpha: u8) {
        let break_into_lines_start = Instant::now();
        let mut lines = path.break_into_lines();
        let break_into_lines_duration = break_into_lines_start.elapsed();
        apply_transform(&mut lines, self.scale, self.offset);
        // println!("lines: {lines:.?}");

        let mut active_segment_list_duration = Duration::ZERO;
        let mut active_segment_list_removal_duration = Duration::ZERO;
        let mut covarage_table_generation_duration = Duration::ZERO;
        let mut resolve_pass_duration = Duration::ZERO;

        let mut lines_by_start_y: HashMap<u32, HashSet<usize>> = HashMap::new();
        let mut lines_by_end_y: HashMap<u32, HashSet<usize>> = HashMap::new();

        for (index, line) in lines.iter().enumerate() {
            let (y1, y2) = line.y_bounds();
            if let Some(set) = lines_by_start_y.get_mut(&y1) {
                set.insert(index);
            } else {
                let mut set = HashSet::new();
                set.insert(index);
                lines_by_start_y.insert(y1, set);
            }

            if let Some(set) = lines_by_end_y.get_mut(&y2) {
                set.insert(index);
            } else {
                let mut set = HashSet::new();
                set.insert(index);
                lines_by_end_y.insert(y2, set);
            }
        }

        // contain a sorted (by x) index of `lines`
        let mut active_segments: Vec<usize> = vec![];

        for y in 0..self.image.height() {
            // fill of current index
            let mut fill_table: Vec<f32> = vec![0.0; self.image.width() as usize];
            // fill of everything after current index
            let mut covarage_table: Vec<f32> = vec![0.0; self.image.width() as usize];

            // update active segment list
            // sort by x
            let active_segment_list_start = Instant::now();
            if let Some(_lines) = lines_by_start_y.get(&y) {
                active_segments.extend(_lines);
                // its nearly sorted btw
                active_segments.sort_by_key(|index| lines[*index].min_x());
                // println!("active_segments = {active_segments:.?}");
            }
            active_segment_list_duration += active_segment_list_start.elapsed();
            if let Some(profile) = &mut self.profile {
                profile.active_segment_counts.push(active_segments.len());
            }

            let covarage_table_start = Instant::now();
            for line_index in &active_segments {
                // clip it to y-strip
                let Some(line) = lines[*line_index].clip_y(y, y + 1) else {
                    continue;
                };

                let (x_start, x_end) = line.x_bounds();
                // println!("line = {line:.?}");

                for x in x_start..=x_end {
                    let Some(line) = line.clip_x(x, x + 1) else {
                        continue;
                    };

                    let dy = line.1.y - line.0.y;
                    let xmid = (line.0.x + line.1.x) / 2.0 - x as f32;

                    let x = x as usize;
                    covarage_table[x] += dy;
                    fill_table[x] += dy * (1.0 - xmid); // trapezoid, see image.png, or https://www.youtube.com/watch?v=B9bztU1sTFA
                }
            }
            covarage_table_generation_duration += covarage_table_start.elapsed();

            // remove shit from active segment list
            let active_segment_list_removal_start = Instant::now();
            if let Some(lines) = lines_by_end_y.get(&y) {
                let mut indices: Vec<usize> = vec![];
                for line in lines {
                    let index = active_segments.iter().position(|it| *it == *line).unwrap();
                    indices.push(index);
                }

                indices.sort();
                for index in indices.iter().rev() {
                    active_segments.remove(*index);
                }
            }
            active_segment_list_removal_duration += active_segment_list_removal_start.elapsed();

            // resolve pass
            let resolve_pass_start = Instant::now();
            let mut acc: f32 = 0.0;
            for x in 0..self.image.width() {
                let winding = acc + fill_table[x as usize];

                let opacity = match path.fill_rule {
                    FillRule::NonZero => winding.abs().min(1.0),
                    FillRule::EvenOdd => {
                        if winding as u32 % 2 == 0 {
                            winding % 1.0
                        } else {
                            1.0 - (winding % 1.0)
                        }
                    }
                };

                let pixel = self.image.get_pixel_mut(x, y);
                shitty_blend(
                    &[color.red, color.green, color.blue, color_alpha],
                    &mut pixel.0,
                    opacity,
                );

                acc += covarage_table[x as usize];
            }
            resolve_pass_duration += resolve_pass_start.elapsed();
        }

        if let Some(profile) = &mut self.profile {
            profile.fills += 1;
            profile.break_into_lines += break_into_lines_duration;
            profile.active_segment_list += active_segment_list_duration;
            profile.active_segment_list_removal += active_segment_list_removal_duration;
            profile.covarage_table_generation += covarage_table_generation_duration;
            profile.resolve_pass += resolve_pass_duration;
        }

        // println!("active_segments => {active_segments:.?}");
        // println!("lines.len() = {:.?}", lines.len());
        // println!("lines = {lines:.?}");
        // println!("lines_by_end_y => {lines_by_end_y:.?}");
    }
}

fn apply_transform(lines: &mut Vec<Line>, scale: f32, offset: Point) {
    for line in lines {
        line.0 = line.0 * scale + offset;
        line.1 = line.1 * scale + offset;
    }
}

pub fn raster_band(path: &Path, image: &mut RgbaImage) {
    // we probably need a custom image type just for multithreading

    // break image into h=16px bands, put each line segment into it
    let band_h = 16;
    let mut band_count = image.height() / band_h;
    let last_band_height = image.height() % band_h;
    if last_band_height != 0 {
        band_count += 1;
    }
}

struct BandOutput {}

/// src-over composite of `source` onto `dest`, both straight (non-premultiplied) RGBA.
/// `t` scales the source alpha, e.g. pixel coverage.
fn shitty_blend(source: &[u8; 4], dest: &mut [u8; 4], t: f32) {
    let src_a = (source[3] as f32) / 255.0 * t.clamp(0.0, 1.0);
    let dst_a = (dest[3] as f32) / 255.0;
    let out_a = src_a + dst_a * (1.0 - src_a);

    if out_a <= f32::EPSILON {
        *dest = [0, 0, 0, 0];
        return;
    }

    for i in 0..3 {
        let src = (source[i] as f32) / 255.0;
        let dst = (dest[i] as f32) / 255.0;
        let out = (src * src_a + dst * dst_a * (1.0 - src_a)) / out_a;
        dest[i] = (out * 255.0 + 0.5) as u8;
    }

    dest[3] = (out_a * 255.0 + 0.5) as u8;
}
