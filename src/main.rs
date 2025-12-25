use gif::DisposalMethod;
use gif_compressor::image::{GifFrame, Palette, RGB};
use gif_compressor::undither::undither;
use std::env;
use std::fs::File;
use std::time::Instant;
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let file = File::open(&args[1]).unwrap();
    let decoder = decoder.read_info(file).unwrap();
    let start = Instant::now();
    if decoder.width() == 0 || decoder.height() == 0 {
        panic!("malformed gif: width or height is 0");
    }
    let mut global_palette = decoder
        .global_palette()
        .map(|x| Palette::new(x, decoder.bg_color()));
    let bg = global_palette
        .as_ref()
        .map_or(RGB::default(), |x| x.bg.unwrap_or(RGB::default()));
    let mut canvas = vec![vec![bg; decoder.width() as usize]; decoder.height() as usize]; //reused
    let mut prev_canvas = canvas.clone();
    let mut iter = decoder.into_iter().enumerate();
    while let Some((i, Ok(frame_raw))) = iter.next() {
        let mut frame =
            GifFrame::render_frame_to_canvas(&frame_raw, &mut canvas, global_palette.as_mut());
        let res = undither(&mut frame);
        export_png(&res, i);
        for i in 0..canvas.len() {
            for j in 0..canvas[0].len() {
                canvas[i][j] = match frame_raw.dispose {
                    DisposalMethod::Any | DisposalMethod::Keep => canvas[i][j],
                    DisposalMethod::Background => bg,
                    DisposalMethod::Previous => prev_canvas[i][j],
                }
            }
        }
        prev_canvas = canvas.clone();
    }
    println!("took {:?}", start.elapsed());
}
fn export_png(res: &[Vec<RGB>], i: usize) {
    //TODO: i'm only using image-rs crate to export to png for the undither test
    let mut img_buffer = image::ImageBuffer::new(res[0].len() as u32, res.len() as u32);
    for i in 0..res.len() {
        for j in 0..res[0].len() {
            let pixel = res[i][j];
            img_buffer.put_pixel(
                j as u32,
                i as u32,
                image::Rgba([pixel.r, pixel.g, pixel.b, 255]),
            );
        }
    }
    img_buffer.save(format!("output/frame{}.png", i)).unwrap();
}
