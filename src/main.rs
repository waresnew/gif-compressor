use gif::{Decoder, DisposalMethod, Encoder, Frame};
use gif_compressor::cli::parse_args;
use gif_compressor::image::{Canvas, GifFrame, Palette, RGB, RGB_TRANSPARENT};
use gif_compressor::kdtree::{KdTree, PairFirstOnly, Point};
use gif_compressor::undither::undither_frame;
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::collections::{BTreeMap, BinaryHeap};
use std::env;
use std::fs::File;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let args = parse_args(env::args());
    let decoder0 = make_decoder(&args.input);
    let height = decoder0.height() as usize;
    let width = decoder0.width() as usize;
    let (new_palette, kept_frames) = calc_new_palette(decoder0, !args.stream);

    let palette_formatted: Vec<u8> = new_palette
        .iter()
        .flat_map(|x| [x.r, x.g, x.b])
        .chain([0, 0, 0]) //pad for transparent index, don't put in kdtree
        .collect();
    let mut index_map = FxHashMap::default();
    assert!(new_palette.len() <= 255);
    let transparent_index = new_palette.len() as u8;
    new_palette.iter().enumerate().for_each(|(i, x)| {
        index_map.insert(*x, i as u8);
    });
    let new_palette_tree = KdTree::new(new_palette);
    let mut palette_nn_cache = FxHashMap::default();

    let mut output = File::create(&args.output).unwrap();
    let mut encoder =
        Encoder::new(&mut output, width as u16, height as u16, &palette_formatted).unwrap();
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();
    let mut is_first_frame = true;
    let mut prev_canvas = Canvas::blank(height, width);
    let mut write_frame = |(mut canvas, delay): (Canvas, u16)| {
        let mut indices: Vec<u8> = Vec::with_capacity(width * height);
        for i in 0..height {
            for j in 0..width {
                let cur = canvas.get(i, j);
                if cur.transparent {
                    continue;
                }
                let best = new_palette_tree
                    .k_nn(cur, 1, &mut palette_nn_cache)
                    .unwrap()[0];
                *canvas.get_mut(i, j) = best;
            }
        }
        let (mut top, mut left, mut local_height, mut local_width) = (0, 0, height, width);
        if !is_first_frame {
            (top, left, local_height, local_width) = fuzzy_transparency(&mut canvas, &prev_canvas);
        }
        for i in 0..local_height {
            for j in 0..local_width {
                let cur = canvas.get(top + i, left + j);
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
            delay,
            ..Default::default()
        };
        encoder.write_frame(&frame_output).unwrap();
        is_first_frame = false;
        prev_canvas = canvas;
    };
    if args.stream {
        let decoder = make_decoder(&args.input);
        undither_all_stream(decoder, |frame| {
            write_frame((frame.canvas.clone(), frame.delay));
        });
    } else {
        kept_frames.unwrap().into_iter().for_each(write_frame);
    }
    println!("finished in {:?}", start.elapsed());
}
//1st pass
fn calc_new_palette(
    decoder: Decoder<File>,
    keep_frames: bool,
) -> (Vec<RGB>, Option<Vec<(Canvas, u16)>>) {
    let height = decoder.height() as usize;
    let width = decoder.width() as usize;
    let mut colour_freq = BTreeMap::default(); //not hashmap for into_iter() determinism
    let mut prev_canvas = Canvas::blank(height, width);
    let mut kept_frames = Vec::new();
    let mut is_first_frame = true;
    undither_all_stream(decoder, |frame| {
        let mut canvas = frame.canvas.clone();
        if !is_first_frame {
            fuzzy_transparency(&mut canvas, &prev_canvas);
        }
        for i in 0..height {
            for j in 0..width {
                let cur = canvas.get(i, j);
                if cur.transparent {
                    continue;
                }
                colour_freq.entry(cur).or_insert(0);
                *colour_freq.get_mut(&cur).unwrap() += 1;
            }
        }
        if keep_frames {
            kept_frames.push((canvas.clone(), frame.delay));
        }
        prev_canvas = canvas;
        is_first_frame = false;
    });
    (
        median_cut(
            &mut colour_freq.into_iter().collect::<Vec<(RGB, usize)>>(),
            255,
        ),
        if keep_frames { Some(kept_frames) } else { None },
    )
}
fn undither_all_stream<F>(decoder: Decoder<File>, mut post_undither: F)
where
    F: FnMut(&GifFrame), //ideally keep gifframe immutable to avoid affecting future frame calcs
{
    let height = decoder.height() as usize;
    let width = decoder.width() as usize;
    if width == 0 || height == 0 {
        panic!("malformed gif: width or height is 0");
    }
    let global_palette = decoder.global_palette().map(Palette::new);
    let mut canvas = Canvas::blank(height, width);
    let mut prev_canvas = canvas.clone();
    let mut decoder_iter = decoder.into_iter();
    while let Some(Ok(frame_raw)) = decoder_iter.next() {
        let mut frame =
            GifFrame::render_frame_to_canvas(&frame_raw, &mut canvas, global_palette.as_ref());
        undither_frame(&mut frame);
        post_undither(&frame);
        for i in 0..height {
            for j in 0..width {
                *canvas.get_mut(i, j) = match frame_raw.dispose {
                    DisposalMethod::Any | DisposalMethod::Keep => canvas.get(i, j),
                    DisposalMethod::Background => RGB_TRANSPARENT,
                    DisposalMethod::Previous => prev_canvas.get(i, j),
                }
            }
        }
        prev_canvas = canvas.clone();
    }
}
fn make_decoder(file_name: &str) -> Decoder<File> {
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let file = File::open(file_name).unwrap();
    decoder.read_info(file).unwrap()
}
/// returns (top_i,left_i,height,width) of smallest bounding rect of all opaque pixels
fn fuzzy_transparency(canvas: &mut Canvas, prev_canvas: &Canvas) -> (usize, usize, usize, usize) {
    let height = canvas.height;
    let width = canvas.width;
    let threshold = 5;
    let mut max_i = 0;
    let mut min_i = height - 1;
    let mut max_j = 0;
    let mut min_j = width - 1;
    for i in 0..height {
        for j in 0..width {
            let cur = canvas.get(i, j);
            let prev = prev_canvas.get(i, j);
            if cur.distance_luma_sq(prev) < threshold * threshold {
                canvas.get_mut(i, j).transparent = true;
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
        let mid = slice.len() / 2; //unique colour prio
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
