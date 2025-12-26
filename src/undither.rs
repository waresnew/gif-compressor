use crate::image::{GifFrame, RGB};

pub fn undither(frame: &mut GifFrame) -> Vec<Vec<RGB>> {
    let height = frame.canvas_height();
    let width = frame.canvas_width();
    let mut ans = vec![vec![RGB::default(); width]; height];
    for i in 0..height {
        for j in 0..width {
            let cur = frame.canvas[i][j];
            let mut weight_len = 0;
            let mut sum_r = 0;
            let mut sum_g = 0;
            let mut sum_b = 0;
            let mut prewitt_input = [[0; 3]; 3];
            let canvas = &frame.canvas;
            let palette = frame.get_palette();
            let for_each_neighbour = |f: &mut dyn FnMut((isize, isize), RGB)| {
                for di in -1..=1_isize {
                    for dj in -1..=1_isize {
                        if di == 0 && dj == 0 {
                            continue;
                        }
                        let ni = (i as isize + di).clamp(0, height as isize - 1) as usize;
                        let nj = (j as isize + dj).clamp(0, width as isize - 1) as usize;

                        let neighbour = canvas[ni][nj];
                        f((di, dj), neighbour);
                    }
                }
            };
            for_each_neighbour(&mut |(di, dj), neighbour| {
                prewitt_input[(di + 1) as usize][(dj + 1) as usize] = neighbour.as_luminance();
            });
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
            for_each_neighbour(&mut |(_di, _dj), neighbour| {
                let avg = cur.average(neighbour);
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
            });
            ans[i][j] = RGB {
                r: (sum_r / weight_len) as u8,
                g: (sum_g / weight_len) as u8,
                b: (sum_b / weight_len) as u8,
            };
        }
    }
    assert_eq!(ans.len(), height);
    assert_eq!(ans[0].len(), width);
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
