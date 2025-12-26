use gif::Frame;

use crate::kdtree::{KdTree, Point};

#[derive(Debug, Eq, PartialOrd, Ord, PartialEq, Default, Clone, Copy, Hash)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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
            panic!("RGB Point get() given dim=={}", dim);
        }
    }
}
impl RGB {
    pub fn average(&self, other: RGB) -> RGB {
        RGB {
            r: ((self.r as u16 + other.r as u16) / 2) as u8,
            g: ((self.g as u16 + other.g as u16) / 2) as u8,
            b: ((self.b as u16 + other.b as u16) / 2) as u8,
        }
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
    pub bg: Option<RGB>,
    kdtree: KdTree<RGB, 3>,
}
impl Palette {
    fn parse_raw_palette(palette_raw: &[u8]) -> Vec<RGB> {
        palette_raw
            .chunks_exact(3)
            .map(|c| RGB {
                r: c[0],
                g: c[1],
                b: c[2],
            })
            .collect()
    }
    pub fn new(palette_raw: &[u8], bg_index: Option<usize>) -> Self {
        let mut palette = Self::parse_raw_palette(palette_raw);
        palette.sort();
        palette.dedup();
        let bg = bg_index.map(|i| palette[i]);
        Self {
            kdtree: KdTree::new(palette),
            bg,
        }
    }
    pub fn get_nearest(&self, target: RGB, exclude1: RGB, exclude2: RGB) -> RGB {
        let [res1, res2, res3]: [RGB; 3] =
            self.kdtree.k_nn(target, 3).as_slice().try_into().unwrap();
        if res1 == exclude1 || res1 == exclude2 {
            if res2 == exclude1 || res2 == exclude2 {
                res3
            } else {
                res2
            }
        } else {
            res1
        }
    }
}

///fully composited gif frame
pub struct GifFrame<'a> {
    pub canvas: &'a mut Vec<Vec<RGB>>,
    global_palette: Option<&'a Palette>,
    local_palette: Option<Palette>,
}
impl<'a> GifFrame<'a> {
    pub fn canvas_width(&self) -> usize {
        self.canvas[0].len()
    }
    pub fn canvas_height(&self) -> usize {
        self.canvas.len()
    }
    pub fn render_frame_to_canvas(
        frame: &Frame,
        canvas: &'a mut Vec<Vec<RGB>>,
        global_palette: Option<&'a Palette>,
    ) -> Self {
        let pixels_raw: Vec<Vec<(u8, u8, u8, u8)>> = frame
            .buffer
            .chunks_exact(4)
            .map(|c| (c[0], c[1], c[2], c[3]))
            .collect::<Vec<(u8, u8, u8, u8)>>()
            .chunks_exact(frame.width as usize)
            .map(|c| c.to_vec())
            .collect();
        let (top, left) = (frame.top as usize, frame.left as usize);
        let (height, width) = (frame.height as usize, frame.width as usize);
        for i in 0..height {
            for j in 0..width {
                let (r, g, b, a) = pixels_raw[i][j];
                if a == 0 {
                    continue;
                }
                canvas[top + i][left + j] = RGB { r, g, b };
            }
        }

        Self {
            global_palette,
            local_palette: frame
                .palette
                .as_ref()
                .map(|local| Palette::new(local, None)),
            canvas,
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
