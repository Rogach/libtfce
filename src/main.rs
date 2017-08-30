#![feature(test)]
#![feature(type_ascription)]
#![feature(box_patterns)]

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

extern crate rand;
extern crate test;

use std::collections::VecDeque;
use std::collections::BinaryHeap;
use rand::{Rng, StdRng, SeedableRng};
use test::Bencher;
use std::cmp::Ordering;

fn main() {
    let mut voxels = generate_1d_field(100, 0.0, 10.0);
    eprintln!("{:?}", voxels.iter().map(|v| v.value).collect::<Vec<f64>>());

    println!("from matplotlib import pyplot");
    {
        let mut voxels = voxels.clone();
        slow_tfce(&mut voxels, 50);
        println!("pyplot.plot({:?}, 'r')", voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>());
    }
    {
        let mut voxels = voxels.clone();
        fast_tfce(&mut voxels);
        println!("pyplot.plot({:?}, 'g')", voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>());
    }
    println!("pyplot.show()");
}

#[derive(Debug, Clone)]
struct Voxel {
    voxel_links: Vec<usize>,
    value: f64,
    tfce_value: f64
}

impl Voxel {
    fn new(value: f64, links: Vec<usize>) -> Voxel {
        Voxel {
            voxel_links: links,
            value: value,
            tfce_value: 0.0
        }
    }
}

fn slow_tfce(voxels: &mut Vec<Voxel>, steps: i32) {
    let max_value = voxels.iter().map(|v| v.value).fold(0.0, f64::max);
    let delta = max_value / (steps as f64);

    let mut t = delta / 2.0;
    while t < max_value {
        let clusters = get_clusters(voxels, t);
        for cluster in clusters.into_iter() {
            let increase = (cluster.len() as f64).powf(2.0/3.0) * t.powf(2.0) * delta;
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
                current_cluster.push(vi);
                for ni in voxels[vi].voxel_links.iter() {
                    if !visited[*ni] && voxels[*ni].value >= min_value {
                        queue.push_back(*ni);
                    }
                }
                visited[vi] = true;
            }
            clusters.push(current_cluster);
        }
        i += 1;
    }

    clusters
}

fn fast_tfce(voxels: &mut Vec<Voxel>) {
    let cluster = build_cluster_tree(voxels);
    fill_clusters(voxels, cluster);
}

fn build_cluster_tree(voxels: &mut Vec<Voxel>) -> Cluster {
    let mut visited = {
        let mut v = Vec::with_capacity(voxels.len());
        let mut i = 0;
        while i < voxels.len() {
            v.push(false);
            i += 1;
        }
        v
    };


    let mut voxel_queue = BinaryHeap::new();

    let max_voxel_index = find_max_voxel(voxels);
    let max_value = voxels[max_voxel_index].value;
    voxel_queue.push(VoxelPriority(max_value, max_voxel_index));

    let mut cluster_stack: Vec<ClusterHunk> = Vec::new();

    let mut current_value = max_value;
    let mut low_value = 0.0;

    let mut current_cluster = Cluster::new();

    while !cluster_stack.is_empty() || !voxel_queue.is_empty() {
        while let Some(VoxelPriority(value, index)) = voxel_queue.pop() {
            while value < low_value && !cluster_stack.is_empty() {
                let mut other_hunk = cluster_stack.pop().unwrap();

                // merge voxel queues, smaller into bigger
                if voxel_queue.len() < other_hunk.voxel_queue.len() {
                    std::mem::swap(&mut voxel_queue, &mut other_hunk.voxel_queue);
                }
                for i in other_hunk.voxel_queue.into_iter() {
                    voxel_queue.push(i);
                }

                // create new cluster, pointing to two older ones
                current_cluster = Cluster {
                    voxel_indices: Vec::new(),
                    parent_cluster_1: Some(Box::new(other_hunk.cluster)),
                    parent_cluster_2: Some(Box::new(current_cluster))
                };

                // set low value to older low value
                low_value = other_hunk.low_value;
            }

            if value <= current_value {
                // normal descent, still in the same cluster
                // just mark as visited, add to current cluster
                // and traverse voxel links
                visited[index] = true;
                current_value = value;
                current_cluster.voxel_indices.push(index);
                for &ni in voxels[index].voxel_links.iter() {
                    if !visited[ni] {
                        voxel_queue.push(VoxelPriority(voxels[ni].value, ni));
                    }
                }
            } else {
                // traversed to other hill
                cluster_stack.push(ClusterHunk {
                    low_value: low_value,
                    voxel_queue: std::mem::replace(&mut voxel_queue, BinaryHeap::new()),
                    cluster: std::mem::replace(&mut current_cluster, Cluster::new())
                });
                low_value = current_value;

                let mut max_index = index;
                while let Some(bigger_index) = traverse_max(max_index, voxels) {
                    max_index = bigger_index;
                }
                current_value = voxels[max_index].value;
                voxel_queue.push(VoxelPriority(current_value, max_index));
            }
        }

        if let Some(mut other_hunk) = cluster_stack.pop() {
            // merge voxel queues, smaller into bigger
            if voxel_queue.len() < other_hunk.voxel_queue.len() {
                std::mem::swap(&mut voxel_queue, &mut other_hunk.voxel_queue);
            }
            for i in other_hunk.voxel_queue.into_iter() {
                voxel_queue.push(i);
            }

            // create new cluster, pointing to two older ones
            current_cluster = Cluster {
                voxel_indices: Vec::new(),
                parent_cluster_1: Some(Box::new(other_hunk.cluster)),
                parent_cluster_2: Some(Box::new(current_cluster))
            };

            // set low value to older low value
            low_value = other_hunk.low_value;
        }
    }

    current_cluster
}

