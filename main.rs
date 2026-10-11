use std::io::{self, BufRead};

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
            _ => break
        }
    }
    //println!("{}", total_blocks_used as i32);
}
