use crate::image::{GifFrame, RGB, RGB_TRANSPARENT};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
pub fn undither_frame(frame: &mut GifFrame) {
    let height = frame.canvas_height();
    let width = frame.canvas_width();
    let mut ans = vec![RGB_TRANSPARENT; height * width];
    frame
        .canvas
        .buffer
        .par_chunks_exact(width)
        .zip(ans.par_chunks_exact_mut(width))
        .enumerate()
        .for_each_with(
            FxHashMap::default(),
            |nn_cache, (i, (input_row, output_row))| {
                for (j, cur) in input_row.iter().enumerate() {
                    let mut weight_len: u32 = 0;
                    let mut sum_r = 0;
                    let mut sum_g = 0;
                    let mut sum_b = 0;
                    let mut all_transparent = true;
                    let mut neighbours: Vec<RGB> = Vec::with_capacity(8);
                    for di in -1..=1_isize {
                        for dj in -1..=1_isize {
                            if di == 0 && dj == 0 {
                                continue;
                            }
                            let ni = (i as isize + di).clamp(0, height as isize - 1) as usize;
                            let nj = (j as isize + dj).clamp(0, width as isize - 1) as usize;

                            let neighbour = frame.canvas.get(ni, nj);
                            if !neighbour.transparent {
                                all_transparent = false;
                            }
                            neighbours.push(neighbour);
                        }
                    }
                    if all_transparent && cur.transparent {
                        output_row[j] = input_row[j];
                        continue;
                    }
                    let prewitt = prewitt_3x3_mag([
                        [
                            neighbours[0].as_luma() as i32,
                            neighbours[1].as_luma() as i32,
                            neighbours[2].as_luma() as i32,
                        ],
                        [
                            neighbours[3].as_luma() as i32,
                            cur.as_luma() as i32,
                            neighbours[4].as_luma() as i32,
                        ],
                        [
                            neighbours[5].as_luma() as i32,
                            neighbours[6].as_luma() as i32,
                            neighbours[7].as_luma() as i32,
                        ],
                    ]);
                    let prewitt_high_threshold = 256;
                    let prewitt_low_threshold = 160;
                    let cur_weight = if prewitt > prewitt_high_threshold {
                        output_row[j] = *cur;
                        continue;
                    } else if prewitt > prewitt_low_threshold {
                        24
                    } else {
                        8
                    };
                    weight_len += cur_weight as u32;
                    sum_r += cur_weight as u32 * (cur.r as u32);
                    sum_b += cur_weight as u32 * (cur.b as u32);
                    sum_g += cur_weight as u32 * (cur.g as u32);
                    for neighbour in neighbours {
                        let avg = cur.average(neighbour);
                        let nearest = frame
                            .get_palette()
                            .get_nearest(avg, *cur, neighbour, nn_cache);
                        let weight = if let Some(nearest) = nearest {
                            let dis1 = cur.distance_sq(avg);
                            let dis2 = avg.distance_sq(nearest);
                            if dis2 >= dis1 * 2 {
                                8
                            } else if dis2 >= dis1 {
                                6
                            } else if dis2 * 3 >= dis1 * 2 {
                                1
                            } else {
                                0
                            }
                        } else {
                            8
                        };

                        sum_r += weight as u32 * (neighbour.r as u32);
                        sum_g += weight as u32 * (neighbour.g as u32);
                        sum_b += weight as u32 * (neighbour.b as u32);
                        weight_len += weight as u32;
                    }
                    output_row[j] = RGB::new(
                        (sum_r / weight_len) as u8,
                        (sum_g / weight_len) as u8,
                        (sum_b / weight_len) as u8,
                    );
                }
            },
        );
    frame.canvas.buffer = ans;
}
#[inline]
fn prewitt_3x3_mag(input: [[i32; 3]; 3]) -> u32 {
    let gx = input[0][0] + input[1][0] + input[2][0] - input[0][2] - input[1][2] - input[2][2];
    let gy = input[0][0] + input[0][1] + input[0][2] - input[2][0] - input[2][1] - input[2][2];
    (gx * gx + gy * gy).isqrt() as u32
}
