use crate::image::{GifFrame, Image};

fn apply_transparency(frame: &mut GifFrame, prev_frame: &Image, threshold: u32) {
    let image = &mut frame.image;
    let height = image.height;
    let width = image.width;
    let mut max_i = 0;
    let mut min_i = height - 1;
    let mut max_j = 0;
    let mut min_j = width - 1;
    for i in 0..height {
        for j in 0..width {
            let cur = image.get(i, j);
            let prev = prev_frame.get(i, j);
            if cur.transparent || cur.distance_luma_sq(prev) < threshold * threshold {
                image.get_mut(i, j).transparent = true;
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
    frame.top = min_i;
    frame.left = min_j;
    frame.local_height = max_i - min_i + 1;
    frame.local_width = max_j - min_j + 1;
}

pub fn get_transparency_transform(threshold: u32) -> impl Fn(&mut GifFrame, Option<&Image>) {
    move |frame, prev| {
        let Some(prev) = prev else {
            return;
        };
        apply_transparency(frame, prev, threshold);
    }
}
