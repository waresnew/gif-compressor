use std::{borrow::Cow, fs::File};

use ahash::AHashMap;
use gif::{DisposalMethod, Encoder, Frame};

use crate::image::{GifFrame, Rgb};

pub struct GifWriter<'a, I: Iterator<Item = GifFrame>> {
    encoder: Encoder<&'a mut File>,
    frames: I,
    transparent_index: u8,
    index_map: AHashMap<Rgb, u8>,
    width: usize,
    height: usize,
}
impl<'a, I: Iterator<Item = GifFrame>> GifWriter<'a, I> {
    pub fn new(
        frames: I,
        palette: Vec<Rgb>,
        height: usize,
        width: usize,
        output_file: &'a mut File,
    ) -> Self {
        let palette_formatted: Vec<u8> = palette
            .iter()
            .flat_map(|x| [x.r, x.g, x.b])
            .chain([0, 0, 0]) //pad for transparent index, don't put in kdtree
            .collect();
        let mut encoder =
            Encoder::new(output_file, width as u16, height as u16, &palette_formatted).unwrap();
        encoder.set_repeat(gif::Repeat::Infinite).unwrap();
        assert!(palette.len() <= 255);
        let transparent_index = palette.len() as u8;
        let mut index_map = AHashMap::default();
        palette.iter().enumerate().for_each(|(i, x)| {
            index_map.insert(*x, i as u8);
        });
        Self {
            index_map,
            transparent_index,
            encoder,
            frames,
            width,
            height,
        }
    }
    pub fn write_frame(&mut self) -> bool {
        let Some(frame) = self.frames.next() else {
            return false;
        };
        let mut indices: Vec<u8> = Vec::with_capacity(self.width * self.height);

        for i in 0..frame.local_height {
            for j in 0..frame.local_width {
                let cur = frame.image.get(frame.top + i, frame.left + j);
                if cur.transparent {
                    indices.push(self.transparent_index);
                } else {
                    indices.push(self.index_map[&cur]);
                }
            }
        }
        let frame_output = Frame {
            width: frame.local_width as u16,
            height: frame.local_height as u16,
            top: frame.top as u16,
            left: frame.left as u16,
            buffer: Cow::Borrowed(&indices),
            dispose: DisposalMethod::Keep,
            transparent: Some(self.transparent_index),
            delay: frame.delay,
            ..Default::default()
        };
        self.encoder.write_frame(&frame_output).unwrap();
        true
    }
}
