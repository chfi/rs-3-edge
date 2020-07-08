use gfa::gfa::GFA;
use gfa::parser::parse_gfa;

use std::env;
use std::path::PathBuf;

// struct

fn parse_input(path: &PathBuf) -> Vec<String> {
    let file = File::open(path).unwrap();
    let lines = BufReader::new(file).lines();
}

fn main() {
    let args: Vec<_> = env::args().collect();

    // let num_vertices =
    // let next_sigma_elements =
    println!("Hello, world!");
}
