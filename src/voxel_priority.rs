use std::cmp::Ordering;

#[derive(Debug)]
pub struct VoxelPriority {
    pub value: f64,
    pub index: usize
}

impl PartialEq for VoxelPriority {
    fn eq(&self, other: &VoxelPriority) -> bool {
        self.value == other.value &&
            self.index == other.index
    }
}
impl Eq for VoxelPriority {}
impl PartialOrd for VoxelPriority {
    fn partial_cmp(&self, other: &VoxelPriority) -> Option<Ordering> {
        if self.value < other.value {
            Some(Ordering::Less)
        } else if self.value > other.value {
            Some(Ordering::Greater)
        } else {
            self.index.partial_cmp(&other.index)
        }
    }
}
impl Ord for VoxelPriority {
    fn cmp(&self, other: &VoxelPriority) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
