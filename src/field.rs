use rand::{Rng, StdRng, SeedableRng};
use ::voxel::Voxel;

pub fn generate_1d_field(size: usize) -> Vec<Voxel> {
    assert!(size >= 2);
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
        voxels.push(Voxel::new(0.0, links));
    }
    voxels
}

pub fn generate_2d4c_field(width: usize) -> Vec<Voxel> {
    assert!(width >= 2);
    let mut voxels = Vec::new();
    for y in 0..width {
        for x in 0..width {
            let mut links = Vec::new();
            if x < width - 1 {
                links.push(y * width + x + 1);
            }
            if x > 0 {
                links.push(y * width + x - 1);
            }
            if y < width - 1 {
                links.push((y + 1) * width + x);
            }
            if y > 0 {
                links.push((y - 1) * width + x);
            }
            voxels.push(Voxel::new(0.0, links));
        }
    }
    voxels
}

pub fn generate_2d8c_field(width: usize) -> Vec<Voxel> {
    assert!(width >= 2);
    let mut voxels = Vec::new();
    for y in 0..width {
        for x in 0..width {
            let mut links = Vec::new();
            if x < width - 1 {
                links.push(y * width + x + 1);
            }
            if x > 0 {
                links.push(y * width + x - 1);
            }
            if y < width - 1 {
                links.push((y + 1) * width + x);
            }
            if y > 0 {
                links.push((y - 1) * width + x);
            }
            if x < width - 1 && y < width - 1 {
                links.push((y + 1) * width + (x + 1));
            }
            if x < width - 1 && y > 0 {
                links.push((y - 1) * width + (x + 1));
            }
            if x > 0 && y < width - 1 {
                links.push((y + 1) * width + (x - 1));
            }
            if x > 0 && y > 0 {
                links.push((y - 1) * width + (x - 1));
            }
            voxels.push(Voxel::new(0.0, links));
        }
    }
    voxels
}

pub fn set_random_values(
    voxels: &mut Vec<Voxel>,
    min_value: f64,
    max_value: f64,
    seed: &[usize]
) {
    let mut rng = StdRng::from_seed(seed);
    for v in voxels.iter_mut() {
        v.value = rng.gen_range(min_value, max_value);
    }
}
