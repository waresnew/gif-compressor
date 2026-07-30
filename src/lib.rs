pub mod chunked_iter;
pub mod gpu;
pub mod image;
mod nearest_neighbour;
pub mod palette;
pub mod quantizer;
pub mod reader;
pub mod transparency;
pub mod undither;
pub mod writer;
pub mod bench_impls {
    pub use crate::nearest_neighbour::{bruteforce, kdtree};
}
pub use crate::nearest_neighbour::{ChosenNnSolver, NnSolver};
