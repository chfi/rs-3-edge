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

/// Constructs an adjacency list representation of the given GFA.
/// Returns both the adjacency list and a map from GFA segment names
/// to corresponding index in the graph.
fn gfa_adjacency_list(gfa: &GFA) -> (Graph, HashMap<String, usize>) {
    let mut result: Graph = BTreeMap::new();
    let mut name_map = HashMap::new();

    // for (ix, s) in gfa.segments.iter().enumerate() {
    //     trace = ix + 1;
    //     name_map.insert(s.name.clone(), ix + 1);
    // }

    let mut get_ix = |name: &str| {
        if let Some(ix) = name_map.get(name) {
            *ix
        } else {
            let ix = name_map.len();
            name_map.insert(name.to_string(), ix + 1);
            ix
        }
    };

    for link in gfa.links.iter() {
        let from = &link.from_segment;
        let to = &link.to_segment;
        // let name_len = name_map.len();

        let from_ix = get_ix(from);
        let to_ix = get_ix(to);

        if from != to {
            // Some of the GFAs have identity links
        }
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
    num_components: usize,
    sigma: BTreeMap<usize, BTreeSet<usize>>,
}

// Struct representing an iterator over a node's sigma set
struct SigmaIter<'a> {
    start: usize,
    current: usize,
    next_sigma: &'a [usize],
    done: bool,
}

impl<'a> SigmaIter<'a> {
    fn new(state: &'a State, node: usize) -> SigmaIter<'a> {
        let next_sigma = &state.next_sigma;
        SigmaIter {
            start: node,
            current: next_sigma[node],
            next_sigma: next_sigma,
            done: false,
        }
    }
}

impl<'a> Iterator for SigmaIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.done {
            None
        } else {
            if self.current == self.start {
                self.done = true;
            }

            self.current = self.next_sigma[self.current];
            Some(self.current)
        }
    }
}

impl State {
    // make num_nodes explicit as temporary fix for unused segments in GFAs
    fn initialize(graph: &Graph, num_nodes: usize) -> State {
        let nodes: Vec<_> = graph.keys().collect();
        // let num_nodes = nodes.len() + 1;

        let next_sigma = vec![0; num_nodes];
        let next_on_path = vec![0; num_nodes];
        let pre = vec![0; num_nodes];
        let lowpt = vec![0; num_nodes];
        let nd = vec![0; num_nodes];
        let degrees = vec![0; num_nodes];
        let visited = BTreeSet::new();

        State {
            next_sigma,
            next_on_path,
            pre,
            lowpt,
            nd,
            degrees,
            visited,
            path_u: 0,
            count: 1,
            num_components: 0,
            sigma: BTreeMap::new(),
        }
    }

    fn is_tree_edge(&self, u: usize, v: usize) -> bool {
        self.pre[u] < self.pre[v]
    }

    fn is_back_edge(&self, u: usize, v: usize) -> bool {
        self.pre[u] > self.pre[v]
    }

    fn is_null_path(&self, u: usize) -> bool {
        self.next_on_path[u] == u
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
                    step = self.next_on_path[step];
                }
            }
        }
    }

    fn sigma_iter<'a>(&'a self, start: usize) -> SigmaIter<'a> {
        SigmaIter::new(self, start)
    }

    fn add_component(&mut self, start: usize) {
        self.sigma.insert(start, self.sigma_iter(start).collect());
    }
}

fn three_edge_connect(graph: &Graph, state: &mut State, w: usize, v: usize) {
    // println!("w = {}", w);
    state.visited.insert(w);
    state.next_sigma[w] = w;
    state.next_on_path[w] = w;
    state.pre[w] = state.count;
    state.lowpt[w] = state.count;
    state.nd[w] = 1;
    state.count += 1;

    let edges = &graph[&w];
    // println!("{:?}", edges);

    for edge in edges {
        let u = *edge;
        state.degrees[w] += 1;

        if !state.visited.contains(&u) {
            three_edge_connect(graph, state, u, w);
            state.nd[w] += state.nd[u];

            if state.degrees[u] <= 2 {
                // println!("degrees[{}] <= 2", u);
                state.degrees[w] += state.degrees[u] - 2;
                state.num_components += 1;

                state.add_component(u);

                if state.is_null_path(u) {
                    state.path_u = w;
                } else {
                    state.path_u = state.next_on_path[u];
                }
            } else {
                // println!("degrees[{}] > 2", u);
                state.path_u = u;
            }

            if state.lowpt[w] <= state.lowpt[u] {
                // println!("lowpt[{}] <= lowpt[{}]", w, u);
                state.absorb_path(w, state.path_u, 0);
            } else {
                // println!("lowpt[{}] > lowpt[{}]", w, u);
                state.lowpt[w] = state.lowpt[u];
                state.absorb_path(w, state.next_on_path[w], 0);
                state.next_on_path[w] = state.path_u;
            }
        } else {
            // (w, u) outgoing back-edge of w, i.e. dfs(w) > dfs(u)
            if u != v && state.is_back_edge(w, u) {
                if state.pre[u] < state.lowpt[w] {
                    state.absorb_path(w, state.next_on_path[w], 0);
                    state.next_on_path[w] = w;
                    state.lowpt[w] = state.pre[u];
                }
            // (w, u) incoming back-edge of w, i.e. dfs(u) > dfs(w)
            } else if u != v {
                // println!("pre[{}] <= pre[{}]", w, u);
                state.degrees[w] -= 2;

                if !state.is_null_path(w) {
                    // println!("{}-path not null", w);
                    let mut parent = w;
                    let mut child = state.next_on_path[w];

                    while parent != child
                        // child must have been visited before u
                        && state.pre[child] <= state.pre[u]
                        // u must have been visited before the
                        // children of child?
                        && state.pre[u] <= state.pre[child] + state.nd[child] - 1
                    {
                        parent = child;
                        child = state.next_on_path[child];
                    }

                    state.absorb_path(w, state.next_on_path[w], parent);

                    if state.is_null_path(parent) {
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

    // println!("wait what");
    let path = PathBuf::from(&args[1]);

    let gfa = parse_gfa(&path).unwrap();

    // println!("okay");
    let (graph, name_map) = gfa_adjacency_list(&gfa);
    let inv_name_map: HashMap<usize, &str> =
        name_map.iter().map(|(k, v)| (*v, k.as_str())).collect();

    println!("# segments: {}", gfa.segments.len());
    println!("# links: {}", gfa.links.len());
    println!("# names: {}", name_map.len());
    println!("# nodes: {}", graph.len());

    let mut state = State::initialize(&graph, gfa.segments.len() + 1);

    let nodes: Vec<_> = graph.keys().collect();

    for &n in nodes {
        if !state.visited.contains(&n) {
            three_edge_connect(&graph, &mut state, n, 0);
            state.num_components += 1;
            state.add_component(n);
        }
    }

    println!("# of components: {}", state.num_components);

    for (_k, v) in state.sigma {
        print!("component: ");
        for s in v {
            if s != 0 {
                print!(" {}", inv_name_map[&s]);
            }
        }

        println!();
    }
}
