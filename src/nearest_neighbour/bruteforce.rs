use crate::{image::Rgb, nearest_neighbour::NnSolver};

pub struct Bruteforce {
    palette: Vec<Rgb>,
}
impl NnSolver for Bruteforce {
    fn new(lst: Vec<Rgb>) -> Self {
        if lst.is_empty() {
            panic!("lst must not be empty");
        }
        Self { palette: lst }
    }

    fn nn(&self, target: Rgb, exclude: Option<[Rgb; 2]>) -> Option<Rgb> {
        let mut best_dist = u32::MAX;
        let mut ans = None;
        for &colour in &self.palette {
            if let Some(exclude) = exclude
                && (colour == exclude[0] || colour == exclude[1])
            {
                continue;
            }
            let dist = target.distance_sq(colour);
            if dist < best_dist {
                best_dist = dist;
                ans = Some(colour);
            }
        }
        ans
    }
}
