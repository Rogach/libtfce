use rand::{Rng, StdRng, SeedableRng};

pub fn run_permutation(
    a: &Vec<Vec<f64>>,
    b: &Vec<Vec<f64>>,
    n: i32,
    op: &mut FnMut(Vec<&Vec<f64>>, Vec<&Vec<f64>>) -> Vec<f64>
) -> Vec<bool> {
    let nsubj = a.len();
    let mut rng = StdRng::from_seed(&[17556, 31771, 29830, 29830]);

    let mut distribution = Vec::with_capacity(n as usize);

    for _ in 0..n {
        let mut permuted_a = Vec::with_capacity(nsubj);
        let mut permuted_b = Vec::with_capacity(nsubj);
        for (sa, sb) in a.iter().zip(b.iter()) {
            if rng.gen() {
                permuted_a.push(sa);
                permuted_b.push(sb);
            } else {
                permuted_a.push(sb);
                permuted_b.push(sa);
            }
        }
        distribution.push(op(permuted_a, permuted_b).into_iter().fold(0.0, f64::max));
    }

    distribution.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q95 = distribution[((n as f64) * 0.95).floor() as usize];
    op(a.iter().collect(), b.iter().collect()).into_iter().map(|v| v >= q95).collect()
}

pub fn significant_indices(perm_result: &Vec<bool>) -> Vec<usize> {
    let mut indices = Vec::new();
    for (i, &significant) in perm_result.iter().enumerate() {
        if significant {
            indices.push(i);
        }
    }
    indices
}
