use gif_compressor::cli::parse_args;
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

fn main() {
    let start = Instant::now();
    let args = parse_args(env::args());

    let mut output_file = File::create(&args.output).unwrap();
    let mut reader = GifReader::new(args.input.clone());
    let height = reader.height();
    let width = reader.width();
    reader.apply_transform(GifFrameTransformer::Default(Box::new(undither_frame)));
    reader.apply_transform(GifFrameTransformer::NeedsPrev(Box::new(
        get_transparency_transform(args.transparency_threshold),
    )));
    if !args.stream {
        let mut frames = reader.collect::<Vec<GifFrame>>();
        let palette = gen_palette(frames.clone(), height, width);
        let nn_solver = ChosenNnSolver::new(palette.clone());
        let quantize = get_quantize_transform(nn_solver);
        let optimize_transparency = get_transparency_transform(args.transparency_threshold);
        let mut prev_frame = None;
        for frame in &mut frames {
            quantize(frame);
            optimize_transparency(frame, prev_frame.as_ref());
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
        }
        let mut writer =
            GifWriter::new(frames.into_iter(), palette, height, width, &mut output_file);
        while writer.write_frame() {}
    } else {
        let palette = gen_palette(reader, height, width);
        let nn_solver = ChosenNnSolver::new(palette.clone());
        let mut reader = GifReader::new(args.input.clone());
        reader.apply_transform(GifFrameTransformer::Default(Box::new(undither_frame)));
        reader.apply_transform(GifFrameTransformer::NeedsPrev(Box::new(
            get_transparency_transform(args.transparency_threshold),
        )));
        let quantize = get_quantize_transform(nn_solver);
        let optimize_transparency = get_transparency_transform(args.transparency_threshold);
        let mut prev_frame = None;
        let frames = reader.map(|mut frame| {
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
    }
    println!(
        "finished in {:.1}s",
        start.elapsed().as_millis() as f32 / 1000.0
    );
}
