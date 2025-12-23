use std::ops::Index;

use gif::Frame;

#[derive(Debug, Default, Clone, Copy)]
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
    pub fn new(palette: Vec<RGB>) -> Self {
        Palette { palette }
    }
    pub fn nearest(&self, target: &RGB, exclude1: u8, exclude2: u8) -> &RGB {
        //OPTIMIZE:brute force
        let mut ans = &self.palette[0];
        let mut best_dis = 1000000;
        for (i, c) in self.palette.iter().enumerate() {
            if i == exclude1 as usize || i == exclude2 as usize {
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
pub struct GifFrame {
    pub width: usize,
    pub height: usize,
    pub indices: Vec<Vec<u8>>,
    pub transparent: Option<u8>,
}
impl GifFrame {
    pub fn new(frame: &Frame) -> Self {
        assert_eq!(
            frame.width as usize * frame.height as usize,
            frame.buffer.len()
        );
        let indices: Vec<Vec<u8>> = frame
            .buffer
            .chunks_exact(frame.width as usize)
            .map(|c| c.to_vec())
            .collect();
        Self {
            width: frame.width as usize,
            height: frame.height as usize,
            indices,
            transparent: frame.transparent,
        }
    }
}
