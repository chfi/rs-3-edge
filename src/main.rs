use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

use std::collections::BTreeMap;

type AdjacencyList = Vec<usize>;

type Graph = BTreeMap<usize, AdjacencyList>;

fn parse_input(path: &PathBuf) -> Graph {
    let file = File::open(path).unwrap();
    let mut result = BTreeMap::new();
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let mut fields = line.split_terminator('>');
        let node = fields.next().and_then(|f| f.parse().ok()).unwrap();
        let mut list: AdjacencyList = Vec::new();
        for f in fields {
            list.push(f.parse().unwrap());
        }
        result.insert(node, list);
    }
    result
}

fn main() {
    let args: Vec<_> = env::args().collect();

    let path = PathBuf::from(&args[1]);
    let graph = parse_input(&path);
    for (k, l) in graph.iter() {
        println!("node {}", k);
        for n in l.iter() {
            print!("{}>", n);
        }
        println!();
    }
}