fn fill_clusters(voxels: &mut Vec<Voxel>, root_cluster: Cluster) {
    let mut cluster_stack = Vec::new();
    cluster_stack.push((root_cluster, 0: i32, 0.0, 0.0));

    while let Some((cluster, mut sz, mut prev_value, mut prev_tfce_value)) = cluster_stack.pop() {
        for vi in cluster.voxel_indices.into_iter().rev() {
            let value = voxels[vi].value;
            let delta = value - prev_value;
            sz += 1;
            let tfce_value =
                prev_tfce_value +
                (sz as f64).powf(2.0/3.0) * value.powf(2.0) * delta;
            voxels[vi].tfce_value = tfce_value;

            prev_value = value;
            prev_tfce_value = tfce_value;
        }
        if let Some(box cluster) = cluster.parent_cluster_1 {
            cluster_stack.push((cluster, sz, prev_value, prev_tfce_value));
        }
        if let Some(box cluster) = cluster.parent_cluster_2 {
            cluster_stack.push((cluster, sz, prev_value, prev_tfce_value));
        }
    }
}

fn find_max_voxel(voxels: &Vec<Voxel>) -> usize {
    let mut max_i = 0;
    let mut max_value = voxels[0].value;

    let mut i = 1;
    while i < voxels.len() {
        if voxels[i].value > max_value {
            max_i = i;
            max_value = voxels[i].value;
        }
        i += 1;
    }

    max_i
}

fn traverse_max(from: usize, voxels: &Vec<Voxel>) -> Option<usize> {
    let value = voxels[from].value;
    let mut max_index = None;
    let mut max_value = value;
    for &ni in voxels[from].voxel_links.iter() {
        if voxels[ni].value > max_value {
            max_index = Some(ni);
            max_value = voxels[ni].value;
        }
    }
    max_index
}

