use gif::{DisposalMethod, Encoder, Frame};
use gif_compressor::image::{GifFrame, Palette, RGB};
use gif_compressor::kdtree::{KdTree, PairFirstOnly, Point};
use gif_compressor::undither::undither;
use std::borrow::Cow;
use std::collections::{BinaryHeap, HashMap};
use std::env;
use std::fs::File;
use std::time::Instant;
fn main() {
    let args: Vec<String> = env::args().collect();
    let output_name = &args[2];
    let mut colour_freq = HashMap::new();
    let mut width = 0;
    let mut height = 0;
    let (global_cache, local_cache) = read_and_process_gif(
        &args,
        |frame, _| {
            if width == 0 || height == 0 {
                width = frame.canvas_width();
                height = frame.canvas_height();
            }
            for i in 0..height {
                for j in 0..width {
                    let cur = frame.canvas[i][j];
                    if cur.transparent {
                        continue;
                    }
                    colour_freq.entry(cur).or_insert(0);
                    *colour_freq.get_mut(&cur).unwrap() += 1;
                }
            }
        },
        None,
        Vec::new(),
    );
    let mut flat_freq: Vec<(RGB, usize)> = colour_freq.into_iter().collect();
    let new_palette = median_cut(&mut flat_freq, 255);
    let palette_formatted: Vec<u8> = new_palette.iter().flat_map(|x| [x.r, x.g, x.b]).collect();
    let mut index_map = HashMap::new();
    assert!(new_palette.len() <= 255);
    let transparent_index = new_palette.len() as u8;
    new_palette.iter().enumerate().for_each(|(i, x)| {
        index_map.insert(*x, i as u8);
    });
    let new_palette_tree = KdTree::new(new_palette);
    index_map.insert(RGB::default(), transparent_index); //don't put this in kdtree
    let mut output = File::create(output_name).unwrap();
    let mut encoder =
        Encoder::new(&mut output, width as u16, height as u16, &palette_formatted).unwrap();
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();
    let mut is_first_frame = true;
    read_and_process_gif(
        &args,
        |frame, prev_canvas| {
            let mut indices: Vec<u8> = Vec::with_capacity(width * height);
            for i in 0..height {
                for j in 0..width {
                    let cur = frame.canvas[i][j];
                    let best = new_palette_tree.k_nn(cur, 1)[0];
                    frame.canvas[i][j] = best;
                }
            }
            let (mut top, mut left, mut local_height, mut local_width) = (0, 0, height, width);
            if !is_first_frame {
                (top, left, local_height, local_width) = fuzzy_transparency(frame, prev_canvas);
            }
            for i in 0..local_height {
                for j in 0..local_width {
                    let cur = frame.canvas[top + i][left + j];
                    if cur.transparent {
                        indices.push(transparent_index);
                    } else {
                        indices.push(index_map[&cur]);
                    }
                }
            }
            let frame_output = Frame {
                width: local_width as u16,
                height: local_height as u16,
                top: top as u16,
                left: left as u16,
                buffer: Cow::Borrowed(&indices),
                dispose: DisposalMethod::Keep,
                transparent: Some(transparent_index),
                delay: frame.delay,
                ..Default::default()
            };
            encoder.write_frame(&frame_output).unwrap();
            is_first_frame = false;
        },
        global_cache,
        local_cache,
    );
}
fn read_and_process_gif<F>(
    args: &[String],
    mut on_frame_processed: F,
    global_palette_cached: Option<Palette>,
    local_palettes_cached: Vec<Option<Palette>>,
) -> (Option<Palette>, Vec<Option<Palette>>)
where
    F: FnMut(&mut GifFrame, &Vec<Vec<RGB>>),
{
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let file = File::open(&args[1]).unwrap();
    let decoder = decoder.read_info(file).unwrap();
    let start = Instant::now();
    let height = decoder.height() as usize;
    let width = decoder.width() as usize;
    if width == 0 || height == 0 {
        panic!("malformed gif: width or height is 0");
    }
    let global_palette =
        global_palette_cached.or_else(|| decoder.global_palette().map(Palette::new));
    let mut canvas =
        vec![vec![RGB::transparent(); decoder.width() as usize]; decoder.height() as usize]; //reused
    let mut prev_canvas = canvas.clone();
    let mut decoder_iter = decoder.into_iter();
    let mut local_palettes = Vec::new();
    let mut local_palette_cache_iter = if !local_palettes_cached.is_empty() {
        Some(local_palettes_cached.into_iter())
    } else {
        None
    };
    while let Some(Ok(frame_raw)) = decoder_iter.next() {
        let mut frame = GifFrame::render_frame_to_canvas(
            &frame_raw,
            &mut canvas,
            global_palette.as_ref(),
            local_palette_cache_iter
                .as_mut()
                .and_then(|x| x.next().unwrap()),
        );
        undither(&mut frame);
        fuzzy_transparency(&mut frame, &prev_canvas);
        on_frame_processed(&mut frame, &prev_canvas);
        //drop frame here
        let prev_palette = frame.into_local_palette();
        for i in 0..height {
            for j in 0..width {
                canvas[i][j] = match frame_raw.dispose {
                    DisposalMethod::Any | DisposalMethod::Keep => canvas[i][j],
                    DisposalMethod::Background => RGB::transparent(),
                    DisposalMethod::Previous => prev_canvas[i][j],
                }
            }
        }
        prev_canvas = canvas.clone();
        local_palettes.push(prev_palette);
    }
    println!("took {:?}", start.elapsed());
    (global_palette, local_palettes)
}
/// returns (top_i,left_i,height,width) of smallest bounding rect of all opaque pixels
fn fuzzy_transparency(
    frame: &mut GifFrame,
    prev_canvas: &[Vec<RGB>],
) -> (usize, usize, usize, usize) {
    let height = frame.canvas_height();
    let width = frame.canvas_width();
    let threshold = 5;
    let mut max_i = 0;
    let mut min_i = height - 1;
    let mut max_j = 0;
    let mut min_j = width - 1;
    for i in 0..height {
        for j in 0..width {
            let cur = frame.canvas[i][j];
            let prev = prev_canvas[i][j];
            if cur.distance_luma_sq(prev) < threshold * threshold {
                frame.canvas[i][j].transparent = true;
            } else {
                max_i = max_i.max(i);
                min_i = min_i.min(i);
                max_j = max_j.max(j);
                min_j = min_j.min(j);
            }
        }
    }
    max_i = max_i.max(min_i);
    max_j = max_j.max(min_j);
    (min_i, min_j, max_i - min_i + 1, max_j - min_j + 1)
}

