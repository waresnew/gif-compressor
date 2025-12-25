use crate::image::{GifFrame, RGB};

pub fn undither(frame: &mut GifFrame) -> Vec<Vec<RGB>> {
    let mut ans = vec![vec![RGB::default(); frame.canvas_width()]; frame.canvas_height()];
    for i in 0..frame.canvas_height() {
        for j in 0..frame.canvas_width() {
            let cur = frame.canvas[i][j];
            let mut weight_len = 0;
            let mut sum_r = 0;
            let mut sum_g = 0;
            let mut sum_b = 0;
            let mut prewitt_input = [[0; 3]; 3];
            for di in -1..=1_isize {
                for dj in -1..=1_isize {
                    if di == 0 && dj == 0 {
                        continue;
                    }
                    let ni =
                        (i as isize + di).clamp(0, frame.canvas_height() as isize - 1) as usize;
                    let nj = (j as isize + dj).clamp(0, frame.canvas_width() as isize - 1) as usize;
                    let neighbour = frame.canvas[ni][nj];
                    let avg = cur.average(neighbour);
                    let palette = frame.get_palette_mut();
                    let nearest = palette.get_nearest(avg, cur, neighbour);
                    let dis1 = cur.distance_sq(avg);
                    let dis2 = avg.distance_sq(nearest);
                    let weight = if dis2 >= dis1 * 2 {
                        8
                    } else if dis2 >= dis1 {
                        6
                    } else if dis2 * 3 >= dis1 * 2 {
                        1
                    } else {
                        0
                    } as u32;
                    sum_r += weight * (neighbour.r as u32);
                    sum_g += weight * (neighbour.g as u32);
                    sum_b += weight * (neighbour.b as u32);
                    weight_len += weight;
                    prewitt_input[(di + 1) as usize][(dj + 1) as usize] = neighbour.as_luminance();
                }
            }
            let prewitt = prewitt_3x3_mag(prewitt_input);
            let prewitt_high_threshold = 256;
            let prewitt_low_threshold = 160;
            let cur_weight = if prewitt > prewitt_high_threshold {
                ans[i][j] = cur;
                continue;
            } else if prewitt > prewitt_low_threshold {
                24
            } else {
                8
            };
            weight_len += cur_weight;
            sum_r += cur_weight * (cur.r as u32);
            sum_b += cur_weight * (cur.b as u32);
            sum_g += cur_weight * (cur.g as u32);
            ans[i][j] = RGB {
                r: (sum_r / weight_len) as u8,
                g: (sum_g / weight_len) as u8,
                b: (sum_b / weight_len) as u8,
            };
        }
    }
    assert_eq!(ans.len(), frame.canvas_height());
    assert_eq!(ans[0].len(), frame.canvas_width());
    ans
}
/// returns centre value only
fn prewitt_3x3_mag(input: [[u8; 3]; 3]) -> u32 {
    let gx = convolve_3x3(&input, [[1, 0, -1], [1, 0, -1], [1, 0, -1]]);
    let gy = convolve_3x3(&input, [[1, 1, 1], [0, 0, 0], [-1, -1, -1]]);
    (gx * gx + gy * gy).isqrt() as u32
}
/// returns centre value only
fn convolve_3x3(input: &[[u8; 3]; 3], kernel: [[i8; 3]; 3]) -> i32 {
    let mut ans = 0;
    for m in 0..3 {
        for n in 0..3 {
            ans += input[m][n] as i32 * kernel[m][n] as i32;
        }
    }
    ans
}
