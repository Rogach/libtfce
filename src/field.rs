use rand::{Rng, StdRng, SeedableRng};
use ::voxel::Voxel;

pub fn generate_1d_field(size: usize, min_value: f64, max_value: f64) -> Vec<Voxel> {
    assert!(size >= 2);
    let mut rng = StdRng::from_seed(&[17556, 31771, 29830, 29830]);

    let mut voxels = Vec::new();
    for i in 0..size {
        let links =
            if i == 0 {
                vec![1]
            } else if i == size - 1 {
                vec![i - 1]
            } else {
                vec![i - 1, i + 1]
            };
        voxels.push(Voxel::new(rng.gen_range(min_value, max_value), links));
    }
    voxels
}
