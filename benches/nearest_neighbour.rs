use criterion::{Criterion, criterion_group, criterion_main};
use gif_compressor::{
    image::Rgb,
    nearest_neighbour::{NnSolver, bruteforce::Bruteforce, kdtree::KdTree},
};
use rand::{RngExt, SeedableRng, rngs::SmallRng};
use std::{hint::black_box, iter::repeat_with};

fn bench_nn(c: &mut Criterion) {
    let seed = 1234;
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut random_rgb = || Rgb::new(rng.random(), rng.random(), rng.random());
    let palette: Vec<Rgb> = repeat_with(&mut random_rgb).take(256).collect();
    let queries: Vec<Rgb> = repeat_with(&mut random_rgb).take(500).collect();
    let mut kdtree = KdTree::new(palette.clone());
    c.bench_function("kdtree 500 queries in 256 palette", |b| {
        b.iter(|| {
            for &query in &queries {
                kdtree.nn(query, None);
            }
        })
    });
    let mut bruteforce = Bruteforce::new(palette.clone());
    c.bench_function("bruteforce 500 queries in 256 palette", |b| {
        b.iter(|| {
            for &query in &queries {
                black_box(bruteforce.nn(query, None));
            }
        })
    });
}

criterion_group!(benches, bench_nn);
criterion_main!(benches);
