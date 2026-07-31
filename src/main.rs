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

struct UnditheredChunks<I: Iterator<Item = Vec<GifFrame>>> {
    first_pass: I,
    second_pass: I,
}

/// (height,width)
fn peek_height_and_width(input: String) -> (usize, usize) {
    let reader = GifReader::new(input);
    (reader.height(), reader.width())
}
fn main() {
    let start = Instant::now();
    let mut cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbosity.log_level_filter())
        .init();

    let (height, width) = peek_height_and_width(cli.input.clone());
    if cli.chunk_size == 0 {
        cli.chunk_size = gpu::get_highest_chunk_size(height, width);
        info!("inferring chunk_size = {}", cli.chunk_size);
    }
    let UnditheredChunks {
        first_pass,
        second_pass,
    } = prepare_undithered_chunks(cli.input, cli.chunk_size);
    let palette = palette::gen_palette(first_pass, height, width);

    let quantized_frames = second_pass.flat_map(|chunk| quantizer::quantize_chunk(chunk, &palette));
    let mut transparency = TransparencyOptimizer::new(cli.transparency_threshold);
    let transparency_optimized = transparency.apply_transparency_all(quantized_frames);
    let mut output_file = File::create(&cli.output).unwrap();
    let mut writer = GifWriter::new(
        transparency_optimized,
        palette.clone(),
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

fn prepare_undithered_chunks(
    input: String,
    chunk_size: usize,
) -> UnditheredChunks<impl Iterator<Item = Vec<GifFrame>>> {
    let reader1 = GifReader::new(input.clone());
    let reader2 = GifReader::new(input);
    let undithered_chunks1 = ChunkedIter::new(reader1, chunk_size).map(undither::undither_chunk);
    let undithered_chunks2 = ChunkedIter::new(reader2, chunk_size).map(undither::undither_chunk);
    UnditheredChunks {
        first_pass: undithered_chunks1,
        second_pass: undithered_chunks2,
    }
}
