extern crate libtfce;

use libtfce::field::generate_1d_field;
use libtfce::tfce;
use libtfce::ttest;
use libtfce::permutation;
use libtfce::read_data_file;
use std::env;

fn main() {
    let data_file = env::args().nth(1).expect("expected input filename as first argument");

    let (a, b) = read_data_file(data_file);
    let mut voxels = generate_1d_field(a[0].len());

    let result = permutation::get_periods(permutation::significant_indices(
        &permutation::run_permutation(
            &a, &b, 1000,
            &mut |a, b| {
                for (v, tv) in voxels.iter_mut().zip(::ttest::ttest_rel_vec(&a, &b).into_iter()) {
                    v.value = tv.abs();
                }
                tfce(&mut voxels, 1.9, 0.0);
                voxels.iter().map(|v| v.tfce_value).collect()
            }
        )
    ));

    eprintln!("result: {:?}", result);
}
