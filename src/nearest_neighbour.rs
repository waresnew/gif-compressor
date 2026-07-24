use crate::{image::Rgb, nearest_neighbour::kdtree::KdTree};

pub mod bruteforce;
pub mod kdtree;

pub type ChosenNnSolver = KdTree;
pub trait NnSolver {
    fn new(lst: Vec<Rgb>) -> Self;
    fn nn(&self, target: Rgb, exclude: Option<[Rgb; 2]>) -> Option<Rgb>;
}
