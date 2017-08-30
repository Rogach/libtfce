#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(type_ascription)]

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

// benchmarks:
//   (do git commit before)
//   try using HashSet<usize> instead of Vec<usize> for simple links
//   try using HashSet<usize> or Vec<usize> instead of HashMap<u16, ...>
//   try setting levels on voxels in one pass
// batch push where possible

fn main() {
}

fn find_patches(voxels: &mut Vec<Voxel>, d: u16) -> Vec<Patch> {
    let max_value = voxels.iter().map(|v| v.value).fold(0.0, f64::max);
    let delta = max_value / (d as f64) + 0.000001;

    let mut patches: Vec<Patch> = Vec::new();
    let mut min_unvisited_index: usize = 0;
    loop {
        if min_unvisited_index >= voxels.len() {
            break;
        }
        if voxels[min_unvisited_index].patch_index.is_none() {
            let new_patch_index = patches.len();
            let mut voxel_queue: VecDeque<usize> = VecDeque::new();
            voxel_queue.push_back(min_unvisited_index);
            let target_level = (voxels[min_unvisited_index].value / delta) as u16;
            let mut patch_voxels = Vec::new();
            let mut patch_links = PatchLinks::empty();
            while let Some(v_idx) = voxel_queue.pop_front() {
                let voxel = &mut voxels[v_idx];
                match voxel.patch_index {
                    Some(patch_index) if patch_index == new_patch_index => {},
                    Some(patch_index) => {
                        patch_links.add(patches[patch_index].level, patch_index);
                        patches[patch_index].patch_links.add(target_level, new_patch_index);
                    },
                    None => {
                        if ((voxel.value / delta) as u16) == target_level {
                            voxel.patch_index = Some(new_patch_index);
                            patch_voxels.push(v_idx);
                            for linked_voxel in voxel.voxel_links.iter() {
                                voxel_queue.push_back(*linked_voxel);
                            }
                        }
                    }
                }
            }
            patches.push(Patch::new(patch_voxels, target_level, patch_links));
        }
        min_unvisited_index += 1;
    }

    patches
}

fn build_cluster_tree(voxels: &mut Vec<Voxel>, patches: &mut Vec<Patch>) {
    let max_level = patches.iter().map(|p| p.level).max().unwrap();
    let mut patches_by_level: Vec<Vec<usize>> = vec![Vec::new(); (max_level + 1) as usize];
    for pi in 0..patches.len() {
        patches_by_level[patches[pi].level as usize].push(pi);
    }
    let mut level = max_level;
    let mut clusters: Vec<Cluster> = Vec::new();
    let mut prev_clusters: Vec<usize> = Vec::new();
    loop {
        let mut min_cluster_index: usize = 0;
        let mut new_clusters: Vec<usize> = Vec::new();

        loop {
            if min_cluster_index >= prev_clusters.len() {
                break;
            }
            if !clusters[min_cluster_index].visited {
                let new_cluster_index = clusters.len();
                let mut new_cluster_size = 0;
                let mut new_patch_links = HashSet::new();
                let mut new_cluster_links = HashSet::new();
                let mut cluster_queue: VecDeque<usize> = VecDeque::new();
                let mut patch_queue: VecDeque<usize> = VecDeque::new();
                cluster_queue.push_back(clusters[min_cluster_index].index);
                while cluster_queue.is_empty() && patch_queue.is_empty() {
                    while let Some(cluster_index) = cluster_queue.pop_front() {
                        if !clusters[cluster_index].visited {
                            clusters[cluster_index].visited = true;
                            new_cluster_links.insert(cluster_index);

                            for pi in clusters[cluster_index].patch_links.iter() {
                                patch_queue.push_back(*pi);
                            }
                        }
                    }
                    while let Some(patch_index) = patch_queue.pop_front() {
                        if patches[patch_index].cluster_link.is_none() {
                            patches[patch_index].cluster_link = Some(new_cluster_index);
                            new_patch_links.insert(patch_index);
                            if let Some(cluster_index) = patches[patch_index].cluster_link {
                                cluster_queue.push_back(cluster_index);
                            }
                        }
                    }
                }

                // something shitty here - how we handle situations when level is skipped?
                // how we calculate next level size?
                unimplemented!();
            }
            min_cluster_index += 1;
        }

        // create fresh clusters for all unused patches on the level
        for pi in &patches_by_level[level as usize] {
            if patches[*pi].cluster_link.is_none() {
                let mut cluster =
                    Cluster::new(clusters.len(), level, *pi, patches[*pi].voxel_indices.len());
                cluster.visited = true;
                clusters.push(cluster);
            }
        }

        // go the next level

        if level == 0 {
            break;
        }
        level -= 1;
    }

    // after cluster tree is built,
    // assert that we have a single grand cluster at the bottom
    // traverse tree from the lowest-level cluster,
    // computing patch values and setting voxels as we go
    // (current sum is passed upwards into contained clusters)
}

