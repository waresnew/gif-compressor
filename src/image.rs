use std::hash::Hash;
use std::{cmp::Ordering, hash::Hasher};

#[derive(Debug, Clone, Copy, Default)]
///transparent field is purely a marker; ignored in ord/eq/hash
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub transparent: bool,
}
impl Hash for Rgb {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.r.hash(state);
        self.g.hash(state);
        self.b.hash(state);
    }
}
impl PartialEq for Rgb {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
}
impl Eq for Rgb {}
impl PartialOrd for Rgb {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Rgb {
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
pub const RGB_TRANSPARENT: Rgb = Rgb {
    r: 0,
    g: 0,
    b: 0,
    transparent: true,
};
impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            ..Default::default()
        }
    }
    pub fn get(&self, dim: usize) -> i32 {
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
    pub fn average(&self, other: Rgb) -> Rgb {
        Rgb::new(
            ((self.r as u16 + other.r as u16) / 2) as u8,
            ((self.g as u16 + other.g as u16) / 2) as u8,
            ((self.b as u16 + other.b as u16) / 2) as u8,
        )
    }
    pub fn distance_sq(&self, other: Rgb) -> u32 {
        let dr = self.r as i32 - other.r as i32;
        let dg = self.g as i32 - other.g as i32;
        let db = self.b as i32 - other.b as i32;
        (dr * dr + db * db + dg * dg) as u32
    }
    pub fn distance_luma_sq(&self, other: Rgb) -> u32 {
        let dr = self.r as f32 - other.r as f32;
        let dg = self.g as f32 - other.g as f32;
        let db = self.b as f32 - other.b as f32;
        (0.299 * dr * dr + 0.587 * dg * dg + 0.114 * db * db) as u32
    }
    pub fn as_luma(&self) -> u8 {
        (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) as u8
    }
}
#[derive(Clone, Debug)]
pub struct Image {
    pub buffer: Vec<Rgb>,
    pub height: usize,
    pub width: usize,
}
impl Image {
    pub fn blank(height: usize, width: usize) -> Self {
        Self {
            buffer: vec![RGB_TRANSPARENT; height * width],
            height,
            width,
        }
    }
    pub fn get(&self, i: usize, j: usize) -> Rgb {
        self.buffer[self.width * i + j]
    }
    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut Rgb {
        &mut self.buffer[self.width * i + j]
    }
}

#[derive(Clone)]
pub struct GifFrame {
    pub image: Image,
    pub palette: Vec<Rgb>,
    pub delay: u16,
    pub top: usize,
    pub left: usize,
    pub local_height: usize,
    pub local_width: usize,
}
impl GifFrame {
    pub fn new(image: Image, palette: Vec<Rgb>, delay: u16) -> Self {
        Self {
            top: 0,
            left: 0,
            local_height: image.height,
            local_width: image.width,
            image,
            palette,
            delay,
        }
    }
}
