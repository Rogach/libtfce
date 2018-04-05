use std::fs::File;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Seek, SeekFrom};

pub const KIND_FILE_ID: u32 = 100;
pub const KIND_DIR_POINTER: u32 = 101;
pub const KIND_BLOCK_START: u32 = 104;
pub const KIND_BLOCK_END: u32 = 105;

pub const TYPE_INT: u32 = 3;
pub const TYPE_FLOAT: u32 = 4;
pub const TYPE_STRING: u32 = 10;
pub const TYPE_ID_STRUCT: u32 = 31;
pub const TYPE_DIR_ENTRY_STRUCT: u32 = 32;

pub const BLOCK_MNE_SOURCE_SPACE: u32 = 351;
pub const KIND_MNE_SOURCE_SPACE_USED_VERTEX_COUNT: u32 = 3514;
pub const KIND_MNE_SOURCE_SPACE_USED_TRIANGLES: u32 = 3593;

#[derive(Debug, Clone)]
pub struct Tree {
    pub block_type: u32,
    pub tags: Vec<Tag>,
    pub children: Vec<Tree>
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub kind: u32,
    pub tag_type: u32,
    pub size: u32,
    pub pos: u32
}

#[derive(Debug, Clone)]
pub enum TagData {
    Int(i32),
    Float(f32),
    String(String),
    ArrayInt(Vec<Vec<i32>>),
    ArrayFloat(Vec<Vec<f32>>),
    DirEntryStruct(Vec<Tag>)
}

pub fn open(file: &mut File) -> Tree {
    let id_tag = read_tag(file, 0);
    assert_eq!(id_tag.kind, KIND_FILE_ID);
    assert_eq!(id_tag.tag_type, TYPE_ID_STRUCT);

    let dir_pointer_tag_pos = file.seek(SeekFrom::Current(0)).unwrap() as u32;
    let dir_pointer_tag = read_tag(file, dir_pointer_tag_pos);
    assert_eq!(dir_pointer_tag.kind, KIND_DIR_POINTER);
    let dir_pos = match read_tag_data(file, &dir_pointer_tag) {
        TagData::Int(dir_pos) => dir_pos,
        _ => panic!()
    };

    let dir_struct_tag = read_tag(file, dir_pos as u32);
    assert_eq!(dir_struct_tag.tag_type, TYPE_DIR_ENTRY_STRUCT);
    let dir_entries = match read_tag_data(file, &dir_struct_tag) {
        TagData::DirEntryStruct(entries) => entries,
        _ => panic!()
    };

    make_dir_tree(file, &dir_entries, 0).1
}

pub fn read_tag(file: &mut File, pos: u32) -> Tag {
    file.seek(SeekFrom::Start(pos as u64)).unwrap();

    let tag = Tag {
        kind: file.read_i32::<BigEndian>().unwrap() as u32,
        tag_type: file.read_u32::<BigEndian>().unwrap(),
        size: file.read_i32::<BigEndian>().unwrap() as u32,
        pos
    };

    let next = file.read_i32::<BigEndian>().unwrap();

    if tag.kind == KIND_FILE_ID && tag.tag_type == TYPE_ID_STRUCT {
        if next == 0 {
            file.seek(SeekFrom::Current(tag.size as i64)).unwrap();
        }
    }

    if next > 0 {
        file.seek(SeekFrom::Start(next as u64)).unwrap();
    }

    tag
}

