use std::io::{self, BufRead};

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
    let mut first_block = 0;
    let mut last_block = 0;

    if offset < bsize && length + offset <= bsize {
        first_block = 0;
        last_block = 0;
    } else if offset < bsize && length + offset > bsize {
        last_block += 1;
    } else {
        first_block += 1;
        last_block += 1;
    }

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
            "LOOKUP" => println!("{}", extent_registry.lookup(parts[1].parse().unwrap())),
            "BSIZE" => bsize = parts[1].parse().unwrap(),
            "READ" => println!("{}", read_spans_block(&mut bsize, parts[1].parse().unwrap(), parts[2].parse().unwrap())),
            _ => break
        }
    }

    //println!("{}", total_blocks_used as i32);
}