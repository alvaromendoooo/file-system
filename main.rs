use std::io::{self, BufRead};

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
            _ => break
        }
    }

    println!("{}", total_blocks_used as i32);
}
