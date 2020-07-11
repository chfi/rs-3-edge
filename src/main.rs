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

#[derive(Default, Debug)]
struct State {
    degrees: Vec<usize>,
    next_sigma: Vec<usize>,
    next_on_path: Vec<usize>,
    visited: BTreeSet<usize>,
    pre: Vec<usize>,
    lowpt: Vec<usize>,
    count: usize,
    nd: Vec<usize>,
    outgoing_tree_edge: BTreeMap<usize, bool>,
}

impl State {
    fn initialize(graph: &Graph) -> State {
        let nodes: Vec<_> = graph.keys().collect();
        let num_nodes = nodes.len();

        let next_sigma = vec![0; num_nodes];
        let next_on_path = vec![0; num_nodes];
        let pre = vec![0; num_nodes];
        let lowpt = vec![0; num_nodes];
        let nd = vec![0; num_nodes];
        let degrees = vec![0; num_nodes];
        let visited = BTreeSet::new();

        let mut outgoing_tree_edge = BTreeMap::new();

        for i in 0..num_nodes {
            outgoing_tree_edge.insert(i, true);
        }

        State {
            next_sigma,
            next_on_path,
            pre,
            lowpt,
            nd,
            degrees,
            visited,
            outgoing_tree_edge,
            count: 1,
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