#[derive(Debug, Clone)]
struct Voxel {
    voxel_links: Vec<usize>,
    patch_index: Option<usize>,
    value: f64,
    tfce_value: f64
}
impl Voxel {
    fn new(value: f64, links: Vec<usize>) -> Voxel {
        Voxel {
            voxel_links: links,
            patch_index: None,
            value: value,
            tfce_value: 0.0
        }
    }
}


#[derive(Debug, PartialEq, Eq)]
struct Patch {
    voxel_indices: Vec<usize>,
    level: u16,
    patch_links: PatchLinks,
    cluster_link: Option<usize>
}
impl Patch {
    fn new(voxels: Vec<usize>, level: u16, links: PatchLinks) -> Patch {
        Patch {
            voxel_indices: voxels,
            level: level,
            patch_links: links,
            cluster_link: None
        }
    }
}

#[derive(Debug)]
struct Cluster {
    index: usize,
    level: u16,
    patch_links: HashSet<usize>,
    cluster_links: HashSet<usize>,
    size: usize,
    visited: bool
}
impl Cluster {
    fn new(index: usize, level: u16, patch_index: usize, patch_size: usize) -> Cluster {
        let mut links = HashSet::new();
        links.insert(patch_index);
        Cluster {
            index: index,
            level: level,
            patch_links: links,
            cluster_links: HashSet::new(),
            size: patch_size,
            visited: false
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PatchLinks {
    links: HashMap<u16, HashSet<usize>>
}
impl PatchLinks {
    fn empty() -> PatchLinks {
        PatchLinks { links: HashMap::new() }
    }

    fn new(links: Vec<(u16, usize)>) -> PatchLinks {
        let mut pl = PatchLinks::empty();
        for (level, patch_index) in links {
            pl.add(level, patch_index);
        }
        pl
    }

    fn add(&mut self, level: u16, patch_index: usize) {
        self.links.entry(level).or_insert(HashSet::new()).insert(patch_index);
    }

    fn at_level(&self, level: u16) -> Option<&HashSet<usize>> {
        self.links.get(&level)
    }
}

#[test]
fn test_patches() {
    let mut voxels = vec![
        Voxel::new(1.0, vec![1]),
        Voxel::new(1.0, vec![0, 2]),
        Voxel::new(3.0, vec![1, 3]),
        Voxel::new(3.0, vec![2, 4]),
        Voxel::new(2.0, vec![3, 5]),
        Voxel::new(2.0, vec![4])
    ];

    assert_eq!(
        find_patches(&mut voxels, 3),
        vec![
            Patch::new(vec![0,1], 0, PatchLinks::new(vec![(2, 1)])),
            Patch::new(vec![2,3], 2, PatchLinks::new(vec![(0, 0), (1, 2)])),
            Patch::new(vec![4,5], 1, PatchLinks::new(vec![(2, 1)]))
        ]
    );
}
