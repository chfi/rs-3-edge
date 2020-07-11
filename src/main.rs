use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

type AdjacencyList = Vec<usize>;

type Graph = BTreeMap<usize, AdjacencyList>;

fn parse_input(path: &PathBuf) -> Graph {
    let file = File::open(path).unwrap();
    let mut result = BTreeMap::new();
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let mut fields = line.split_terminator('>');
        let node = fields.next().and_then(|f| f.parse().ok()).unwrap();
        let list = fields.map(|f| f.parse().unwrap()).collect();
        result.insert(node, list);
    }
    result
}

fn absorb_path(
    root: usize,
    path: usize,
    end: usize,
    degrees: &mut [usize],
    next_sigma: &mut [usize],
    next_on_path: &mut [usize],
) {
    let mut current = root;
    let mut path = path; // note!

    if current != path && current != end {
        while current != path {
            degrees[root] += degrees[path] - 2;

            next_sigma.swap(root, path);
            // current = next_sigma[root];
            // next_sigma[root] = next_sigma[path];
            // next_sigma[path] = current;

            current = path;
            if path != end {
                path = next_on_path[path];
            }
        }
    }
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
