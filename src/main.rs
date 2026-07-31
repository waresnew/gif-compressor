use clap::Parser;
use gif_compressor::chunked_file::ChunkedFile;
use gif_compressor::chunked_iter::ChunkedIter;
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

fn main() {
    let start = Instant::now();
    let mut cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbosity.log_level_filter())
        .init();
    let reader = GifReader::new(cli.input);
    let height = reader.height();
    let width = reader.width();
    if cli.chunk_size == 0 {
        cli.chunk_size = gpu::get_highest_chunk_size(height, width);
        info!("inferring chunk_size = {}", cli.chunk_size);
    }

    let undithered_chunks = ChunkedIter::new(reader, cli.chunk_size).map(undither::undither_chunk);
    let mut temp_file = tempfile::tempfile().unwrap();
    let mut chunked_file = ChunkedFile::new(&mut temp_file);
    let palette = palette::gen_palette(
        undithered_chunks.inspect(|chunk| chunked_file.write_chunk(chunk.clone())),
        height,
        width,
    );
    chunked_file.finish_writing();
    info!(
        "saved {:.1} MB of undithered chunks to temp file",
        chunked_file.size() as f64 / 1_000_000.0
    );

    let quantized_frames =
        chunked_file.flat_map(|chunk| quantizer::quantize_chunk(chunk, &palette));
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
