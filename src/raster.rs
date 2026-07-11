use std::collections::{HashMap, HashSet};

use image::{ImageFormat, RgbaImage};
use usvg::{Color, FillRule};

use crate::path::{Line, Path, Point, point};

pub struct Canvas {
    pub image: RgbaImage,
    pub scale: f32,
    pub offset: Point,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        let image = RgbaImage::new(width, height);

        Self {
            image,
            scale: 1.0,
            offset: point(0.0, 0.0),
        }
    }

    pub fn save(&self, filename: &str) {
        self.image
            .save_with_format(filename, ImageFormat::Png)
            .unwrap();
    }

    pub fn fill_scanline(&mut self, path: &Path, color: &Color, color_alpha: u8) {
        let mut lines = path.break_into_lines();
        apply_transform(&mut lines, self.scale, self.offset);
        // println!("lines: {lines:.?}");

        let mut lines_by_start_y: HashMap<u32, HashSet<usize>> = HashMap::new();
        let mut lines_by_end_y: HashMap<u32, HashSet<usize>> = HashMap::new();

        let mut min_y: u32 = 0;
        let mut max_y: u32 = 0;

        for (index, line) in lines.iter().enumerate() {
            let (y1, y2) = line.y_bounds();
            min_y = y1.min(min_y);
            max_y = y2.max(max_y);
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

        let w = self.image.width() as usize;
        let h = self.image.height() as usize;
        let buffer = self.image.as_mut();

        // contain a sorted (by x) index of `lines`
        let mut active_segments: Vec<usize> = vec![];

        // fill of current index
        let mut fill_table: Vec<f32> = vec![0.0; w];
        // fill of everything after current index
        let mut covarage_table: Vec<f32> = vec![0.0; w];

        for y in min_y..=max_y {
            // update active segment list
            // sort by x
            if let Some(_lines) = lines_by_start_y.get(&y) {
                active_segments.extend(_lines);
                // its nearly sorted btw
                active_segments.sort_by_key(|index| lines[*index].min_x());
                // println!("active_segments = {active_segments:.?}");
            }

            let mut row_start: u32 = 0;
            let mut row_end: u32 = 0;
            let should_skip = active_segments.len() == 0;

            if !should_skip {
                fill_table.fill(0.0);
                covarage_table.fill(0.0);

                for line_index in &active_segments {
                    // clip it to y-strip
                    let Some(line) = lines[*line_index].clip_y(y, y + 1) else {
                        continue;
                    };

                    let (x_start, x_end) = line.x_bounds();
                    // we can actually produce strip?
                    row_start = row_start.min(x_start);
                    row_end = row_end.max(x_end);
                    // println!("line = {line:.?}");

                    for x in x_start..=x_end {
                        let Some(line) = line.clip_x(x, x + 1) else {
                            continue;
                        };

                        let dy = line.1.y - line.0.y;
                        let xmid = (line.0.x + line.1.x) / 2.0 - x as f32;

                        let x = x as usize;
                        covarage_table[x] += dy;
                        fill_table[x] += dy * (1.0 - xmid); // trapezoid, see https://www.youtube.com/watch?v=B9bztU1sTFA
                    }
                }
            }

            // remove shit from active segment list
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

            if !should_skip {
                // resolve pass
                let pixels = &mut buffer[4 * w * y as usize..][..w * 4];
                let mut acc: f32 = 0.0;
                for x in row_start as usize..=row_end as usize {
                    let px = &mut pixels[4 * x..][..4];
                    let winding = acc + fill_table[x];

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

                    if opacity < f32::EPSILON {
                        continue;
                    }

                    shitty_blend(
                        &[color.red, color.green, color.blue, color_alpha],
                        px.try_into().unwrap(),
                        opacity,
                    );

                    acc += covarage_table[x];
                }
            }
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
#[inline(always)]
fn shitty_blend(source: &[u8; 4], dest: &mut [u8; 4], opacity: f32) {
    let src_a = source[3] as u32 * (opacity * 255.0) as u32 / 255;

    if src_a == 255 {
        dest[0] = source[0];
        dest[1] = source[1];
        dest[2] = source[2];
        dest[3] = source[3];
        return;
    }

    let dst_a = dest[3] as u32;
    let out_a = src_a + dst_a * (255 - src_a) / 255;

    if out_a == 0 {
        *dest = [0, 0, 0, 0];
        return;
    }

    for c in 0..3 {
        let s = source[c] as u32;
        let d = dest[c] as u32;
        let out = (s * src_a + d * dst_a * (255 - src_a) / 255) / out_a;

        dest[c] = out as u8;
    }

    dest[3] = out_a as u8;
}
