use crate::{
    gpu,
    image::{GifFrame, Image},
};

pub fn undither_chunk(chunk: Vec<GifFrame>) -> Vec<GifFrame> {
    let images: Vec<&Image> = chunk.iter().map(|frame| &frame.image).collect();
    let palettes = chunk.iter().map(|frame| &frame.palette).collect();
    let output_images = gpu::run_shader_with_frames("undither_frame", images, palettes);
    chunk
        .into_iter()
        .zip(output_images)
        .map(|(mut frame, output_image)| {
            frame.image = output_image;
            frame
        })
        .collect()
}
