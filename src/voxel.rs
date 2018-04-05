#[derive(Debug, Clone)]
pub struct Voxel {
    pub voxel_links: Vec<usize>,
    pub value: f64,
    pub tfce_value: f64
}

impl Voxel {
    pub fn new(value: f64, links: Vec<usize>) -> Voxel {
        Voxel {
            voxel_links: links,
            value,
            tfce_value: 0.0
        }
    }
}
