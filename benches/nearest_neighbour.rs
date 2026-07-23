use criterion::{Criterion, criterion_group, criterion_main};
use gif_compressor::{image::Rgb, kdtree::KdTree};
use rand::{RngExt, SeedableRng, rngs::SmallRng};
use rustc_hash::FxHashMap;
use std::{hint::black_box, iter::repeat_with};

fn bench_nn(c: &mut Criterion) {
    let seed = 1234;
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut random_rgb = || Rgb::new(rng.random(), rng.random(), rng.random());
    let palette: Vec<Rgb> = repeat_with(&mut random_rgb).take(256).collect();
    let queries: Vec<Rgb> = repeat_with(&mut random_rgb).take(500).collect();
    let kdtree = KdTree::new(palette.clone());
    c.bench_function("kdtree 500 queries in 256 palette", |b| {
        b.iter(|| {
            for &query in &queries {
                let mut cache = FxHashMap::default();
                kdtree.k_nn(query, 1, &mut cache);
            }
        })
    });
    c.bench_function("bruteforce 500 queries in 256 palette", |b| {
        b.iter(|| {
            for &query in &queries {
                let mut best_dist = u32::MAX;
                for &colour in &palette {
                    let dist = query.distance_sq(colour);
                    if dist < best_dist {
                        best_dist = dist;
                    }
                }
                black_box(best_dist);
            }
        })
    });
}

criterion_group!(benches, bench_nn);
criterion_main!(benches);
