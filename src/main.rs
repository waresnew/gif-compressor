use gif_compressor::cli::{Args, parse_args};
use gif_compressor::image::{GifFrame, RGB};
use gif_compressor::kdtree::{PairFirstOnly, Point};
use gif_compressor::utils::{GifQuantizer, UnditheredIter};
use std::collections::{BTreeMap, BinaryHeap};
use std::env;
use std::fs::File;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let args = parse_args(env::args());
    let (new_palette, kept_frames, (height, width)) =
        calc_new_palette(UnditheredIter::new(&args), &args);

    let mut output_file = File::create(&args.output).unwrap();
    let mut gif_writer = GifQuantizer::new(&mut output_file, &args, new_palette, height, width);
    if args.stream {
        UnditheredIter::new(&args).for_each(|x| gif_writer.write_frame(x));
    } else {
        kept_frames
            .unwrap()
            .into_iter()
            .for_each(|x| gif_writer.write_frame(x));
    }
    println!(
        "finished in {:.1}s",
        start.elapsed().as_millis() as f32 / 1000.0
    );
}
fn calc_new_palette(
    iter: UnditheredIter,
    args: &Args,
) -> (Vec<RGB>, Option<Vec<GifFrame>>, (usize, usize)) {
    let mut colour_freq = BTreeMap::default(); //not hashmap for into_iter() determinism
    let mut kept_frames = Vec::new();
    let height = iter.height;
    let width = iter.width;
    for frame in iter {
        for i in 0..height {
            for j in 0..width {
                let cur = frame.canvas.get(i, j);
                if cur.transparent {
                    continue;
                }
                colour_freq.entry(cur).or_insert(0);
                *colour_freq.get_mut(&cur).unwrap() += 1;
            }
        }
        if !args.stream {
            kept_frames.push(frame.clone());
        }
    }
    (
        median_cut(
            &mut colour_freq.into_iter().collect::<Vec<(RGB, usize)>>(),
            255,
        ),
        if !args.stream {
            Some(kept_frames)
        } else {
            None
        },
        (height, width),
    )
}

///lst:(RGB, freq)
pub fn median_cut(lst: &mut [(RGB, usize)], max_n: usize) -> Vec<RGB> {
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