///lst:(RGB, freq)
fn median_cut(lst: &mut [(RGB, usize)], max_n: usize) -> Vec<RGB> {
    if lst.len() <= max_n {
        return lst.iter().map(|x| x.0).collect();
    }
    type MaxRangeAndDim = (usize, u8);
    let mut pq: BinaryHeap<PairFirstOnly<MaxRangeAndDim, &mut [(RGB, usize)]>> = BinaryHeap::new();
    fn calc_max_range(lst: &[(RGB, usize)]) -> (usize, u8) {
        let (mut mn_r, mut mn_g, mut mn_b) = (255_usize, 255_usize, 255_usize);
        let (mut mx_r, mut mx_g, mut mx_b) = (0_usize, 0_usize, 0_usize);
        for (x, _) in lst {
            mn_r = mn_r.min(x.r as usize);
            mx_r = mx_r.max(x.r as usize);
            mn_g = mn_g.min(x.g as usize);
            mx_g = mx_g.max(x.g as usize);
            mn_b = mn_b.min(x.b as usize);
            mx_b = mx_b.max(x.b as usize);
        }
        *[
            (mx_r - mn_r, 0_u8),
            (mx_g - mn_g, 1_u8),
            (mx_b - mn_b, 2_u8),
        ]
        .iter()
        .max()
        .unwrap()
    }
    pq.push(PairFirstOnly::new(calc_max_range(lst), lst));
    let mut ans = Vec::with_capacity(max_n);
    while !pq.is_empty() && (ans.len() + pq.len()) < max_n {
        let PairFirstOnly {
            first: (_, split_dim),
            second: slice,
        } = pq.pop().unwrap();
        if slice.len() == 1 {
            ans.push(slice[0].0);
            continue;
        }
        let mid = slice.len() / 2;
        slice.select_nth_unstable_by_key(mid, |x| x.0.get(split_dim as usize));
        let (left, right) = slice.split_at_mut(mid);
        if !left.is_empty() {
            pq.push(PairFirstOnly::new(calc_max_range(left), left));
        }
        if !right.is_empty() {
            pq.push(PairFirstOnly::new(calc_max_range(right), right));
        }
    }
    pq.into_iter().for_each(
        |PairFirstOnly {
             first: _,
             second: slice,
         }| {
            let (mut r_sum, mut g_sum, mut b_sum) = (0, 0, 0);
            let mut total = 0;
            for (rgb, freq) in &*slice {
                r_sum += freq * rgb.r as usize;
                g_sum += freq * rgb.g as usize;
                b_sum += freq * rgb.b as usize;
                total += freq;
            }
            ans.push(RGB::new(
                (r_sum / total) as u8,
                (g_sum / total) as u8,
                (b_sum / total) as u8,
            ));
        },
    );
    ans
}
