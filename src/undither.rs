use crate::{
    gpu,
    image::{GifFrame, Image},
};

pub fn undither_frames(
    frames: impl Iterator<Item = GifFrame> + 'static,
    stream: bool,
) -> Box<dyn Iterator<Item = GifFrame>> {
    if !stream {
        let frames: Vec<GifFrame> = frames.collect();
        let images: Vec<&Image> = frames.iter().map(|frame| &frame.image).collect();
        let palettes = frames.iter().map(|frame| &frame.palette).collect();
        let output_images = gpu::run_shader_with_frames("undither_frame", images, palettes);
        Box::new(
            frames
                .into_iter()
                .zip(output_images)
                .map(|(mut frame, output_image)| {
                    frame.image = output_image;
                    frame
                }),
        )
    } else {
        Box::new(frames.map(|mut frame| {
            let image = gpu::run_shader_with_frames(
                "undither_frame",
                vec![&frame.image],
                vec![&frame.palette],
            );
            frame.image = image.into_iter().next().unwrap();
            frame
        }))
    }
}
