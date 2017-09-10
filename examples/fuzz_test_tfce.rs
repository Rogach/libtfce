extern crate libtfce;

use libtfce::field::{generate_2d8c_field, set_random_values};
use libtfce::tfce;
use libtfce::approximate_tfce;

fn main() {
    for x in 29830..29900 {
        println!("x = {}", x);
        let n = 10;
        let mut voxels = generate_2d8c_field(n);
        set_random_values(&mut voxels, 0.0, 1.0, &[17556, 31771, 29830, x]);

        let mut approx_voxels = voxels.clone();
        approximate_tfce(&mut approx_voxels, 10000);

        let mut exact_voxels = voxels.clone();
        tfce(&mut exact_voxels, 2.0/3.0, 2.0);

        for n in 0..voxels.len() {
            if (approx_voxels[n].tfce_value - exact_voxels[n].tfce_value).abs() > 1e-3 {
                panic!("difference at x = {}", x);
            }
        }
    }
}
