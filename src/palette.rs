use std::collections::{BTreeSet, BinaryHeap};

use crate::image::{GifFrame, Rgb};

pub fn gen_palette(
    iter: impl IntoIterator<Item = GifFrame>,
    height: usize,
    width: usize,
) -> Vec<Rgb> {
    let mut colour_freq = BTreeSet::default(); //not hashset for into_iter() determinism
    for frame in iter {
        for i in 0..height {
            for j in 0..width {
                let cur = frame.image.get(i, j);
                if cur.transparent {
                    continue;
                }
                colour_freq.insert(cur);
            }
        }
    }
    median_cut(&mut colour_freq.into_iter().collect::<Vec<Rgb>>(), 255)
}
/// frequency-blind
fn median_cut(lst: &mut [Rgb], max_n: usize) -> Vec<Rgb> {
    if lst.len() <= max_n {
        return lst.to_vec();
    }
    type MaxRangeAndDim = (usize, u8); //what was the max range, and which dim did it correspond to
    let mut pq: BinaryHeap<(MaxRangeAndDim, &mut [Rgb])> = BinaryHeap::new();
    fn calc_max_range(lst: &[Rgb]) -> (usize, u8) {
        let (mut mn_r, mut mn_g, mut mn_b) = (255_usize, 255_usize, 255_usize);
        let (mut mx_r, mut mx_g, mut mx_b) = (0_usize, 0_usize, 0_usize);
        for x in lst {
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
    pq.push((calc_max_range(lst), lst));
    let mut ans = Vec::with_capacity(max_n);
    while !pq.is_empty() && (ans.len() + pq.len()) < max_n {
        let ((_, split_dim), slice) = pq.pop().unwrap();
        if slice.len() == 1 {
            ans.push(slice[0]);
            continue;
        }
        let mid = slice.len() / 2; //unique colour prio
        slice.select_nth_unstable_by_key(mid, |x| x.get(split_dim as usize));
        let (left, right) = slice.split_at_mut(mid);
        if !left.is_empty() {
            pq.push((calc_max_range(left), left));
        }
        if !right.is_empty() {
            pq.push((calc_max_range(right), right));
        }
    }
    pq.into_iter().for_each(|(_, slice)| {
        let (mut r_sum, mut g_sum, mut b_sum) = (0, 0, 0);
        for rgb in &*slice {
            r_sum += rgb.r as usize;
            g_sum += rgb.g as usize;
            b_sum += rgb.b as usize;
        }
        let total = slice.len();
        ans.push(Rgb::new(
            (r_sum / total) as u8,
            (g_sum / total) as u8,
            (b_sum / total) as u8,
        ));
    });
    ans
}
