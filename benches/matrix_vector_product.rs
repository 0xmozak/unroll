#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use rand::distributions::Standard;
use rand::prelude::*;
use unroll::unroll_for_loops;

static SEED: [u8; 32] = [3; 32];
const N: usize = 5;

#[inline]
fn make_random_prod_pair() -> ([[f64; N]; N], [f64; N])
where
    Standard: Distribution<f64>,
{
    let mut rng: StdRng = SeedableRng::from_seed(SEED);
    let mut mtx = [[0.0; N]; N];
    let mut vec = [0.0; N];
    for i in 0..N {
        for j in 0..N {
            mtx[i][j] = rng.gen();
        }
        vec[i] = rng.gen();
    }

    (mtx, vec)
}

#[inline]
#[allow(clippy::needless_range_loop)]
fn mtx_vec_mul(mtx: &[[f64; N]; N], vec: &[f64; N]) -> [f64; N] {
    let mut out = [0.0; N];
    for col in 0..5 {
        for row in 0..5 {
            out[row] += mtx[col][row] * vec[col];
        }
    }
    out
}

#[inline]
#[unroll_for_loops]
fn unroll_mtx_vec_mul(mtx: &[[f64; N]; N], vec: &[f64; N]) -> [f64; N] {
    let mut out = [0.0; N];
    for col in 0..5 {
        for row in 0..5 {
            out[row] += mtx[col][row] * vec[col];
        }
    }
    out
}

fn ordinary(c: &mut Criterion) {
    let (m, v) = make_random_prod_pair();
    c.bench_with_input(BenchmarkId::new("mtx_vec_mul", 0), &(m, v), |b, (m, v)| {
        b.iter(|| mtx_vec_mul(m, v))
    });
}

fn unrolled_mtx_vec_mul(c: &mut Criterion) {
    let (m, v) = make_random_prod_pair();
    c.bench_with_input(BenchmarkId::new("mtx_vec_mul", 0), &(m, v), |b, (m, v)| {
        b.iter(|| unroll_mtx_vec_mul(m, v))
    });
}

fn matrix_vector_product(c: &mut Criterion) {
    ordinary(c);
    unrolled_mtx_vec_mul(c);
}

criterion_group!(benches, matrix_vector_product);
criterion_main!(benches);
