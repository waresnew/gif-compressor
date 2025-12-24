use std::ops::Index;

use gif::Frame;

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl RGB {
    pub fn average(&self, other: &RGB) -> RGB {
        RGB {
            r: ((self.r as u16 + other.r as u16) / 2) as u8,
            g: ((self.g as u16 + other.g as u16) / 2) as u8,
            b: ((self.b as u16 + other.b as u16) / 2) as u8,
        }
    }
    pub fn distance_sq(&self, other: &RGB) -> u32 {
        let dr = self.r as i32 - other.r as i32;
        let dg = self.g as i32 - other.g as i32;
        let db = self.b as i32 - other.b as i32;
        (dr * dr + db * db + dg * dg) as u32
    }
    pub fn as_luminance(&self) -> u8 {
        //Y component in YCbCr
        (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) as u8
    }
}
#[derive(Debug)]
pub struct Palette {
    palette: Vec<RGB>,
}
impl Palette {
    pub fn from_raw(palette_raw: &[u8]) -> Self {
        Palette {
            palette: palette_raw
                .chunks_exact(3)
                .map(|c| RGB {
                    r: c[0],
                    g: c[1],
                    b: c[2],
                })
                .collect(),
        }
    }
    pub fn new(palette: Vec<RGB>) -> Self {
        Palette { palette }
    }
    pub fn nearest(&self, target: &RGB, exclude1: &RGB, exclude2: &RGB) -> &RGB {
        //OPTIMIZE:brute force
        let mut ans = &self.palette[0];
        let mut best_dis = 1000000;
        for c in &self.palette {
            if c == exclude1 || c == exclude2 {
                continue;
            }
            let dis = c.distance_sq(target);
            if dis < best_dis {
                ans = c;
                best_dis = dis;
            }
        }
        ans
    }
}
impl Index<u8> for Palette {
    type Output = RGB;

    fn index(&self, index: u8) -> &Self::Output {
        &self.palette[(index as usize).clamp(0, self.palette.len() - 1)]
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
    /// draws `frame` onto `canvas`
    pub fn render_to_canvas(
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
        dbg!(top);
        dbg!(left);
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
            local_palette: frame.palette.as_ref().map(|local| Palette::from_raw(local)),
            canvas,
        }
    }
    pub fn get_palette(&self) -> &Palette {
        self.local_palette
            .as_ref()
            .or(self.global_palette)
            .expect("malformed gif: no global or local palette")
    }
}
