use crate::{
    image::{GifFrame, Rgb},
    nearest_neighbour::{ChosenNnSolver, NnSolver},
};

//TODO: use gpu
pub fn quantize_frames(
    chunks: impl Iterator<Item = Vec<GifFrame>>,
    palette: Vec<Rgb>,
) -> impl Iterator<Item = Vec<GifFrame>> {
    let mut nn_solver = ChosenNnSolver::new(palette);
    chunks.map(move |chunk| {
        chunk
            .into_iter()
            .map(|mut frame| {
                quantize_frame(&mut frame, &mut nn_solver);
                frame
            })
            .collect()
    })
}
fn quantize_frame(frame: &mut GifFrame, nn_solver: &mut ChosenNnSolver) {
    for i in 0..frame.image.height {
        for j in 0..frame.image.width {
            let cur = frame.image.get(i, j);
            let best = nn_solver.nn(cur, None).unwrap();
            *frame.image.get_mut(i, j) = best;
        }
    }
}
