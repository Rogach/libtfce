use super::*;
use ::field::generate_1d_field;
use test::Bencher;

#[bench]
fn benchmark_tfce(b: &mut Bencher) {
    let mut voxels = generate_1d_field(10000, 0.1, 1.0);
    b.iter(|| tfce(&mut voxels));
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
