use clap::Parser;
use gif_compressor::cli::Cli;
use gif_compressor::image::{GifFrame, Rgb};
use gif_compressor::nearest_neighbour::{ChosenNnSolver, NnSolver};
use gif_compressor::palette::gen_palette;
use gif_compressor::quantizer::quantize;
use gif_compressor::reader::GifReader;
use gif_compressor::transparency::TransparencyOptimizer;
use gif_compressor::undither::undither_frame;
use gif_compressor::writer::GifWriter;
use std::fs::File;
use std::time::Instant;

fn create_reader(args: &Cli) -> GifReader {
    let mut reader = GifReader::new(args.input.clone());
    reader.apply_transform(undither_frame);
    reader
}
type FrameIter = Box<dyn Iterator<Item = GifFrame>>;
fn main() {
    let start = Instant::now();
    let args = Cli::parse();

    let mut output_file = File::create(&args.output).unwrap();
    let reader = create_reader(&args);
    let height = reader.height();
    let width = reader.width();
    let (frames, palette): (FrameIter, Vec<Rgb>) = if !args.stream {
        let mut transparency = TransparencyOptimizer::new(args.transparency_threshold);
        let frames = reader.collect::<Vec<GifFrame>>();
        let palette = gen_palette(
            frames.clone().into_iter().map(|mut frame| {
                transparency.apply_transparency(&mut frame);
                frame
            }),
            height,
            width,
        );
        (Box::new(frames.into_iter()), palette)
    } else {
        let mut transparency = TransparencyOptimizer::new(args.transparency_threshold);
        let palette = gen_palette(
            reader.map(|mut frame| {
                transparency.apply_transparency(&mut frame);
                frame
            }),
            height,
            width,
        );
        (Box::new(create_reader(&args)), palette)
    };
    let mut transparency_pre_quantize = TransparencyOptimizer::new(args.transparency_threshold);
    let mut transparency_post_quantize = TransparencyOptimizer::new(args.transparency_threshold);
    let mut nn_solver = ChosenNnSolver::new(palette.clone());
    let frames = frames.map(|mut frame| {
        transparency_pre_quantize.apply_transparency(&mut frame);
        quantize(&mut frame, &mut nn_solver);
        transparency_post_quantize.apply_transparency(&mut frame);
        frame
    });
    let mut writer = GifWriter::new(frames, palette, height, width, &mut output_file);
    while writer.write_frame() {}
    println!(
        "finished in {:.1}s",
        start.elapsed().as_millis() as f32 / 1000.0
    );
}
