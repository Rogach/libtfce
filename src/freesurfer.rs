use ::fiff;
use ::voxel::Voxel;

use std::fs::File;

pub fn read_source_space_to_graph(filename: &str) -> Vec<Voxel> {
    let mut file =
        File::open(&filename).expect(&format!("failed to open source space file {}", &filename));

    let tree = fiff::open(&mut file);

    let source_spaces = fiff::find_blocks(&tree, fiff::BLOCK_MNE_SOURCE_SPACE);

    let lh_vertex_count = read_vertex_count(&mut file, source_spaces[0]);
    let rh_vertex_count = read_vertex_count(&mut file, source_spaces[1]);

    let lh_triangles = read_triangles(&mut file, source_spaces[0]);
    let rh_triangles = read_triangles(&mut file, source_spaces[1]);

    triangles_to_graph(lh_vertex_count, lh_triangles, rh_vertex_count, rh_triangles)
}

fn read_vertex_count(file: &mut File, source_space: &fiff::Tree) -> usize {
    match fiff::find_and_read_tag(file, source_space, fiff::KIND_MNE_SOURCE_SPACE_USED_VERTEX_COUNT) {
        Some(fiff::TagData::Int(count)) => (count + 1) as usize,
        _ => panic!()
    }
}

fn read_triangles(file: &mut File, source_space: &fiff::Tree) -> Vec<Vec<i32>> {
    match fiff::find_and_read_tag(file, source_space, fiff::KIND_MNE_SOURCE_SPACE_USED_TRIANGLES) {
        Some(fiff::TagData::ArrayInt(triangles)) => triangles,
        _ => panic!()
    }
}

fn triangles_to_graph(
    lh_vertex_count: usize,
    lh_triangles: Vec<Vec<i32>>,
    rh_vertex_count: usize,
    rh_triangles: Vec<Vec<i32>>
) -> Vec<Voxel> {
    let mut voxels = Vec::with_capacity(lh_vertex_count + rh_vertex_count);

    for _ in 0..(lh_vertex_count + rh_vertex_count) {
        voxels.push(Voxel::new(0.0, Vec::with_capacity(5)));
    }

    for triangle in lh_triangles.into_iter() {
        for src_vertex in triangle.iter() {
            let src_vertex_index = (*src_vertex - 1) as usize;
            let mut links = &mut voxels[src_vertex_index].voxel_links;
            for dst_vertex in triangle.iter() {
                let dst_vertex_index = (*dst_vertex - 1) as usize;
                if src_vertex != dst_vertex && !links.contains(&dst_vertex_index) {
                    links.push(dst_vertex_index);
                }
            }
        }
    }

    for triangle in rh_triangles.into_iter() {
        for src_vertex in triangle.iter() {
            let src_vertex_index = lh_vertex_count + (*src_vertex - 1) as usize;
            let mut links = &mut voxels[src_vertex_index].voxel_links;
            for dst_vertex in triangle.iter() {
                let dst_vertex_index = lh_vertex_count + (*dst_vertex - 1) as usize;
                if src_vertex != dst_vertex && !links.contains(&dst_vertex_index) {
                    links.push(dst_vertex_index);
                }
            }
        }
    }

    voxels
}

pub fn extend_graph_into_time(voxels: Vec<Voxel>, n_times: usize) -> Vec<Voxel> {
    let n_voxels = voxels.len();
    let mut new_voxels = Vec::with_capacity(n_voxels * n_times);

    for t in 0..n_times {
        for n in 0..n_voxels {
            let mut v = voxels[n].clone();

            for l in 0..v.voxel_links.len() {
                v.voxel_links[l] += t * n_voxels;
            }

            if t != n_times - 1 {
                v.voxel_links.push((t + 1) * n_voxels + n);
            }

            if t != 0 {
                v.voxel_links.push((t - 1) * n_voxels + n);
            }

            new_voxels.push(v);
        }
    }

    new_voxels
}
