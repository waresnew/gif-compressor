use crate::{
    image::{GifFrame, Rgb},
    nearest_neighbour::{ChosenNnSolver, NnSolver},
};

fn quantize(frame: &mut GifFrame, nn_solver: &ChosenNnSolver) {
    for i in 0..frame.image.height {
        for j in 0..frame.image.width {
            let cur = frame.image.get(i, j);
            if cur.transparent {
                continue;
            }
            let best = nn_solver.nn(cur, None).unwrap();
            *frame.image.get_mut(i, j) = best;
        }
    }
}
pub fn get_quantize_transform(nn_solver: ChosenNnSolver) -> impl Fn(&mut GifFrame) {
    move |frame| quantize(frame, &nn_solver)
}
