#[macro_use]
extern crate criterion;

use criterion::{Criterion, Fun};
use rand::distributions::{Distribution, Standard};
use rand::prelude::*;
use unroll::unroll_for_loops;

static SEED: [u8; 32] = [3; 32];
const LEN: usize = 524_288;

#[inline]
fn make_random_vec(len: usize) -> Vec<f64>
where
    Standard: Distribution<f64>,
{
    let mut rng: StdRng = SeedableRng::from_seed(SEED);
    (0..len).map(|_| rng.gen()).collect()
}

#[inline]
fn compute(x: &f64) -> f64 {
    // An arbitrary function
    x * 3.6321 + 42314.0
}

#[inline]
#[unroll_for_loops]
fn unrolled_for_loop(v: &[f64]) -> f64 {
    let mut res = [0.0; 32];
    for i in (0..LEN / 32).step_by(32) {
        for j in 0..32 {
            res[j] += compute(unsafe { v.get_unchecked(i + j) });
        }
    }
    res.iter().sum()
}

#[inline]
fn explicit_for_loop(v: &[f64]) -> f64 {
    let mut res = 0.0;
    for i in 0..LEN {
        res += compute(unsafe { v.get_unchecked(i) });
    }
    res
}

fn unroll_for_loops(c: &mut Criterion) {
    let for_each_loop = Fun::new("Map", move |b, _| {
        let v = make_random_vec(LEN);
        b.iter(|| v.iter().map(compute).sum::<f64>())
    });

    let for_loop = Fun::new("For loop", move |b, _| {
        let v = make_random_vec(LEN);
        b.iter(|| {
            let mut res = 0.0;
            for a in v.iter() {
                res += compute(a);
            }
            res
        })
    });

    let explicit_for_loop = Fun::new("Explicit for loop", move |b, _| {
        let v = make_random_vec(LEN);
        b.iter(|| explicit_for_loop(&v))
    });

    let unrolled_for_loop = Fun::new("Unrolled for loop", move |b, _| {
        let v = make_random_vec(LEN);
        b.iter(|| unrolled_for_loop(&v))
    });

    let fns = vec![
        for_each_loop,
        for_loop,
        explicit_for_loop,
        unrolled_for_loop,
    ];
    c.bench_functions("Unroll For Loops", fns, ());
}

criterion_group!(benches, unroll_for_loops);
criterion_main!(benches);
