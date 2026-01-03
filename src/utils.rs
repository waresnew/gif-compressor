use std::{borrow::Cow, fs::File};

use gif::{Decoder, DisposalMethod, Encoder, Frame};
use rustc_hash::FxHashMap;

use crate::{
    cli::Args,
    image::{Canvas, GifFrame, Palette, RGB, RGB_TRANSPARENT},
    kdtree::KdTree,
    undither::undither_frame,
};

pub struct UnditheredIter<'a> {
    pub height: usize,
    pub width: usize,
    global_palette: Option<Palette>,
    prev_canvas: Canvas,
    is_first_frame: bool,
    args: &'a Args,
    decoder_iter: <Decoder<File> as IntoIterator>::IntoIter,
}
impl<'a> UnditheredIter<'a> {
    pub fn new(args: &'a Args) -> Self {
        let decoder = make_decoder(&args.input);
        let height = decoder.height() as usize;
        let width = decoder.width() as usize;
        if width == 0 || height == 0 {
            panic!("malformed gif: width or height is 0");
        }
        let global_palette = decoder.global_palette().map(Palette::new);
        let decoder_iter = decoder.into_iter();
        Self {
            height,
            width,
            global_palette,
            prev_canvas: Canvas::blank(height, width),
            is_first_frame: true,
            decoder_iter,
            args,
        }
    }
}
impl<'a> Iterator for UnditheredIter<'a> {
    type Item = GifFrame;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(Ok(frame_raw)) = self.decoder_iter.next() else {
            return None;
        };
        let mut frame = GifFrame::render_frame_to_canvas(&frame_raw, &self.prev_canvas);
        let palette = if let Some(local) = frame_raw.palette {
            &Palette::new(&local)
        } else if let Some(global) = self.global_palette.as_ref() {
            global
        } else {
            panic!("malformed gif: no global or local palette");
        };
        undither_frame(&mut frame.canvas, palette);
        if !self.is_first_frame {
            apply_transparency(&mut frame.canvas, &self.prev_canvas, self.args);
        } else {
            self.is_first_frame = false;
        }
        let ret = frame.clone();
        for i in 0..self.height {
            for j in 0..self.width {
                *frame.canvas.get_mut(i, j) = match frame_raw.dispose {
                    DisposalMethod::Any | DisposalMethod::Keep => frame.canvas.get(i, j),
                    DisposalMethod::Background => RGB_TRANSPARENT,
                    DisposalMethod::Previous => self.prev_canvas.get(i, j),
                }
            }
        }
        self.prev_canvas = frame.canvas;
        Some(ret)
    }
}
fn make_decoder(file_name: &str) -> Decoder<File> {
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let file = File::open(file_name).unwrap();
    decoder.read_info(file).unwrap()
}
pub struct GifQuantizer<'a> {
    height: usize,
    width: usize,
    palette_tree: KdTree<RGB, 3>,
    palette_nn_cache: FxHashMap<RGB, Vec<RGB>>,
    is_first_frame: bool,
    index_map: FxHashMap<RGB, u8>,
    transparent_index: u8,
    encoder: Encoder<&'a mut File>,
    prev_canvas: Canvas,
    args: &'a Args,
}
impl<'a> GifQuantizer<'a> {
    pub fn new(
        output_file: &'a mut File,
        args: &'a Args,
        new_palette: Vec<RGB>,
        height: usize,
        width: usize,
    ) -> Self {
        let palette_formatted: Vec<u8> = new_palette
            .iter()
            .flat_map(|x| [x.r, x.g, x.b])
            .chain([0, 0, 0]) //pad for transparent index, don't put in kdtree
            .collect();
        let mut encoder =
            Encoder::new(output_file, width as u16, height as u16, &palette_formatted).unwrap();
        encoder.set_repeat(gif::Repeat::Infinite).unwrap();
        assert!(new_palette.len() <= 255);
        let transparent_index = new_palette.len() as u8;
        let mut index_map = FxHashMap::default();
        new_palette.iter().enumerate().for_each(|(i, x)| {
            index_map.insert(*x, i as u8);
        });
        Self {
            height,
            width,
            palette_tree: KdTree::new(new_palette),
            palette_nn_cache: FxHashMap::default(),
            is_first_frame: true,
            index_map,
            transparent_index,
            encoder,
            prev_canvas: Canvas::blank(height, width),
            args,
        }
    }
    pub fn write_frame(&mut self, mut frame: GifFrame) {
        let mut indices: Vec<u8> = Vec::with_capacity(self.width * self.height);
        for i in 0..self.height {
            for j in 0..self.width {
                let cur = frame.canvas.get(i, j);
                if cur.transparent {
                    continue;
                }
                let best = self
                    .palette_tree
                    .k_nn(cur, 1, &mut self.palette_nn_cache)
                    .unwrap()[0];
                *frame.canvas.get_mut(i, j) = best;
            }
        }

        let (mut top, mut left, mut local_height, mut local_width) =
            (0, 0, self.height, self.width);
        if !self.is_first_frame {
            (top, left, local_height, local_width) =
                apply_transparency(&mut frame.canvas, &self.prev_canvas, self.args);
        } else {
            self.is_first_frame = false;
        }

        for i in 0..local_height {
            for j in 0..local_width {
                let cur = frame.canvas.get(top + i, left + j);
                if cur.transparent {
                    indices.push(self.transparent_index);
                } else {
                    indices.push(self.index_map[&cur]);
                    *self.prev_canvas.get_mut(top + i, left + j) = cur;
                }
            }
        }
        let frame_output = Frame {
            width: local_width as u16,
            height: local_height as u16,
            top: top as u16,
            left: left as u16,
            buffer: Cow::Borrowed(&indices),
            dispose: DisposalMethod::Keep,
            transparent: Some(self.transparent_index),
            delay: frame.delay,
            ..Default::default()
        };
        self.encoder.write_frame(&frame_output).unwrap();
    }
}
/// returns (top_i,left_i,height,width) of smallest bounding rect of all opaque pixels
fn apply_transparency(
    canvas: &mut Canvas,
    prev_canvas: &Canvas,
    args: &Args,
) -> (usize, usize, usize, usize) {
    let height = canvas.height;
    let width = canvas.width;
    let mut max_i = 0;
    let mut min_i = height - 1;
    let mut max_j = 0;
    let mut min_j = width - 1;
    for i in 0..height {
        for j in 0..width {
            let cur = canvas.get(i, j);
            let prev = prev_canvas.get(i, j);
            if cur.transparent
                || cur.distance_luma_sq(prev)
                    < args.transparency_threshold * args.transparency_threshold
            {
                canvas.get_mut(i, j).transparent = true;
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
    (min_i, min_j, max_i - min_i + 1, max_j - min_j + 1)
}
