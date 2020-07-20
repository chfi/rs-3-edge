use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

use gfa::gfa::GFAParsingConfig;
use gfa::parser::parse_gfa_stream_config;

type AdjacencyList = Vec<usize>;

type Graph = BTreeMap<usize, AdjacencyList>;

/// An adjacency list representation of a GFA graph, including the
/// maps required to go from GFA segment names to graph node indices,
/// and back
struct ALGraph {
    graph: Graph,
    inv_names: Vec<String>,
}

impl ALGraph {
    /// Constructs an adjacency list representation of the given GFA
    /// parser stream. Returns both the adjacency list and a map from GFA
    /// segment names to corresponding index in the graph.
    fn from_gfa_file(path: &PathBuf) -> ALGraph {
        let buffer = File::open(&path).unwrap();
        let reader = BufReader::new(buffer);
        let lines = &mut reader.lines();

        let conf = GFAParsingConfig {
            links: true,
            ..GFAParsingConfig::none()
        };
        let gfa_lines = parse_gfa_stream_config(lines, conf);

        let mut graph: BTreeMap<usize, AdjacencyList> = BTreeMap::new();
        let mut name_map = HashMap::new();
        let mut inv_names = Vec::new();

        let mut get_ix = |name: &str| {
            if let Some(ix) = name_map.get(name) {
                *ix
            } else {
                let ix = name_map.len();
                name_map.insert(name.to_string(), ix);
                inv_names.push(name.to_string());
                ix
            }
        };

        use gfa::gfa::Line;
        for line in gfa_lines {
            if let Line::Link(link) = line {
                let from_ix = get_ix(&link.from_segment);
                let to_ix = get_ix(&link.to_segment);

                graph.entry(from_ix).or_default().push(to_ix);
                graph.entry(to_ix).or_default().push(from_ix);
            }
        }

        ALGraph { graph, inv_names }
    }
}

#[derive(Default, Debug, Clone)]
struct State {
    degrees: Vec<isize>,
    next_sigma: Vec<usize>,
    next_on_path: Vec<usize>,
    visited: BTreeSet<usize>,
    pre: Vec<usize>,
    lowpt: Vec<usize>,
    count: usize,
    num_descendants: Vec<usize>,
    path_u: usize,
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
    fn initialize(graph: &Graph) -> State {
        let num_nodes = graph.len();

        State {
            count: 1,
            next_sigma: vec![0; num_nodes],
            next_on_path: vec![0; num_nodes],
            pre: vec![0; num_nodes],
            lowpt: vec![0; num_nodes],
            num_descendants: vec![0; num_nodes],
            degrees: vec![0; num_nodes],
            visited: BTreeSet::new(),
            sigma: BTreeMap::new(),
            path_u: 0,
        }
    }

    fn is_back_edge(&self, u: usize, v: usize) -> bool {
        self.pre[u] > self.pre[v]
    }

    fn is_null_path(&self, u: usize) -> bool {
        self.next_on_path[u] == u
    }

    fn absorb_path(&mut self, root: usize, path: usize, end: Option<usize>) {
        if Some(root) != end {
            let mut current = root;
            let mut step = path;
            while current != step {
                self.degrees[root] += self.degrees[step] - 2;
                self.next_sigma.swap(root, step);
                current = step;
                if Some(step) != end {
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

#[derive(Debug)]
enum Inst {
    Recur(usize, usize),
    Loop(usize, usize, usize),
    Return(usize, usize),
}

type InstStack = VecDeque<Inst>;

fn run_inst(
    inst: Inst,
    stack: &mut InstStack,
    state: &mut State,
    graph: &Graph,
) {
    match inst {
        Inst::Recur(w, v) => {
            state.visited.insert(w);
            state.next_sigma[w] = w;
            state.next_on_path[w] = w;
            state.pre[w] = state.count;
            state.lowpt[w] = state.count;
            state.num_descendants[w] = 1;
            state.count += 1;

            graph[&w]
                .iter()
                .rev()
                .for_each(|edge| stack.push_front(Inst::Loop(w, v, *edge)));
        }
        Inst::Loop(w, v, u) => {
            state.degrees[w] += 1;

            if !state.visited.contains(&u) {
                stack.push_front(Inst::Return(w, u));
                stack.push_front(Inst::Recur(u, w));
            } else {
                // (w, u) outgoing back-edge of w, i.e. dfs(w) > dfs(u)
                if u != v && state.is_back_edge(w, u) {
                    if state.pre[u] < state.lowpt[w] {
                        state.absorb_path(w, state.next_on_path[w], None);
                        state.next_on_path[w] = w; // P_w in paper
                        state.lowpt[w] = state.pre[u];
                    }
                // (w, u) incoming back-edge of w, i.e. dfs(u) > dfs(w)
                } else if u != v {
                    state.degrees[w] -= 2;

                    if !state.is_null_path(w) {
                        let mut parent = w;
                        let mut child = state.next_on_path[w];

                        while !state.is_null_path(parent)
                            && state.pre[child] <= state.pre[u]
                        // child must have been visited before u
                            && state.pre[u] < state.pre[child] + state.num_descendants[child]
                        // child is still an ancestor of u
                        {
                            parent = child;
                            child = state.next_on_path[child];
                        }

                        // P_w[w..u] in paper
                        state.absorb_path(
                            w,
                            state.next_on_path[w],
                            Some(parent),
                        );

                        state.next_on_path[w] = if state.is_null_path(parent) {
                            w
                        } else {
                            state.next_on_path[parent]
                        }
                    }
                }
            }
        }
        Inst::Return(w, u) => {
            state.num_descendants[w] += state.num_descendants[u];

            if state.degrees[u] <= 2 {
                state.degrees[w] += state.degrees[u] - 2;
                state.add_component(u);

                state.path_u = if state.is_null_path(u) {
                    w // P_u = w + P_u
                } else {
                    state.next_on_path[u] // P_u
                };
            } else {
                // since degree[u] != 2, u can be absorbed
                state.path_u = u;
            }

            if state.lowpt[w] <= state.lowpt[u] {
                // w + P_u in paper
                state.absorb_path(w, state.path_u, None);
            } else {
                state.lowpt[w] = state.lowpt[u];
                // P_w in paper
                state.absorb_path(w, state.next_on_path[w], None);
                state.next_on_path[w] = state.path_u;
            }
        }
    }
}

fn three_edge_connect(graph: &Graph, state: &mut State) {
    let mut stack: InstStack = VecDeque::new();

    let nodes: Vec<_> = graph.keys().collect();
    for &n in nodes {
        if !state.visited.contains(&n) {
            stack.push_front(Inst::Recur(n, 0));
            while let Some(inst) = stack.pop_front() {
                run_inst(inst, &mut stack, state, graph);
            }
            state.add_component(n);
        }
    }
}

/// Prints each component, one per row, with space-delimited GFA
/// segment names, in the BTreeMap node order
fn print_components(
    inv_name_arr: &[String],
    sigma: &BTreeMap<usize, BTreeSet<usize>>,
) {
    for (_k, component) in sigma {
        for (i, j) in component.iter().enumerate() {
            if i > 0 {
                print!("\t");
            }
            let name = &inv_name_arr[*j];
            print!("{}", name);
        }
        println!();
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    let path = PathBuf::from(&args[1]);
    let algraph = ALGraph::from_gfa_file(&path);

    let mut state = State::initialize(&algraph.graph);

    three_edge_connect(&algraph.graph, &mut state);

    print_components(&algraph.inv_names, &state.sigma);
}
