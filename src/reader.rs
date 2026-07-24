use std::fs::File;

use gif::{Decoder, DisposalMethod};

use crate::image::{GifFrame, Image, RGB_TRANSPARENT, Rgb};

pub struct GifReader {
    height: usize,
    width: usize,
    global_palette: Option<Vec<Rgb>>,
    prev_frame: Option<Image>,
    decoder_iter: <Decoder<File> as IntoIterator>::IntoIter,
    transforms: Vec<fn(&mut GifFrame)>,
}
impl GifReader {
    pub fn new(input: String) -> Self {
        let decoder = make_decoder(input);
        let height = decoder.height() as usize;
        let width = decoder.width() as usize;
        if width == 0 || height == 0 {
            panic!("malformed gif: width or height is 0");
        }
        let global_palette = decoder.global_palette().map(parse_palette);
        let decoder_iter = decoder.into_iter();
        Self {
            height,
            width,
            prev_frame: None,
            global_palette,
            decoder_iter,
            transforms: Vec::new(),
        }
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    /// these transforms are different than just map()'ing self bc these will influence self.prev_frame
    pub fn apply_transform(&mut self, f: fn(&mut GifFrame)) {
        self.transforms.push(f);
    }
}
impl Iterator for GifReader {
    type Item = GifFrame;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(Ok(frame_raw)) = self.decoder_iter.next() else {
            return None;
        };
        let mut new_frame = self
            .prev_frame
            .clone() //TODO: check all clones used in this program
            .unwrap_or(Image::blank(self.height, self.width));
        let pixels_raw: Vec<(u8, u8, u8, u8)> = frame_raw
            .buffer
            .chunks_exact(4)
            .map(|c| (c[0], c[1], c[2], c[3]))
            .collect();
        let (top, left) = (frame_raw.top as usize, frame_raw.left as usize);
        let (height, width) = (frame_raw.height as usize, frame_raw.width as usize);
        for i in 0..height {
            for j in 0..width {
                let (r, g, b, a) = pixels_raw[i * width + j];
                if a == 0 {
                    continue;
                }
                *new_frame.get_mut(top + i, left + j) = Rgb::new(r, g, b);
            }
        }

        let palette = if let Some(local) = frame_raw.palette {
            parse_palette(local.as_slice())
        } else if let Some(global) = &self.global_palette {
            global.clone() //PERF: is this fine?
        } else {
            panic!("malformed gif: no global or local palette");
        };
        let mut frame = GifFrame::new(new_frame, palette, frame_raw.delay);
        for transform in &self.transforms {
            transform(&mut frame);
        }
        let mut new_prev = frame.clone();
        for i in 0..self.height {
            for j in 0..self.width {
                *new_prev.image.get_mut(i, j) = match frame_raw.dispose {
                    DisposalMethod::Any | DisposalMethod::Keep => new_prev.image.get(i, j),
                    DisposalMethod::Background => RGB_TRANSPARENT,
                    DisposalMethod::Previous => self
                        .prev_frame
                        .as_ref()
                        .unwrap_or(&Image::blank(self.height, self.width))
                        .get(i, j),
                }
            }
        }
        self.prev_frame = Some(new_prev.image);
        Some(frame)
    }
}
fn make_decoder(file_name: String) -> Decoder<File> {
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let file = File::open(file_name).unwrap();
    decoder.read_info(file).unwrap()
}
fn parse_palette(palette_raw: &[u8]) -> Vec<Rgb> {
    let mut palette: Vec<Rgb> = palette_raw
        .chunks_exact(3)
        .map(|c| Rgb::new(c[0], c[1], c[2]))
        .collect();
    palette.sort();
    palette.dedup();
    palette
}
