use clap::Parser;
use gif_compressor::image::GifFrame;
use gif_compressor::quantizer;
use gif_compressor::reader::GifReader;
use gif_compressor::transparency::TransparencyOptimizer;
use gif_compressor::writer::GifWriter;
use gif_compressor::{palette, undither};
use log::info;
use std::fs::File;
use std::time::Instant;

use crate::cli::Cli;

mod cli;

type FrameIter = Box<dyn Iterator<Item = GifFrame>>;
struct GenUnditheredFramesOutput {
    first_pass: FrameIter,
    second_pass: FrameIter,
    height: usize,
    width: usize,
}

fn main() {
    let start = Instant::now();
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbosity.log_level_filter())
        .init();

    let GenUnditheredFramesOutput {
        first_pass,
        second_pass,
        height,
        width,
    } = gen_undithered_frames(cli.input, cli.stream);

    let palette = palette::gen_palette(first_pass, height, width);

    let quantized_frames = quantizer::quantize_frames(second_pass, palette.clone());

    let mut transparency = TransparencyOptimizer::new(cli.transparency_threshold);
    let transparency_optimized = transparency.apply_transparency_all(quantized_frames);

    let mut output_file = File::create(&cli.output).unwrap();
    let mut writer = GifWriter::new(
        transparency_optimized,
        palette,
        height,
        width,
        &mut output_file,
    );
    while writer.write_frame() {}
    info!(
        "finished in {:.1}s",
        start.elapsed().as_millis() as f32 / 1000.0
    );
}

/// handles the stream option
fn gen_undithered_frames(input: String, stream: bool) -> GenUnditheredFramesOutput {
    if stream {
        let reader = GifReader::new(input);
        let height = reader.height();
        let width = reader.width();
        let frames = undither::undither_frames(reader, stream).collect::<Vec<GifFrame>>();
        GenUnditheredFramesOutput {
            first_pass: Box::new(frames.clone().into_iter()),
            second_pass: Box::new(frames.into_iter()),
            height,
            width,
        }
    } else {
        let reader1 = GifReader::new(input.clone());
        let reader2 = GifReader::new(input);
        let height = reader1.height();
        let width = reader1.width();
        GenUnditheredFramesOutput {
            first_pass: Box::new(undither::undither_frames(reader1, stream)),
            second_pass: Box::new(undither::undither_frames(reader2, stream)),
            height,
            width,
        }
    }
}
