use std::collections::{HashMap, HashSet};

use image::{ImageFormat, Rgba, Rgba32FImage, RgbaImage};

use crate::path::{Line, Path};

pub fn raster_scanline(path: &Path, image: &mut RgbaImage) {
    let lines = path.break_into_lines();

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

    for y in 0..image.height() {
        // fill of current index
        let mut fill_table: Vec<f32> = vec![0.0; image.width() as usize];
        // fill of everything after current index
        let mut covarage_table: Vec<f32> = vec![0.0; image.width() as usize];

        // update active segment list?
        // sort by x
        if let Some(_lines) = lines_by_start_y.get(&y) {
            active_segments.extend(_lines);
            // its nearly sorted btw
            active_segments.sort_by_key(|index| lines[*index].min_x());
            // println!("active_segments = {active_segments:.?}");
        }

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

        // resolve pass
        let mut acc: f32 = 0.0;
        for x in 0..image.width() {
            let actual_coverage = acc + fill_table[x as usize];

            let pixel = image.get_pixel_mut(x, y);
            shitty_blend(&[255, 0, 0, 255], &mut pixel.0, actual_coverage / 2.0);

            acc += covarage_table[x as usize];
        }
    }

    // println!("active_segments => {active_segments:.?}");
    // println!("lines.len() = {:.?}", lines.len());
    // println!("lines = {lines:.?}");
    // println!("lines_by_end_y => {lines_by_end_y:.?}");
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
