use ::voxel::Voxel;
use std::collections::VecDeque;

pub fn approximate_tfce(voxels: &mut Vec<Voxel>, steps: i32) {
    for v in voxels.iter_mut() {
        v.tfce_value = 0.0;
    }

    let max_value = voxels.iter().map(|v| v.value).fold(0.0, f64::max);
    let delta = max_value / (steps as f64);

    let mut t = 0.0;
    while t < max_value {
        let clusters = get_clusters(voxels, t);
        for cluster in clusters.into_iter() {
            let increase = (cluster.len() as f64).powf(2.0/3.0) * (t + delta / 2.0).powf(2.0) * delta;
            for i in cluster.into_iter() {
                voxels[i].tfce_value += increase;
            }
        }
        t += delta;
    }
}

fn get_clusters(voxels: &mut Vec<Voxel>, min_value: f64) -> Vec<Vec<usize>> {
    let mut visited = {
        let mut v = Vec::with_capacity(voxels.len());
        let mut i = 0;
        while i < voxels.len() {
            v.push(false);
            i += 1;
        }
        v
    };

    let mut clusters = Vec::new();
    let mut i: usize = 0;
    while i < voxels.len() {
        if !visited[i] && voxels[i].value >= min_value {
            let mut current_cluster = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(i);
            while let Some(vi) = queue.pop_front() {
                if !visited[vi] {
                    current_cluster.push(vi);
                    for ni in voxels[vi].voxel_links.iter() {
                        if !visited[*ni] && voxels[*ni].value >= min_value {
                            queue.push_back(*ni);
                        }
                    }
                    visited[vi] = true;
                }
            }
            clusters.push(current_cluster);
        }
        i += 1;
    }

    clusters
}

#[cfg(test)]
mod tests {
    use ::field::generate_1d_field;
    use ::field::set_random_values;
    use test::Bencher;
    use super::*;

    #[test]
    fn test_get_clusters() {
        let mut voxels = generate_1d_field(6);
        set_random_values(&mut voxels, 0.0, 1.0, &[17556, 31771, 29830, 29830]);
        voxels[2].value = 3.0;
        voxels[3].value = 3.5;
        assert_eq!(
            get_clusters(&mut voxels, 2.5),
            vec![vec![2, 3]]
        );
    }

    #[bench]
    fn benchmark_get_clusters(b: &mut Bencher) {
        let mut voxels = generate_1d_field(10000);
        set_random_values(&mut voxels, 0.0, 1.0, &[17556, 31771, 29830, 29830]);
        b.iter(|| get_clusters(&mut voxels, 0.5));
    }

    #[test]
    fn test_approximate_tfce() {
        let mut voxels = generate_1d_field(6);
        set_random_values(&mut voxels, 0.0, 1.0, &[17556, 31771, 29830, 29830]);
        approximate_tfce(&mut voxels, 50);
        println!("{:?}", voxels.iter().map(|v| v.value).collect::<Vec<f64>>());
        println!("{:?}", voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>());
    }

    #[bench]
    fn benchmark_approximate_tfce(b: &mut Bencher) {
        let mut voxels = generate_1d_field(10000);
        set_random_values(&mut voxels, 0.0, 1.0, &[17556, 31771, 29830, 29830]);
        b.iter(|| approximate_tfce(&mut voxels, 50));
    }
}
