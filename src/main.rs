use gif_compressor::cli::{Args, parse_args};
use gif_compressor::image::GifFrame;
use gif_compressor::nearest_neighbour::{ChosenNnSolver, NnSolver};
use gif_compressor::palette::gen_palette;
use gif_compressor::quantizer::get_quantize_transform;
use gif_compressor::reader::{GifFrameTransformer, GifReader};
use gif_compressor::transparency::get_transparency_transform;
use gif_compressor::undither::undither_frame;
use gif_compressor::writer::GifWriter;
use std::env;
use std::fs::File;
use std::time::Instant;
use std::vec::IntoIter;

enum GifFrameIterator {
    Vec(IntoIter<GifFrame>),
    GifReader(GifReader),
}
impl Iterator for GifFrameIterator {
    type Item = GifFrame;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GifFrameIterator::Vec(vec) => vec.next(),
            GifFrameIterator::GifReader(reader) => reader.next(),
        }
    }
}
fn create_reader(args: &Args) -> GifReader {
    let mut reader = GifReader::new(args.input.clone());
    reader.apply_transform(GifFrameTransformer::Default(Box::new(undither_frame)));
    reader.apply_transform(GifFrameTransformer::NeedsPrev(Box::new(
        get_transparency_transform(args.transparency_threshold),
    )));
    reader
}
fn main() {
    let start = Instant::now();
    let args = parse_args(env::args());

    let mut output_file = File::create(&args.output).unwrap();
    let reader = create_reader(&args);
    let height = reader.height();
    let width = reader.width();
    let (frames, palette) = if !args.stream {
        let frames = reader.collect::<Vec<GifFrame>>();
        let palette = gen_palette(frames.clone(), height, width);
        (GifFrameIterator::Vec(frames.into_iter()), palette)
    } else {
        let palette = gen_palette(reader, height, width);
        (GifFrameIterator::GifReader(create_reader(&args)), palette)
    };
    let nn_solver = ChosenNnSolver::new(palette.clone());
    let quantize = get_quantize_transform(nn_solver);
    let optimize_transparency = get_transparency_transform(args.transparency_threshold);
    let mut prev_frame = None;
    let frames = frames.map(|mut frame| {
        quantize(&mut frame);
        optimize_transparency(&mut frame, prev_frame.as_ref());
        if let Some(prev) = &mut prev_frame {
            for i in 0..height {
                for j in 0..width {
                    let cur = frame.image.get(i, j);
                    if !cur.transparent {
                        *prev.get_mut(i, j) = cur;
                    }
                }
            }
        } else {
            prev_frame = Some(frame.clone().image);
        }
        frame
    });
    let mut writer = GifWriter::new(frames, palette, height, width, &mut output_file);
    while writer.write_frame() {}
    println!(
        "finished in {:.1}s",
        start.elapsed().as_millis() as f32 / 1000.0
    );
}
