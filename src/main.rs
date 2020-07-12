use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;

use gfa::gfa::GFA;
use gfa::parser::parse_gfa;

type AdjacencyList = Vec<usize>;

type Graph = BTreeMap<usize, AdjacencyList>;

fn parse_input(path: &PathBuf) -> Graph {
    let file = File::open(path).unwrap();
    let mut result = BTreeMap::new();
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let mut fields = line.split_terminator('>');
        let node: usize = fields.next().and_then(|f| f.parse().ok()).unwrap();
        let list: Vec<_> = fields.map(|f| f.parse().unwrap()).collect();

        // The original uses reversed adjacency lists, but the results
        // should be the same
        // list.reverse();

        result.insert(node, list);
    }
    result
}

/// Constructs an adjacency list representation of the given GFA.
/// Returns both the adjacency list and a map from GFA segment names
/// to corresponding index in the graph.
fn gfa_adjacency_list(gfa: &GFA) -> (Graph, HashMap<String, usize>) {
    let mut result: Graph = BTreeMap::new();
    let mut name_map = HashMap::new();

    for (ix, s) in gfa.segments.iter().enumerate() {
        name_map.insert(s.name.clone(), ix + 1);
    }

    for link in gfa.links.iter() {
        let from = &link.from_segment;
        let to = &link.to_segment;
        let from_ix = name_map[from];
        let to_ix = name_map[to];

        result.entry(from_ix).or_default().push(to_ix);
        result.entry(to_ix).or_default().push(from_ix);
    }

    (result, name_map)
}

#[derive(Default, Debug)]
struct State {
    degrees: Vec<isize>,
    next_sigma: Vec<usize>,
    next_on_path: Vec<usize>,
    visited: BTreeSet<usize>,
    pre: Vec<usize>,
    lowpt: Vec<usize>,
    count: usize,
    nd: Vec<usize>,
    path_u: usize,
    outgoing_tree_edge: BTreeMap<usize, bool>,
    num_components: usize,
    sigma: BTreeMap<usize, BTreeSet<usize>>,
}

impl State {
    fn initialize(graph: &Graph) -> State {
        let nodes: Vec<_> = graph.keys().collect();
        let num_nodes = nodes.len() + 1;

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
            num_components: 0,
            sigma: BTreeMap::new(),
        }
    }

    fn absorb_path(&mut self, root: usize, path: usize, end: usize) {
        let mut current = root;
        let mut step = path;

        if current != step && current != end {
            while current != step {
                self.degrees[root] += self.degrees[step] - 2;

                self.next_sigma.swap(root, step);

                current = step;
                if step != end {
                    println!("does this happen?");
                    step = self.next_on_path[step];
                }
            }
        }
    }

    fn sigma_path(&self, start: usize) -> Vec<usize> {
        let mut u = self.next_sigma[start];
        let mut steps = vec![u];
        while u != start {
            u = self.next_sigma[u];
            steps.push(u);
        }
        steps
    }

    fn node_path(&self, start: usize) -> Vec<usize> {
        let mut u = self.next_on_path[start];
        let mut steps = vec![u];
        while u != start {
            u = self.next_on_path[u];
            steps.push(u);
        }
        steps
    }

    fn add_component(&mut self, start: usize) {
        self.sigma
            .insert(start, self.sigma_path(start).into_iter().collect());
    }
}

fn three_edge_connect(graph: &Graph, state: &mut State, w: usize, v: usize) {
    state.visited.insert(w);
    state.next_sigma[w] = w;
    state.next_on_path[w] = w;
    state.pre[w] = state.count;
    state.lowpt[w] = state.count;
    state.nd[w] = 1;
    state.count += 1;

    let edges = &graph[&w];

    for edge in edges {
        let u = *edge;
        state.degrees[w] += 1;

        if !state.visited.contains(&u) {
            three_edge_connect(graph, state, u, w);
            state.nd[w] += state.nd[u];

            if state.degrees[u] <= 2 {
                println!("degrees[{}] <= 2", u);
                state.degrees[w] += state.degrees[u] - 2;
                state.num_components += 1;

                state.add_component(u);

                if state.next_on_path[u] == u {
                    state.path_u = w;
                } else {
                    state.path_u = state.next_on_path[u];
                }
            } else {
                println!("degrees[{}] > 2", u);
                state.path_u = u;
            }

            if state.lowpt[w] <= state.lowpt[u] {
                println!("lowpt[{}] <= lowpt[{}]", w, u);
                state.absorb_path(w, state.path_u, 0);
            } else {
                println!("lowpt[{}] > lowpt[{}]", w, u);
                state.lowpt[w] = state.lowpt[u];
                state.absorb_path(w, state.next_on_path[w], 0);
                state.next_on_path[w] = state.path_u;
            }
        } else {
            if u == v && state.outgoing_tree_edge[&w] {
                state.outgoing_tree_edge.insert(w, false);
            } else if state.pre[w] > state.pre[u] {
                println!("pre[{}] > pre[{}]", w, u);
                if state.pre[u] < state.lowpt[w] {
                    state.absorb_path(w, state.next_on_path[w], 0);
                    state.next_on_path[w] = w;
                    state.lowpt[w] = state.pre[u];
                }
            } else {
                println!("pre[{}] <= pre[{}]", w, u);
                state.degrees[w] -= 2;

                if state.next_on_path[w] != w {
                    println!("{}-path not null", w);
                    let mut parent = w;
                    let mut child = state.next_on_path[w];

                    while parent != child
                        && state.pre[child] <= state.pre[u]
                        && state.pre[u] <= state.pre[child] + state.nd[child] - 1
                    {
                        parent = child;
                        child = state.next_on_path[child];
                    }

                    state.absorb_path(w, state.next_on_path[w], parent);

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

    let gfa = parse_gfa(&path).unwrap();
    let (graph, name_map) = gfa_adjacency_list(&gfa);

    for (n, ix) in name_map.iter() {
        println!("{} -> {}", n, ix);
    }
    println!();

    println!("{}", name_map.len());
    for (k, l) in graph.iter() {
        print!("{}", k);
        for n in l.iter() {
            print!(">{}", n);
        }
        println!();
    }

    /*
    let graph = parse_input(&path);
    let mut state = State::initialize(&graph);

    let nodes: Vec<_> = graph.keys().collect();

    for (k, l) in graph.iter() {
        println!("node {}", k);
        for n in l.iter() {
            print!("{}>", n);
        }
        println!();
    }

    for &n in nodes {
        if !state.visited.contains(&n) {
            three_edge_connect(&graph, &mut state, n, 0);
            state.num_components += 1;
            state.add_component(n);
        }
    }

    println!("# of components: {}", state.num_components);

    for (k, v) in state.sigma {
        println!("start: {}", k);
        for s in v {
            println!("\t{}", s);
        }
    }
    */
}
