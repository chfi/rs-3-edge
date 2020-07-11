use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

type AdjacencyList = Vec<usize>;

fn parse_input(path: &PathBuf) -> Vec<AdjacencyList> {
    let file = File::open(path).unwrap();
    let mut result = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let fields = line.split_terminator('>');
        let mut list = Vec::new();
        for f in fields {
            list.push(f.parse().unwrap());
        }
        result.push(list);
    }
    result
}

fn main() {
    let args: Vec<_> = env::args().collect();

    let path = PathBuf::from(&args[1]);
    let adj = parse_input(&path);
    println!("{}", adj.len());
}
