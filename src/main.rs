#![feature(test)]

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

extern crate rand;
extern crate test;

use std::collections::VecDeque;
use rand::{Rng, StdRng, SeedableRng};
use test::Bencher;

fn main() {

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
    let mut voxels = generate_1d_field(10000, 0.1, 1.0);
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
    let mut voxels = generate_1d_field(10000, 0.1, 1.0);
    b.iter(|| slow_tfce(&mut voxels, 50));
}


// benchmarks:
//   (do git commit before)
//   try using HashSet<usize> instead of Vec<usize> for simple links
//   try using HashSet<usize> or Vec<usize> instead of HashMap<u16, ...>
//   try setting levels on voxels in one pass
// batch push where possible

// fn find_patches(voxels: &mut Vec<Voxel>, d: u16) -> Vec<Patch> {
//     let max_value = voxels.iter().map(|v| v.value).fold(0.0, f64::max);
//     let delta = max_value / (d as f64) + 0.000001;

//     let mut patches: Vec<Patch> = Vec::new();
//     let mut min_unvisited_index: usize = 0;
//     loop {
//         if min_unvisited_index >= voxels.len() {
//             break;
//         }
//         if voxels[min_unvisited_index].patch_index.is_none() {
//             let new_patch_index = patches.len();
//             let mut voxel_queue: VecDeque<usize> = VecDeque::new();
//             voxel_queue.push_back(min_unvisited_index);
//             let target_level = (voxels[min_unvisited_index].value / delta) as u16;
//             let mut patch_voxels = Vec::new();
//             let mut patch_links = PatchLinks::empty();
//             while let Some(v_idx) = voxel_queue.pop_front() {
//                 let voxel = &mut voxels[v_idx];
//                 match voxel.patch_index {
//                     Some(patch_index) if patch_index == new_patch_index => {},
//                     Some(patch_index) => {
//                         patch_links.add(patches[patch_index].level, patch_index);
//                         patches[patch_index].patch_links.add(target_level, new_patch_index);
//                     },
//                     None => {
//                         if ((voxel.value / delta) as u16) == target_level {
//                             voxel.patch_index = Some(new_patch_index);
//                             patch_voxels.push(v_idx);
//                             for linked_voxel in voxel.voxel_links.iter() {
//                                 voxel_queue.push_back(*linked_voxel);
//                             }
//                         }
//                     }
//                 }
//             }
//             patches.push(Patch::new(patch_voxels, target_level, patch_links));
//         }
//         min_unvisited_index += 1;
//     }

//     patches
// }

// fn build_cluster_tree(voxels: &mut Vec<Voxel>, patches: &mut Vec<Patch>) {
//     let max_level = patches.iter().map(|p| p.level).max().unwrap();
//     let mut patches_by_level: Vec<Vec<usize>> = vec![Vec::new(); (max_level + 1) as usize];
//     for pi in 0..patches.len() {
//         patches_by_level[patches[pi].level as usize].push(pi);
//     }
//     let mut level = max_level;
//     let mut clusters: Vec<Cluster> = Vec::new();
//     let mut prev_clusters: Vec<usize> = Vec::new();
//     loop {
//         let mut min_cluster_index: usize = 0;
//         let mut new_clusters: Vec<usize> = Vec::new();

//         loop {
//             if min_cluster_index >= prev_clusters.len() {
//                 break;
//             }
//             if !clusters[min_cluster_index].visited {
//                 let new_cluster_index = clusters.len();
//                 let mut new_cluster_size = 0;
//                 let mut new_patch_links = HashSet::new();
//                 let mut new_cluster_links = HashSet::new();
//                 let mut cluster_queue: VecDeque<usize> = VecDeque::new();
//                 let mut patch_queue: VecDeque<usize> = VecDeque::new();
//                 cluster_queue.push_back(clusters[min_cluster_index].index);
//                 while cluster_queue.is_empty() && patch_queue.is_empty() {
//                     while let Some(cluster_index) = cluster_queue.pop_front() {
//                         if !clusters[cluster_index].visited {
//                             clusters[cluster_index].visited = true;
//                             new_cluster_links.insert(cluster_index);

