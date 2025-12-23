use gif_compressor::image::{GifFrame, Palette, RGB};
use gif_compressor::undither::undither;
use std::env;
use std::fs::File;
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::Indexed);
    let file = File::open(&args[1]).unwrap();
    let mut decoder = decoder.read_info(file).unwrap();
    if let Some(palette_raw) = decoder.global_palette() {
        //TODO:we just test first frame for now (so no transparent indices)
        // once we do multiple frames, need to handle transparent index and gif disposal mode
        let palette = Palette::new(
            palette_raw
                .chunks_exact(3)
                .map(|c| RGB {
                    r: c[0],
                    g: c[1],
                    b: c[2],
                })
                .collect(),
        );
        let Some(frame) = decoder.read_next_frame().unwrap() else {
            return;
        };
        dbg!(&frame.transparent);

        let frame = GifFrame::new(frame);
        let res = undither(&frame, &palette);
        export_png(&res);
    } else {
        todo!("convert local palette gif to global palette");
    }
}
fn export_png(res: &Vec<Vec<RGB>>) {
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
    img_buffer.save("output.png").unwrap();
}
