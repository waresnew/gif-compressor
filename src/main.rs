use clap::Parser;
use gif_compressor::chunked_iter::ChunkedIter;
use gif_compressor::image::GifFrame;
use gif_compressor::reader::GifReader;
use gif_compressor::transparency::TransparencyOptimizer;
use gif_compressor::writer::GifWriter;
use gif_compressor::{gpu, quantizer};
use gif_compressor::{palette, undither};
use log::info;
use std::fs::File;
use std::time::Instant;

use crate::cli::Cli;

mod cli;

struct GenUnditheredChunksOutput<I: Iterator<Item = Vec<GifFrame>>> {
    first_pass: I,
    second_pass: I,
    height: usize,
    width: usize,
}

fn main() {
    let start = Instant::now();
    let mut cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbosity.log_level_filter())
        .init();

    if cli.chunk_size == 0 {
        cli.chunk_size = gpu::get_highest_chunk_size();
    }
    let GenUnditheredChunksOutput {
        first_pass,
        second_pass,
        height,
        width,
    } = gen_undithered_frames(cli.input, cli.chunk_size);
    let palette = palette::gen_palette(first_pass, height, width);

    let quantized_frames = quantizer::quantize_frames(second_pass, palette.clone()).flatten();
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
fn gen_undithered_frames(
    input: String,
    chunk_size: usize,
) -> GenUnditheredChunksOutput<impl Iterator<Item = Vec<GifFrame>>> {
    let reader1 = GifReader::new(input.clone());
    let reader2 = GifReader::new(input);
    let height = reader1.height();
    let width = reader1.width();
    let undithered_chunks1 = ChunkedIter::new(reader1, chunk_size).map(undither::undither_chunk);
    let undithered_chunks2 = ChunkedIter::new(reader2, chunk_size).map(undither::undither_chunk);
    GenUnditheredChunksOutput {
        first_pass: undithered_chunks1,
        second_pass: undithered_chunks2,
        height,
        width,
    }
}
