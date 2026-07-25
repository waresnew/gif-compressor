use clap::Parser;
use gif_compressor::image::{GifFrame, Rgb};
use gif_compressor::palette;
use gif_compressor::quantizer;
use gif_compressor::reader::GifReader;
use gif_compressor::transparency::TransparencyOptimizer;
use gif_compressor::undither;
use gif_compressor::writer::GifWriter;
use gif_compressor::{ChosenNnSolver, NnSolver};
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
    env_logger::init();
    let start = Instant::now();
    let args = Cli::parse();

    let mut output_file = File::create(&args.output).unwrap();
    let GenUnditheredFramesOutput {
        first_pass,
        second_pass,
        height,
        width,
    } = gen_undithered_frames(args.input, args.stream);
    let palette = gen_undithered_palette(first_pass, height, width, args.transparency_threshold);
    let quantized_frames =
        quantize_with_transparency(second_pass, palette.clone(), args.transparency_threshold);
    let mut writer = GifWriter::new(quantized_frames, palette, height, width, &mut output_file);
    while writer.write_frame() {}
    println!(
        "finished in {:.1}s",
        start.elapsed().as_millis() as f32 / 1000.0
    );
}

fn create_undithered_reader(input: String) -> GifReader {
    let mut reader = GifReader::new(input);
    reader.apply_transform(undither::undither_frame);
    reader
}

/// handles the stream option
fn gen_undithered_frames(input: String, stream: bool) -> GenUnditheredFramesOutput {
    if stream {
        let reader = create_undithered_reader(input);
        let height = reader.height();
        let width = reader.width();
        let frames = reader.collect::<Vec<GifFrame>>();
        GenUnditheredFramesOutput {
            first_pass: Box::new(frames.clone().into_iter()),
            second_pass: Box::new(frames.into_iter()),
            height,
            width,
        }
    } else {
        let reader1 = create_undithered_reader(input.clone());
        let reader2 = create_undithered_reader(input);
        let height = reader1.height();
        let width = reader1.width();
        GenUnditheredFramesOutput {
            first_pass: Box::new(reader1),
            second_pass: Box::new(reader2),
            height,
            width,
        }
    }
}
fn gen_undithered_palette(
    frames: impl Iterator<Item = GifFrame>,
    height: usize,
    width: usize,
    transparency_threshold: u32,
) -> Vec<Rgb> {
    let mut transparency = TransparencyOptimizer::new(transparency_threshold);
    palette::gen_palette(
        frames.map(|mut frame| {
            transparency.apply_transparency(&mut frame);
            frame
        }),
        height,
        width,
    )
}
fn quantize_with_transparency(
    frames: impl Iterator<Item = GifFrame>,
    palette: Vec<Rgb>,
    threshold: u32,
) -> impl Iterator<Item = GifFrame> {
    let mut transparency_pre_quantize = TransparencyOptimizer::new(threshold);
    let mut transparency_post_quantize = TransparencyOptimizer::new(threshold);
    let mut nn_solver = ChosenNnSolver::new(palette);
    frames.map(move |mut frame| {
        transparency_pre_quantize.apply_transparency(&mut frame);
        quantizer::quantize(&mut frame, &mut nn_solver);
        transparency_post_quantize.apply_transparency(&mut frame);
        frame
    })
}
