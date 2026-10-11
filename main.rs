use std::io::{self, BufRead};

#[derive(Debug)]
pub struct BitMap {
    pub blocks: Vec<i32>,
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

const EXT2_MAGIC_NUMBER: &str = "0xEF53";

fn main() {
    let stdin = io::stdin();
    let mut block_size = 0.0;
    let mut total_blocks_used = 0.0;
    let mut bitmap = BitMap::new();

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
            _ => break
        }
    }

    //println!("{}", total_blocks_used as i32);
}
