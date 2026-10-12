use std::{collections::HashMap, io::{self, BufRead}};

#[derive(Debug)]
pub struct BitMap {
    pub blocks: Vec<i32>,
}

#[derive(Debug, Default)]
pub struct Extent {
    pub logical_start: i32,
    pub length: i32,
    pub physical_start: i32,
}

#[derive(Debug, Default)]
pub struct ExtentRegistry {
    pub extents: Vec<Extent>,
}

#[derive(Debug, Default)]
pub struct EntryInodeRegistry {
    pub entry_inode: HashMap<String, i32>
}

#[derive(Debug)]
pub struct Directory {
    pub inode: i32,
    pub content: String,
    pub inode_content: i32,
}

#[derive(Debug, Default)]
pub struct DirectoryRegistry {
    pub directories: Vec<Directory>,
}

impl DirectoryRegistry {
    pub fn register_directory(&mut self, inode: i32, rest: String) {
        let splitted_content: Vec<&str> = rest.split('=').collect();

        if splitted_content.len() < 2 {
            return;
        }

        let content = splitted_content[0].to_string();
        let inode_content = splitted_content[1].parse::<i32>().unwrap_or(0);

        let directory_created = Directory {
            inode,
            content,
            inode_content,
        };

        self.directories.push(directory_created);
    }

    pub fn walk_directories(&self, path: String) -> Result<i32, &'static str> {
        if path == "/" {
            return Ok(1);
        }

        let path_split: Vec<&str> = path.split('/').collect();

        if let Some(last_path) = path_split.last() {
            let last_directory_metadata = self.directories
                .iter()
                .find(|directory| directory.content == *last_path)
                .ok_or("ENOENT")?;

            Ok(last_directory_metadata.inode_content)
        } else {
            Err("ENOENT")
        }
    }
}

impl EntryInodeRegistry {
    pub fn register_entry_inode(&mut self, entry: String, inode: i32) {
        self.entry_inode.insert(entry, inode);
    }

    pub fn lookup(&self, entry: String) -> Result<i32, &'static str> {
        if let Some(inode) = self.entry_inode.get(&entry).copied() {
            Ok(inode)
        } else {
            Err("ENOENT")
        }
    }
}

impl ExtentRegistry {
    pub fn register_new_extent(&mut self, logic_start: i32, length: i32, phys_start: i32) {
        let extent = Extent {
            logical_start: logic_start,
            length,
            physical_start: phys_start
        };

        self.extents.push(extent);
    }

    pub fn lookup(&self, logical: i32) -> String {
        if let Some(extent) = self.extents.iter().find(|e| {
            logical >= e.logical_start && logical < e.logical_start + e.length
        }) {
            format!("{}", extent.physical_start + (logical - extent.logical_start))
        } else {
            format!("HOLE")
        }
    }
}

impl BitMap {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
        }
    }

    pub fn with_blocks(n_blocks: usize) -> Self {
        Self {
            blocks: vec![0; n_blocks],
        }
    }

    pub fn alloc_block(&mut self) -> Result<usize, String> {
        if let Some((index, _)) = self.blocks.iter().enumerate().find(|&(_, &b)| b == 0) {
            self.blocks[index] = 1;
            Ok(index)
        } else {
            Err(format!("All blocks of defined bitmap are allocated"))
        }    
    }

    pub fn free_block(&mut self, index: i32) -> Result<&'static str, String> {
        let index = usize::try_from(index)
            .map_err(|_| format!("Invalid bitmap block index {}", index))?;
        let block = self.blocks.get_mut(index)
            .ok_or_else(|| format!("Invalid bitmap block index {}", index))?;

        if *block == 0 {
            Err(format!("Bitmap's block value at index {} is already free", index))
        } else {
            *block = 0;
            Ok("OK")
        }
    }

    pub fn used_blocks(&self) -> i32 {
        let mut counter = 0;

        for &block_value in self.blocks.iter() {
            if block_value != 0 {
                counter += 1;
            }
        }

        counter
    }
}

pub fn validate_supermagic_block_number(magic_str: &str) -> String {
    let magic_hex = u64::from_str_radix(magic_str, 16).unwrap();

    if magic_hex == u64::from_str_radix(EXT2_MAGIC_NUMBER.trim_start_matches("0x"), 16).unwrap() {
        format!("VALID")
    } else {
        format!("BAD_MAGIC got=0x{:X}", magic_hex)
    }
}