//                             for pi in clusters[cluster_index].patch_links.iter() {
//                                 patch_queue.push_back(*pi);
//                             }
//                         }
//                     }
//                     while let Some(patch_index) = patch_queue.pop_front() {
//                         if patches[patch_index].cluster_link.is_none() {
//                             patches[patch_index].cluster_link = Some(new_cluster_index);
//                             new_patch_links.insert(patch_index);
//                             if let Some(cluster_index) = patches[patch_index].cluster_link {
//                                 cluster_queue.push_back(cluster_index);
//                             }
//                         }
//                     }
//                 }

//                 // something shitty here - how we handle situations when level is skipped?
//                 // how we calculate next level size?
//                 unimplemented!();
//             }
//             min_cluster_index += 1;
//         }

//         // create fresh clusters for all unused patches on the level
//         for pi in &patches_by_level[level as usize] {
//             if patches[*pi].cluster_link.is_none() {
//                 let mut cluster =
//                     Cluster::new(clusters.len(), level, *pi, patches[*pi].voxel_indices.len());
//                 cluster.visited = true;
//                 clusters.push(cluster);
//             }
//         }

//         // go the next level

//         if level == 0 {
//             break;
//         }
//         level -= 1;
//     }

//     // after cluster tree is built,
//     // assert that we have a single grand cluster at the bottom
//     // traverse tree from the lowest-level cluster,
//     // computing patch values and setting voxels as we go
//     // (current sum is passed upwards into contained clusters)
// }

// #[derive(Debug, Clone)]
// struct Voxel {
//     voxel_links: Vec<usize>,
//     patch_index: Option<usize>,
//     value: f64,
//     tfce_value: f64
// }
// impl Voxel {
//     fn new(value: f64, links: Vec<usize>) -> Voxel {
//         Voxel {
//             voxel_links: links,
//             patch_index: None,
//             value: value,
//             tfce_value: 0.0
//         }
//     }
// }


// #[derive(Debug, PartialEq, Eq)]
// struct Patch {
//     voxel_indices: Vec<usize>,
//     level: u16,
//     patch_links: PatchLinks,
//     cluster_link: Option<usize>
// }
// impl Patch {
//     fn new(voxels: Vec<usize>, level: u16, links: PatchLinks) -> Patch {
//         Patch {
//             voxel_indices: voxels,
//             level: level,
//             patch_links: links,
//             cluster_link: None
//         }
//     }
// }

// #[derive(Debug)]
// struct Cluster {
//     index: usize,
//     level: u16,
//     patch_links: HashSet<usize>,
//     cluster_links: HashSet<usize>,
//     size: usize,
//     visited: bool
// }
// impl Cluster {
//     fn new(index: usize, level: u16, patch_index: usize, patch_size: usize) -> Cluster {
//         let mut links = HashSet::new();
//         links.insert(patch_index);
//         Cluster {
//             index: index,
//             level: level,
//             patch_links: links,
//             cluster_links: HashSet::new(),
//             size: patch_size,
//             visited: false
//         }
//     }
// }

// #[derive(Debug, PartialEq, Eq)]
// struct PatchLinks {
//     links: HashMap<u16, HashSet<usize>>
// }
// impl PatchLinks {
//     fn empty() -> PatchLinks {
//         PatchLinks { links: HashMap::new() }
//     }

//     fn new(links: Vec<(u16, usize)>) -> PatchLinks {
//         let mut pl = PatchLinks::empty();
//         for (level, patch_index) in links {
//             pl.add(level, patch_index);
//         }
//         pl
//     }

//     fn add(&mut self, level: u16, patch_index: usize) {
//         self.links.entry(level).or_insert(HashSet::new()).insert(patch_index);
//     }

//     fn at_level(&self, level: u16) -> Option<&HashSet<usize>> {
//         self.links.get(&level)
//     }
// }

// #[test]
// fn test_patches() {
//     let mut voxels = vec![
//         Voxel::new(1.0, vec![1]),
//         Voxel::new(1.0, vec![0, 2]),
//         Voxel::new(3.0, vec![1, 3]),
//         Voxel::new(3.0, vec![2, 4]),
//         Voxel::new(2.0, vec![3, 5]),
//         Voxel::new(2.0, vec![4])
//     ];

//     assert_eq!(
//         find_patches(&mut voxels, 3),
//         vec![
//             Patch::new(vec![0,1], 0, PatchLinks::new(vec![(2, 1)])),
//             Patch::new(vec![2,3], 2, PatchLinks::new(vec![(0, 0), (1, 2)])),
//             Patch::new(vec![4,5], 1, PatchLinks::new(vec![(2, 1)]))
//         ]
//     );
// }
