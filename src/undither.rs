use crate::image::{Canvas, GifFrame, RGB};

pub fn undither_frame(frame: &mut GifFrame) {
    let height = frame.canvas_height();
    let width = frame.canvas_width();
    let mut ans = Canvas::blank(height, width);
    for i in 0..height {
        for j in 0..width {
            let cur = frame.canvas.get(i, j);
            let mut weight_len: u32 = 0;
            let mut sum_r = 0;
            let mut sum_g = 0;
            let mut sum_b = 0;
            fn for_each_neighbour<F>(
                mut f: F,
                (i, j): (usize, usize),
                (height, width): (usize, usize),
                frame_canvas: &Canvas,
            ) where
                F: FnMut((isize, isize), RGB),
            {
                for di in -1..=1_isize {
                    for dj in -1..=1_isize {
                        if di == 0 && dj == 0 {
                            continue;
                        }
                        let ni = (i as isize + di).clamp(0, height as isize - 1) as usize;
                        let nj = (j as isize + dj).clamp(0, width as isize - 1) as usize;

                        let neighbour = frame_canvas.get(ni, nj);
                        f((di, dj), neighbour);
                    }
                }
            }
            let mut prewitt_input = [[0; 3]; 3];
            for_each_neighbour(
                |(di, dj), neighbour| {
                    prewitt_input[(di + 1) as usize][(dj + 1) as usize] = neighbour.as_luma();
                },
                (i, j),
                (height, width),
                frame.canvas,
            );
            let prewitt = prewitt_3x3_mag(prewitt_input);
            let prewitt_high_threshold = 256;
            let prewitt_low_threshold = 160;
            let cur_weight = if prewitt > prewitt_high_threshold {
                *ans.get_mut(i, j) = cur;
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
            for_each_neighbour(
                |(_di, _dj), neighbour| {
                    let avg = cur.average(neighbour);
                    let nearest = frame.get_palette().get_nearest(avg, cur, neighbour);
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
                },
                (i, j),
                (height, width),
                frame.canvas,
            );
            *ans.get_mut(i, j) = RGB::new(
                (sum_r / weight_len) as u8,
                (sum_g / weight_len) as u8,
                (sum_b / weight_len) as u8,
            );
        }
    }
    for i in 0..height {
        for j in 0..width {
            *frame.canvas.get_mut(i, j) = ans.get(i, j);
        }
    }
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
