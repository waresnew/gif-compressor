use crate::{
    image::{GifFrame, Rgb},
    nearest_neighbour::{ChosenNnSolver, NnSolver},
};

pub fn quantize(frame: &mut GifFrame, nn_solver: &ChosenNnSolver) {
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
