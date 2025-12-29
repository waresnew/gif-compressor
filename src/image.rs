use std::hash::Hash;
use std::{cmp::Ordering, hash::Hasher};

use gif::Frame;

use crate::kdtree::{KdTree, Point};

#[derive(Debug, Clone, Copy, Default)]
///transparent field is purely a marker; ignored in ord/eq/hash
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub transparent: bool,
}
impl Hash for RGB {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.r.hash(state);
        self.g.hash(state);
        self.b.hash(state);
    }
}
impl PartialEq for RGB {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
}
impl Eq for RGB {}
impl PartialOrd for RGB {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RGB {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.r.cmp(&other.r) {
            Ordering::Equal => match self.g.cmp(&other.g) {
                Ordering::Equal => self.b.cmp(&other.b),
                x => x,
            },
            x => x,
        }
    }
}
impl Point<3> for RGB {
    fn get(&self, dim: usize) -> i32 {
        if dim == 0 {
            self.r as i32
        } else if dim == 1 {
            self.g as i32
        } else if dim == 2 {
            self.b as i32
        } else {
            unreachable!("RGB Point get() given dim=={}", dim);
        }
    }
}
pub const RGB_TRANSPARENT: RGB = RGB {
    r: 0,
    g: 0,
    b: 0,
    transparent: true,
};
impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            ..Default::default()
        }
    }
    pub fn average(&self, other: RGB) -> RGB {
        RGB::new(
            ((self.r as u16 + other.r as u16) / 2) as u8,
            ((self.g as u16 + other.g as u16) / 2) as u8,
            ((self.b as u16 + other.b as u16) / 2) as u8,
        )
    }
    pub fn distance_sq(&self, other: RGB) -> u32 {
        let dr = self.r as i32 - other.r as i32;
        let dg = self.g as i32 - other.g as i32;
        let db = self.b as i32 - other.b as i32;
        (dr * dr + db * db + dg * dg) as u32
    }
    pub fn distance_luma_sq(&self, other: RGB) -> u32 {
        let dr = self.r as f32 - other.r as f32;
        let dg = self.g as f32 - other.g as f32;
        let db = self.b as f32 - other.b as f32;
        (0.299 * dr * dr + 0.587 * dg * dg + 0.114 * db * db) as u32
    }
    pub fn as_luma(&self) -> u8 {
        //Y component in YCbCr
        (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) as u8
    }
}
#[derive(Debug)]
pub struct Palette {
    kdtree: KdTree<RGB, 3>,
}
impl Palette {
    pub fn new(palette_raw: &[u8]) -> Self {
        let mut palette: Vec<RGB> = palette_raw
            .chunks_exact(3)
            .map(|c| RGB::new(c[0], c[1], c[2]))
            .collect();
        palette.sort();
        palette.dedup();
        Self {
            kdtree: KdTree::new(palette),
        }
    }
    #[inline]
    pub fn get_nearest(&self, target: RGB, exclude1: RGB, exclude2: RGB) -> Option<RGB> {
        let res = self.kdtree.k_nn(target, 3);
        if let Some(res) = res {
            for &rgb in &*res {
                if rgb != exclude1 && rgb != exclude2 {
                    return Some(rgb);
                }
            }
            unreachable!();
        } else {
            None
        }
    }
}
#[derive(Clone, Debug)]
/// access a 1d vec in a 2d manner
pub struct Canvas {
    pub buffer: Vec<RGB>,
    pub height: usize,
    pub width: usize,
}
impl Canvas {
    pub fn blank(height: usize, width: usize) -> Self {
        Self {
            buffer: vec![RGB_TRANSPARENT; height * width],
            height,
            width,
        }
    }
    pub fn get(&self, i: usize, j: usize) -> RGB {
        self.buffer[self.width * i + j]
    }
    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut RGB {
        &mut self.buffer[self.width * i + j]
    }
}

///fully composited gif frame
pub struct GifFrame<'a> {
    pub canvas: &'a mut Canvas,
    global_palette: Option<&'a Palette>,
    local_palette: Option<Palette>,
    pub delay: u16,
}
impl<'a> GifFrame<'a> {
    pub fn canvas_width(&self) -> usize {
        self.canvas.width
    }
    pub fn canvas_height(&self) -> usize {
        self.canvas.height
    }
    pub fn render_frame_to_canvas(
        frame: &Frame,
        canvas: &'a mut Canvas,
        global_palette: Option<&'a Palette>,
    ) -> Self {
        let pixels_raw: Vec<(u8, u8, u8, u8)> = frame
            .buffer
            .chunks_exact(4)
            .map(|c| (c[0], c[1], c[2], c[3]))
            .collect();
        let (top, left) = (frame.top as usize, frame.left as usize);
        let (height, width) = (frame.height as usize, frame.width as usize);
        for i in 0..height {
            for j in 0..width {
                let (r, g, b, a) = pixels_raw[i * width + j];
                if a == 0 {
                    continue;
                }
                *canvas.get_mut(top + i, left + j) = RGB::new(r, g, b);
            }
        }

        Self {
            global_palette,
            local_palette: frame.palette.as_ref().map(|local| Palette::new(local)),
            canvas,
            delay: frame.delay,
        }
    }
    pub fn get_palette(&self) -> &Palette {
        if let Some(local) = &self.local_palette {
            local
        } else if let Some(global) = self.global_palette {
            global
        } else {
            panic!("malformed gif: no global or local palette");
        }
    }
}