#[derive(Debug)]
struct VoxelPriority (f64, usize);
impl PartialEq for VoxelPriority {
    fn eq(&self, other: &VoxelPriority) -> bool {
        self.0 == other.0 &&
            self.1 == other.1
    }
}
impl Eq for VoxelPriority {}
impl PartialOrd for VoxelPriority {
    fn partial_cmp(&self, other: &VoxelPriority) -> Option<Ordering> {
        if self.0 < other.0 {
            Some(Ordering::Less)
        } else if self.0 > other.0 {
            Some(Ordering::Greater)
        } else {
            self.1.partial_cmp(&other.1)
        }
    }
}
impl Ord for VoxelPriority {
    fn cmp(&self, other: &VoxelPriority) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Cluster {
    voxel_indices: Vec<usize>,
    parent_cluster_1: Option<Box<Cluster>>,
    parent_cluster_2: Option<Box<Cluster>>
}
impl Cluster {
    fn new() -> Cluster {
        Cluster {
            voxel_indices: Vec::new(),
            parent_cluster_1: None,
            parent_cluster_2: None
        }
    }
}

#[derive(Debug)]
struct ClusterHunk {
    low_value: f64,
    voxel_queue: BinaryHeap<VoxelPriority>,
    cluster: Cluster
}

fn generate_1d_field(size: usize, min_value: f64, max_value: f64) -> Vec<Voxel> {
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

#[test]
fn test_get_clusters() {
    let mut voxels = generate_1d_field(6, 0.0, 1.0);
    voxels[2].value = 3.0;
    voxels[3].value = 3.5;
    assert_eq!(
        get_clusters(&mut voxels, 2.5),
        vec![vec![2, 3]]
    );
}

#[bench]
fn benchmark_get_clusters(b: &mut Bencher) {
    let mut voxels = generate_1d_field(50000, 0.1, 1.0);
    b.iter(|| get_clusters(&mut voxels, 0.5));
}

#[test]
fn test_slow_tfce() {
    let mut voxels = generate_1d_field(6, 0.0, 1.0);
    slow_tfce(&mut voxels, 50);
    println!("{:?}", voxels.iter().map(|v| v.value).collect::<Vec<f64>>());
    println!("{:?}", voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>());
}

#[bench]
fn benchmark_slow_tfce(b: &mut Bencher) {
    let mut voxels = generate_1d_field(50000, 0.1, 1.0);
    b.iter(|| slow_tfce(&mut voxels, 50));
}

#[bench]
fn benchmark_fast_tfce(b: &mut Bencher) {
    let mut voxels = generate_1d_field(50000, 0.1, 1.0);
    b.iter(|| fast_tfce(&mut voxels));
}

#[test]
fn test_build_clusters_single_element() {
    let mut voxels = vec![
        Voxel::new(1.0, vec![])
    ];
    assert_eq!(
        build_cluster_tree(&mut voxels),
        Cluster {
            voxel_indices: vec![0],
            parent_cluster_1: None,
            parent_cluster_2: None
        }
    );
}

#[test]
fn test_build_clusters_two_elements() {
    let mut voxels = vec![
        Voxel::new(1.0, vec![1]),
        Voxel::new(2.0, vec![0])
    ];
    assert_eq!(
        build_cluster_tree(&mut voxels),
        Cluster {
            voxel_indices: vec![1, 0],
            parent_cluster_1: None,
            parent_cluster_2: None
        }
    );
}

#[test]
fn test_build_clusters_three_elements_linear() {
    let mut voxels = vec![
        Voxel::new(1.0, vec![1]),
        Voxel::new(2.0, vec![0, 2]),
        Voxel::new(3.0, vec![1])
    ];
    assert_eq!(
        build_cluster_tree(&mut voxels),
        Cluster {
            voxel_indices: vec![2, 1, 0],
            parent_cluster_1: None,
            parent_cluster_2: None
        }
    );
}

#[test]
fn test_build_clusters_three_elements_hut() {
    let mut voxels = vec![
        Voxel::new(1.0, vec![1]),
        Voxel::new(3.0, vec![0, 2]),
        Voxel::new(2.0, vec![1])
    ];
    assert_eq!(
        build_cluster_tree(&mut voxels),
        Cluster {
            voxel_indices: vec![1, 2, 0],
            parent_cluster_1: None,
            parent_cluster_2: None
        }
    );
}

#[test]
fn test_build_clusters_four_elements_linear_with_peak() {
    let mut voxels = vec![
        Voxel::new(3.0, vec![1]),
        Voxel::new(2.0, vec![0, 2]),
        Voxel::new(1.0, vec![1, 3]),
        Voxel::new(2.5, vec![2])
    ];
    assert_eq!(
        build_cluster_tree(&mut voxels),
        Cluster {
            voxel_indices: vec![],
            parent_cluster_1: Some(Box::new(Cluster {
                voxel_indices: vec![0, 1, 2],
                parent_cluster_1: None,
                parent_cluster_2: None
            })),
            parent_cluster_2: Some(Box::new(Cluster {
                voxel_indices: vec![3],
                parent_cluster_1: None,
                parent_cluster_2: None
            }))
        }
    );
}

#[test]
fn test_build_clusters_five_elements_two_clusters() {
    let mut voxels = vec![
        Voxel::new(3.0, vec![1]),
        Voxel::new(2.0, vec![0, 2]),
        Voxel::new(1.0, vec![1, 3]),
        Voxel::new(2.5, vec![2, 4]),
        Voxel::new(2.0, vec![3])
    ];
    assert_eq!(
        build_cluster_tree(&mut voxels),
        Cluster {
            voxel_indices: vec![],
            parent_cluster_1: Some(Box::new(Cluster {
                voxel_indices: vec![0, 1, 2],
                parent_cluster_1: None,
                parent_cluster_2: None
            })),
            parent_cluster_2: Some(Box::new(Cluster {
                voxel_indices: vec![3, 4],
                parent_cluster_1: None,
                parent_cluster_2: None
            }))
        }
    );
}
