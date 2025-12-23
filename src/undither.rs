use crate::image::{GifFrame, Palette, RGB};

/// returns unpaletted 2d vec of RGBA's
pub fn undither(frame: &GifFrame, palette: &Palette) -> Vec<Vec<RGB>> {
    let mut ans = vec![vec![RGB::default(); frame.width]; frame.height];
    for i in 0..frame.height {
        for j in 0..frame.width {
            let cur = frame.indices[i][j];
            if let Some(transparent) = frame.transparent
                && cur == transparent
            {
                continue;
            }
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
                    let ni = (i as isize + di).clamp(0, frame.height as isize - 1) as usize;
                    let nj = (j as isize + dj).clamp(0, frame.width as isize - 1) as usize;
                    let neighbour = frame.indices[ni][nj];
                    prewitt_input[(di + 1) as usize][(dj + 1) as usize] =
                        palette[neighbour].as_luminance();
                    if let Some(transparent) = frame.transparent
                        && neighbour == transparent
                    {
                        continue;
                    }
                    let avg = palette[cur].average(&palette[neighbour]);
                    //OPTIMIZE: precompute
                    let nearest = palette.nearest(&avg, cur, neighbour);
                    let dis1 = palette[cur].distance_sq(&avg);
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
                    sum_r += weight * (palette[neighbour].r as u32);
                    sum_g += weight * (palette[neighbour].g as u32);
                    sum_b += weight * (palette[neighbour].b as u32);
                    weight_len += weight;
                }
            }
            //OPTIMIZE:precompute
            let prewitt = prewitt_3x3_mag(prewitt_input);
            let prewitt_high_threshold = 256;
            let prewitt_low_threshold = 160;
            let cur_weight = if prewitt > prewitt_high_threshold {
                ans[i][j] = palette[cur];
                continue;
            } else if prewitt > prewitt_low_threshold {
                24
            } else {
                8
            };
            weight_len += cur_weight;
            sum_r += cur_weight * (palette[cur].r as u32);
            sum_b += cur_weight * (palette[cur].b as u32);
            sum_g += cur_weight * (palette[cur].g as u32);
            ans[i][j] = RGB {
                r: (sum_r / weight_len) as u8,
                g: (sum_g / weight_len) as u8,
                b: (sum_b / weight_len) as u8,
            };
        }
    }
    assert_eq!(ans.len(), frame.height);
    assert_eq!(ans[0].len(), frame.width);
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
