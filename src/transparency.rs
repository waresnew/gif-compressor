use crate::image::{GifFrame, Image};

pub struct TransparencyOptimizer {
    prev_frame: Option<Image>,
    threshold: u32,
}
pub type TransparencyOutput = (GifFrame, Vec<bool>);
impl TransparencyOptimizer {
    pub fn new(threshold: u32) -> Self {
        Self {
            prev_frame: None,
            threshold,
        }
    }
    pub fn apply_transparency_all(
        &mut self,
        frames: impl Iterator<Item = GifFrame>,
    ) -> impl Iterator<Item = TransparencyOutput> {
        frames.map(|mut frame| {
            let transparent_pixels = self.apply_transparency_once(&mut frame);
            (frame, transparent_pixels)
        })
    }
    fn apply_transparency_once(&mut self, frame: &mut GifFrame) -> Vec<bool> {
        let mut transparent_pixels = vec![false; frame.image.width * frame.image.height];
        let Some(prev_frame) = &self.prev_frame else {
            self.prev_frame = Some(frame.clone().image);
            return transparent_pixels;
        };
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
                if cur.distance_luma_sq(prev) < self.threshold * self.threshold {
                    image.get_mut(i, j).r = prev.r;
                    image.get_mut(i, j).g = prev.g;
                    image.get_mut(i, j).b = prev.b;
                    transparent_pixels[i * width + j] = true;
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
        self.prev_frame = Some(frame.clone().image);
        transparent_pixels
    }
}
