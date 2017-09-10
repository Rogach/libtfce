extern crate libtfce;
extern crate rand;

use libtfce::field::{generate_1d_field};
use libtfce::tfce;
use libtfce::ttest;
use libtfce::permutation;
use libtfce::prob_bin_search::{probabilistic_binary_search, NormDistr};
use rand::{Rng, StdRng, SeedableRng};

fn main() {
    let nsubj = 10;
    let n = 100;
    let mut rng = StdRng::from_seed(&[17556, 31771, 29830, 29830]);

    for _ in 0..12 {
        eprintln!("{:?}", probabilistic_binary_search(
            NormDistr::new(2.0, 2.0),
            3000,
            &mut |t| {
                if t <= 0.0 {
                    true
                } else {
                    let mut voxels = generate_1d_field(n);

                    let mut a = Vec::with_capacity(nsubj);
                    let mut b = Vec::with_capacity(nsubj);

                    for _ in 0..nsubj {
                        let mut sa = Vec::with_capacity(n);
                        let mut sb = Vec::with_capacity(n);
                        for _ in 0..n {
                            sa.push(rng.next_f64());
                            sb.push(rng.next_f64());
                        }
                        a.push(sa);
                        b.push(sb);
                    }

                    for x in 5..15 {
                        for s in 0..nsubj {
                            b[s][x] += t;
                        }
                    }

                    let result = permutation::significant_indices(
                        &permutation::run_permutation(
                            &a, &b, 200,
                            &mut |a, b| {
                                for (v, tv) in voxels.iter_mut().zip(::ttest::ttest_rel_vec(&a, &b).into_iter()) {
                                    v.value = tv.abs();
                                }
                                tfce(&mut voxels, 2.0/3.0, 2.0);
                                voxels.iter().map(|v| v.tfce_value).collect()
                            }
                        )
                    );

                    result.is_empty()
                }
            }
        ));
    }

}
