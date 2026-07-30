use crate::{
    gpu,
    image::{GifFrame, Image, Rgb},
};
pub fn quantize_chunk(chunk: Vec<GifFrame>, palette: &Vec<Rgb>) -> Vec<GifFrame> {
    let images: Vec<&Image> = chunk.iter().map(|frame| &frame.image).collect();
    let palettes = vec![palette; chunk.len()];
    let output_images = gpu::run_shader_with_frames("nn_in_palette", images, palettes);
    chunk
        .into_iter()
        .zip(output_images)
        .map(|(mut frame, output_image)| {
            frame.image = output_image;
            frame
        })
        .collect()
}