pub fn read_tag_data(file: &mut File, tag: &Tag) -> TagData {
    file.seek(SeekFrom::Start((tag.pos+16) as u64)).unwrap();

    let matrix_coding = (tag.tag_type & 0xffff0000) >> 16;
    if matrix_coding > 0 {
        match matrix_coding {
            0x4000 => {
                // dense coding
                let pos = file.seek(SeekFrom::Current(0)).unwrap();
                file.seek(SeekFrom::Current((tag.size-4) as i64)).unwrap();
                let ndim = file.read_i32::<BigEndian>().unwrap();

                if ndim != 2 {
                    panic!("only two-dimensional matrices are supported");
                }

                file.seek(SeekFrom::Current((-(ndim+1)*4) as i64)).unwrap();
                let mut dimensions = Vec::new();
                for _ in 0..ndim {
                    dimensions.push(file.read_i32::<BigEndian>().unwrap() as usize);
                }

                file.seek(SeekFrom::Start(pos)).unwrap();

                let matrix_type = tag.tag_type & 0xffff;

                match matrix_type {
                    TYPE_INT => {
                        let mut data = Vec::with_capacity(dimensions[1]);
                        for _ in 0..dimensions[1] {
                            let mut row = Vec::new();
                            for _ in 0..dimensions[0] {
                                row.push(file.read_i32::<BigEndian>().unwrap());
                            }
                            data.push(row);
                        }
                        TagData::ArrayInt(data)
                    },
                    TYPE_FLOAT => {
                        let mut data = Vec::with_capacity(dimensions[1]);
                        for _ in 0..dimensions[1] {
                            let mut row = Vec::new();
                            for _ in 0..dimensions[0] {
                                row.push(file.read_f32::<BigEndian>().unwrap());
                            }
                            data.push(row);
                        }
                        TagData::ArrayFloat(data)
                    },
                    _ => panic!("unexpected matrix type: {}", matrix_type)
                }
            },
            _ => panic!("unexpected matrix coding: {:x}", matrix_coding)
        }
    } else {
        match tag.tag_type {
            TYPE_INT => {
                TagData::Int(file.read_i32::<BigEndian>().unwrap())
            },
            TYPE_FLOAT => {
                TagData::Float(file.read_f32::<BigEndian>().unwrap())
            },
            TYPE_STRING => {
                let mut bytes = Vec::with_capacity(tag.size as usize);
                for _ in 0..tag.size {
                    bytes.push(file.read_u8().unwrap());
                }
                TagData::String(String::from_utf8(bytes).unwrap())
            },
            TYPE_DIR_ENTRY_STRUCT => {
                let mut entries = Vec::new();

                for _ in 0..tag.size/16 {
                    entries.push(Tag {
                        kind: file.read_i32::<BigEndian>().unwrap() as u32,
                        tag_type: file.read_u32::<BigEndian>().unwrap(),
                        size: file.read_i32::<BigEndian>().unwrap() as u32,
                        pos: file.read_i32::<BigEndian>().unwrap() as u32
                    });
                }

                TagData::DirEntryStruct(entries)
            },
            _ => panic!(format!("Unexpected tag type: {}", tag.tag_type))
        }
    }
}

pub fn make_dir_tree(file: &mut File, dir: &Vec<Tag>, start: usize) -> (usize, Tree) {
    let mut i = start;
    let mut tree = Tree {
        block_type: 0,
        tags: Vec::new(),
        children: Vec::new()
    };

    if dir[start].kind == KIND_BLOCK_START {
        let entry = &dir[start];
        if let TagData::Int(block_type) = read_tag_data(file, entry) {
            tree.block_type = block_type as u32;
        } else {
            panic!();
        }
        i += 1;
    }

    while i < dir.len() {
        let entry = &dir[i];
        match entry.kind {
            KIND_BLOCK_START => {
                let (i2, child) = make_dir_tree(file, dir, i);
                i = i2;
                tree.children.push(child);
            },
            KIND_BLOCK_END => {
                return (i+1, tree);
            },
            _ => {
                tree.tags.push(Tag {
                    kind: entry.kind,
                    tag_type: entry.tag_type,
                    size: entry.size,
                    pos: entry.pos
                });
                i += 1;;
            }
        }
    }

    (i, tree)
}

pub fn find_blocks(tree: &Tree, block_type: u32) -> Vec<&Tree> {
    let mut matching_blocks = Vec::new();

    if tree.block_type == block_type {
        matching_blocks.push(tree);
    } else {
        for child in tree.children.iter() {
            matching_blocks.extend(find_blocks(child, block_type));
        }
    }

    matching_blocks
}

pub fn find_and_read_tag(file: &mut File, tree: &Tree, kind: u32) -> Option<TagData> {
    for tag in tree.tags.iter() {
        if tag.kind == kind {
            return Some(read_tag_data(file, tag));
        }
    }

    for child in tree.children.iter() {
        if let found @ Some(_) = find_and_read_tag(file, &child, kind) {
            return found;
        }
    }

    None
}
