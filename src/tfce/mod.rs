pub mod approximate_tfce;
pub use self::approximate_tfce::approximate_tfce;
#[cfg(test)] mod tfce_tests;

use std::collections::BinaryHeap;
use std::mem;
use ::voxel::Voxel;
use ::voxel_priority::VoxelPriority;
use ::permutation;

#[derive(Debug, PartialEq, Eq)]
struct Cluster {
    voxel_indices: Vec<usize>,
    size: usize,
    parent_cluster_1: Option<Box<Cluster>>,
    parent_cluster_2: Option<Box<Cluster>>
}
impl Cluster {
    fn new() -> Cluster {
        Cluster {
            voxel_indices: Vec::new(),
            size: 0,
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

pub fn tfce(voxels: &mut Vec<Voxel>, e: f64, h: f64) {
    let cluster = build_cluster_tree(voxels);
    fill_clusters(voxels, cluster, e, h);
}

pub fn run_permutation(
    mut voxels: &mut Vec<Voxel>,
    a: &Vec<Vec<f64>>,
    b: &Vec<Vec<f64>>,
    n: i32,
    e: f64,
    h: f64,
    negative: bool,
    positive: bool
) -> Vec<bool> {
    permutation::run_permutation(
        &a, &b, n,
        &mut |a, b| {
            for (v, tv) in voxels.iter_mut().zip(::ttest::ttest_rel_vec(&a, &b).into_iter()) {
                if negative {
                    v.value = tv.min(0.0).abs();
                } else if positive {
                    v.value = tv.max(0.0);
                } else {
                    v.value = tv.abs();
                }
            }
            tfce(&mut voxels, e, h);
            voxels.iter().map(|v| v.tfce_value).collect()
        }
    )
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
    voxel_queue.push(VoxelPriority { value: max_value, index: max_voxel_index });

    let mut cluster_stack: Vec<ClusterHunk> = Vec::new();

    let mut current_value = max_value;
    let mut low_value = 0.0;

    let mut current_cluster = Cluster::new();

    while !cluster_stack.is_empty() || !voxel_queue.is_empty() {
        while let Some(VoxelPriority { value, index }) = voxel_queue.pop() {
            if !visited[index] {
                if value < low_value && !cluster_stack.is_empty() {
                    let mut other_hunk = cluster_stack.pop().unwrap();

                    // merge voxel queues, smaller into bigger
                    if voxel_queue.len() < other_hunk.voxel_queue.len() {
                        mem::swap(&mut voxel_queue, &mut other_hunk.voxel_queue);
                    }
                    for i in other_hunk.voxel_queue.into_iter() {
                        voxel_queue.push(i);
                    }

                    let shared_voxel = other_hunk.cluster.voxel_indices.pop().unwrap();
                    other_hunk.cluster.size -= 1;

                    // create new cluster, pointing to two older ones
                    current_cluster = Cluster {
                        voxel_indices: vec![shared_voxel],
                        size: current_cluster.size + other_hunk.cluster.size + 1,
                        parent_cluster_1: Some(Box::new(other_hunk.cluster)),
                        parent_cluster_2: Some(Box::new(current_cluster))
                    };

                    // push current voxel onto queue
                    voxel_queue.push(VoxelPriority { value, index });

                    // set low value to older low value
                    current_value = low_value;
                    low_value = other_hunk.low_value;
                } else if value <= current_value {
                    // normal descent, still in the same cluster
                    // just mark as visited, add to current cluster
                    // and traverse voxel links
                    current_value = value;
                    visited[index] = true;
                    current_cluster.voxel_indices.push(index);
                    current_cluster.size += 1;
                    for &ni in voxels[index].voxel_links.iter() {
                        if !visited[ni] {
                            voxel_queue.push(VoxelPriority { value: voxels[ni].value, index: ni });
                        }
                    }
                } else {
                    // traversed to other hill
                    cluster_stack.push(ClusterHunk {
                        low_value: low_value,
                        voxel_queue: mem::replace(&mut voxel_queue, BinaryHeap::new()),
                        cluster: mem::replace(&mut current_cluster, Cluster::new())
                    });
                    low_value = current_value;

                    let mut max_index = index;
                    while let Some(bigger_index) = traverse_max(max_index, voxels) {
                        max_index = bigger_index;
                    }

                    current_value = voxels[max_index].value;
                    voxel_queue.push(VoxelPriority { value: current_value, index: max_index });
                }
            }
        }

        if let Some(mut other_hunk) = cluster_stack.pop() {
            // merge voxel queues, smaller into bigger
            if voxel_queue.len() < other_hunk.voxel_queue.len() {
                mem::swap(&mut voxel_queue, &mut other_hunk.voxel_queue);
            }
            for i in other_hunk.voxel_queue.into_iter() {
                voxel_queue.push(i);
            }

            let shared_voxel = other_hunk.cluster.voxel_indices.pop().unwrap();
            other_hunk.cluster.size -= 1;

            // create new cluster, pointing to two older ones
            current_cluster = Cluster {
                voxel_indices: vec![shared_voxel],
                size: current_cluster.size + other_hunk.cluster.size + 1,
                parent_cluster_1: Some(Box::new(other_hunk.cluster)),
                parent_cluster_2: Some(Box::new(current_cluster))
            };

            // set low value to older low value
            current_value = low_value;
            low_value = other_hunk.low_value;
        }
    }

    current_cluster
}

fn fill_clusters(voxels: &mut Vec<Voxel>, root_cluster: Cluster, e: f64, h: f64) {
    let mut cluster_stack = Vec::new();
    cluster_stack.push((root_cluster, 0.0f64, 0.0f64));

    while let Some((cluster, mut prev_value, mut prev_tfce_value)) = cluster_stack.pop() {
        let mut sz = cluster.size;
        for vi in cluster.voxel_indices.into_iter().rev() {
            let value = voxels[vi].value;
            let h1 = h + 1.0;
            let tfce_value =
                prev_tfce_value +
                (sz as f64).powf(e) * ((value.powf(h1) - prev_value.powf(h1)) / h1);
            voxels[vi].tfce_value = tfce_value;

            prev_value = value;
            prev_tfce_value = tfce_value;
            sz -= 1;
        }
        if let Some(box cluster) = cluster.parent_cluster_1 {
            cluster_stack.push((cluster, prev_value, prev_tfce_value));
        }
        if let Some(box cluster) = cluster.parent_cluster_2 {
            cluster_stack.push((cluster, prev_value, prev_tfce_value));
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
