use std::fs::File;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub struct Stc {
    time_min: f32,
    time_step: f32,
    pub vertex_count: usize,
    vertex_indices: Vec<u32>,
    pub time_count: usize,
    pub data: Vec<Vec<f64>>
}

pub fn read(filename: &str) -> Stc {
    let mut file =
        File::open(&filename).expect(&format!("failed to open source space file {}", &filename));

    let time_min = file.read_f32::<BigEndian>().unwrap();
    let time_step = file.read_f32::<BigEndian>().unwrap();

    let vertex_count = file.read_u32::<BigEndian>().unwrap() as usize;
    let mut vertex_indices = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        vertex_indices.push(file.read_u32::<BigEndian>().unwrap());
    }

    let time_count = file.read_u32::<BigEndian>().unwrap() as usize;

    let mut data = Vec::with_capacity(time_count);

    for _ in 0..time_count {
        let mut row = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            row.push(file.read_f32::<BigEndian>().unwrap() as f64);
        }
        data.push(row);
    }

    Stc {
        time_min,
        time_step,
        vertex_count,
        vertex_indices,
        time_count,
        data
    }
}

pub fn concat_pair(stc_lh: &Stc, stc_rh: &Stc) -> Vec<f64> {
    let time_count = stc_lh.time_count;
    let vertex_count = stc_lh.vertex_count + stc_rh.vertex_count;

    let mut data = Vec::with_capacity(time_count * vertex_count);

    for t in 0..time_count {
        data.extend(&stc_lh.data[t]);
        data.extend(&stc_rh.data[t]);
    }

    data
}

pub fn write(filename: &str, stc: Stc) {
    let mut file =
        File::create(&filename).expect(&format!("failed to open source space file {}", &filename));

    file.write_f32::<BigEndian>(stc.time_min).unwrap();
    file.write_f32::<BigEndian>(stc.time_step).unwrap();

    file.write_u32::<BigEndian>(stc.vertex_count as u32).unwrap();
    for i in stc.vertex_indices.into_iter() {
        file.write_u32::<BigEndian>(i).unwrap();
    }

    file.write_u32::<BigEndian>(stc.time_count as u32).unwrap();

    for row in stc.data.into_iter() {
        for v in row.into_iter() {
            file.write_f32::<BigEndian>(v as f32).unwrap();
        }
    }
}
