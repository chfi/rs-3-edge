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
    degrees: &mut [usize],
    next_sigma: &mut [usize],
    next_on_path: &mut [usize],
    root: usize,
    path: usize,
    end: usize,
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
    path_u: usize,
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
        let outgoing_tree_edge = (0..num_nodes).map(|i| (i, true)).collect();

        State {
            next_sigma,
            next_on_path,
            pre,
            lowpt,
            nd,
            degrees,
            visited,
            outgoing_tree_edge,
            path_u: 0,
            count: 1,
        }
    }
}

fn three_edge_connect(graph: &Graph, state: &mut State, w: usize, v: usize) {
    state.visited.insert(w);
    state.next_sigma[w] = w;
    state.next_on_path[w] = w;
    state.pre[w] = state.count;
    state.lowpt[w] = state.count;
    state.count += 1;

    let edges = &graph[&w];

    for edge in edges {
        let u = *edge;
        state.degrees[w] += 1;

        if !state.visited.contains(&u) {
            three_edge_connect(graph, state, u, w);
            state.nd[w] += state.nd[u];

            if state.degrees[u] <= 2 {
                state.degrees[w] += state.degrees[u] - 2;

                if state.next_on_path[u] == u {
                    state.path_u = w;
                } else {
                    state.path_u = state.next_on_path[u];
                }
            } else {
                state.path_u = u;
            }

            if state.lowpt[w] <= state.lowpt[u] {
                absorb_path(
                    &mut state.degrees,
                    &mut state.next_sigma,
                    &mut state.next_on_path,
                    w,
                    state.path_u,
                    0,
                );
            } else {
                state.lowpt[w] = state.lowpt[u];
                let next_on_w = state.next_on_path[w];
                absorb_path(
                    &mut state.degrees,
                    &mut state.next_sigma,
                    &mut state.next_on_path,
                    w,
                    next_on_w,
                    0,
                );
                state.next_on_path[w] = state.path_u;
            }
        } else {
            if u == v && state.outgoing_tree_edge[&w] {
                state.outgoing_tree_edge.insert(w, false);
            } else if state.pre[w] > state.pre[u] {
                let next_on_w = state.next_on_path[w];
                if state.pre[u] < state.lowpt[w] {
                    absorb_path(
                        &mut state.degrees,
                        &mut state.next_sigma,
                        &mut state.next_on_path,
                        w,
                        next_on_w,
                        0,
                    );
                    state.next_on_path[w] = w;
                    state.lowpt[w] = state.pre[u];
                }
            } else {
                state.degrees[w] -= 2;

                if state.next_on_path[w] != w {
                    let mut parent = w;
                    let mut child = state.next_on_path[w];

                    while parent != child
                        && state.pre[child] <= state.pre[u]
                        && state.pre[u] <= state.pre[child] + state.nd[child] - 1
                    {
                        parent = child;
                        child = state.next_on_path[child];
                    }

                    let next_on_w = state.next_on_path[w];
                    absorb_path(
                        &mut state.degrees,
                        &mut state.next_sigma,
                        &mut state.next_on_path,
                        w,
                        next_on_w,
                        parent,
                    );

                    if parent == state.next_on_path[parent] {
                        state.next_on_path[w] = w;
                    } else {
                        state.next_on_path[w] = state.next_on_path[parent];
                    }
                }
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