pub fn blocks_inode_reference(inode_size: i32, block_size: i32) -> String {
    let mut count_direct_blocks = 0;
    let mut count_indirect_blocks = 0;
    let mut count_double_indirect_blocks = 0;

    let num_blocks_needed = (inode_size as f32 / block_size as f32).ceil() as i32;

    if num_blocks_needed <= MAX_DIRECT {
        count_direct_blocks = num_blocks_needed;
    } 
    else if num_blocks_needed <= MAX_INDIRECT {
        count_direct_blocks = MAX_DIRECT;
        count_indirect_blocks = 1;
    } 
    else {
        count_direct_blocks = MAX_DIRECT;
        count_indirect_blocks = 1;         
        count_double_indirect_blocks = 1; 
    }

    format!("DIRECT {}\nINDIRECT {}\nDOUBLE {}\n", count_direct_blocks, count_indirect_blocks, count_double_indirect_blocks)
}

pub fn read_spans_block(block_size: &mut i32, offset: i32, length: i32) -> String {
    let bsize = *block_size;
    
    if bsize <= 0 || length <= 0 {
        return format!("0 0");
    }

    let first_block = offset / bsize;

    let last_block = (offset + length - 1) / bsize;

    format!("{} {}", first_block, last_block)
}

const EXT2_MAGIC_NUMBER: &str = "0xEF53";
const MAX_DIRECT: i32 = 12;
const MAX_INDIRECT: i32 = 12 + 1024;

fn main() {
    let stdin = io::stdin();
    let mut block_size = 0.0;
    let mut total_blocks_used = 0.0;
    let mut bitmap = BitMap::new();
    let mut extent_registry = ExtentRegistry::default();
    let mut bsize = 0;
    let mut entry_inode_registry = EntryInodeRegistry::default();
    let mut directory_registry = DirectoryRegistry::default();

    for line in stdin.lock().lines() {
        let l = line.unwrap();
        if l.is_empty() { continue; }
    
        let parts: Vec<&str> = l.split_whitespace().collect();

        match parts[0] {
            "BLOCK_SIZE" => block_size = parts[1].parse::<f32>().unwrap(),
            "FILE" => {
                let file_size = parts[1].parse::<f32>().unwrap();
                total_blocks_used += (file_size / block_size).ceil();
            },
            "CHECK" => println!("{}", validate_supermagic_block_number(parts[1])),
            "INIT" => {
                bitmap = BitMap::with_blocks(parts[1].parse::<usize>().unwrap());
                println!("OK");
            },
            "ALLOC" => {
                match bitmap.alloc_block() {
                    Ok(val) => println!("{}", val),
                    Err(e) => println!("{}", e),
                }
            },
            "FREE" => {
                match bitmap.free_block(parts[1].parse::<i32>().unwrap()) {
                    Ok(v) => println!("{}", v),
                    Err(e) => println!("{}", e),
                }
            },
            "USED" => println!("{}", bitmap.used_blocks()),
            "SIZE" => println!("{}", blocks_inode_reference(parts[1].parse().unwrap(), parts[3].parse().unwrap())),
            "EXTENT" => extent_registry.register_new_extent(parts[1].parse().unwrap(), parts[2].parse().unwrap(), 
                                                    parts[3].parse().unwrap()),
            "LOOKUP" => {
                //println!("{}", extent_registry.lookup(parts[1].parse().unwrap()));
                match entry_inode_registry.lookup(parts[1].to_string()) {
                    Ok(result) => println!("{}", result),
                    Err(e) => println!("{}", e),
                }
            },
            "BSIZE" => bsize = parts[1].parse().unwrap(),
            "READ" => println!("{}", read_spans_block(&mut bsize, parts[1].parse().unwrap(), parts[2].parse().unwrap())),
            "ENTRY" => entry_inode_registry.register_entry_inode(parts[1].to_string(), parts[2].parse().unwrap()),
            "DIR" => directory_registry.register_directory(parts[1].parse().unwrap(), parts[2].to_string()),
            "WALK" => {
                match directory_registry.walk_directories(parts[1].to_string()) {
                    Ok(inode) => println!("{}", inode),
                    Err(e) => println!("{}", e),
                }
            },
            _ => break
        }
    }

    //println!("{}", total_blocks_used as i32);
}