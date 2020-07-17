use std::env;
use std::path::PathBuf;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;

use gfa::gfa::GFA;
use gfa::parser::parse_gfa;

type AdjacencyList = Vec<usize>;

type Graph = BTreeMap<usize, AdjacencyList>;

/// An adjacency list representation of a GFA graph, including the
/// maps required to go from GFA segment names to graph node indices,
/// and back
struct ALGraph {
    graph: Graph,
    name_map: HashMap<String, usize>,
    inv_name_map: HashMap<usize, String>,
}

impl ALGraph {
    /// Constructs an adjacency list representation of the given GFA.
    /// Returns both the adjacency list and a map from GFA segment names
    /// to corresponding index in the graph.
    pub fn from_gfa(gfa: &GFA) -> ALGraph {
        let mut graph: BTreeMap<usize, AdjacencyList> = BTreeMap::new();
        let mut name_map = HashMap::new();
        let mut inv_name_map = HashMap::new();

        let mut get_ix = |name: &str| {
            if let Some(ix) = name_map.get(name) {
                *ix
            } else {
                let ix = name_map.len();
                name_map.insert(name.to_string(), ix);
                inv_name_map.insert(ix, name.to_string());
                ix
            }
        };

        for link in gfa.links.iter() {
            let from = &link.from_segment;
            let to = &link.to_segment;

            let from_ix = get_ix(from);
            let to_ix = get_ix(to);

            graph.entry(from_ix).or_default().push(to_ix);
            graph.entry(to_ix).or_default().push(from_ix);
        }

        ALGraph {
            graph,
            name_map,
            inv_name_map,
        }
    }
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
        let num_nodes = graph.len() + 1;

        State {
            next_sigma: vec![0; num_nodes],
            next_on_path: vec![0; num_nodes],
            pre: vec![0; num_nodes],
            lowpt: vec![0; num_nodes],
            num_descendants: vec![0; num_nodes],
            degrees: vec![0; num_nodes],
            visited: BTreeSet::new(),
            path_u: 0,
            count: 1,
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

    fn next_step(&self, step: usize) -> Option<usize> {
        let next = self.next_on_path[step];
        if next != step {
            Some(next)
        } else {
            None
        }
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

fn three_edge_connect(graph: &Graph, state: &mut State, w: usize, v: usize) {
    state.visited.insert(w);
    state.next_sigma[w] = w;
    state.next_on_path[w] = w;
    state.pre[w] = state.count;
    state.lowpt[w] = state.count;
    state.num_descendants[w] = 1;
    state.count += 1;

    let edges = &graph[&w];

    for edge in edges {
        let u = *edge;
        state.degrees[w] += 1;

        if !state.visited.contains(&u) {
            three_edge_connect(graph, state, u, w);
            state.num_descendants[w] += state.num_descendants[u];

            if state.degrees[u] <= 2 {
                state.degrees[w] += state.degrees[u] - 2;

                state.add_component(u);

                if state.is_null_path(u) {
                    state.path_u = w;
                } else {
                    state.path_u = state.next_on_path[u];
                }
            } else {
                state.path_u = u;
            }

            if state.lowpt[w] <= state.lowpt[u] {
                state.absorb_path(w, state.path_u, None);
            } else {
                state.lowpt[w] = state.lowpt[u];
                state.absorb_path(w, state.next_on_path[w], None);
                state.next_on_path[w] = state.path_u;
            }
        } else {
            // (w, u) outgoing back-edge of w, i.e. dfs(w) > dfs(u)
            if u != v && state.is_back_edge(w, u) {
                if state.pre[u] < state.lowpt[w] {
                    state.absorb_path(w, state.next_on_path[w], None);
                    state.next_on_path[w] = w;
                    state.lowpt[w] = state.pre[u];
                }
            // (w, u) incoming back-edge of w, i.e. dfs(u) > dfs(w)
            } else if u != v {
                state.degrees[w] -= 2;

                if !state.is_null_path(w) {
                    let mut parent = w;
                    let mut child = state.next_on_path[w];

                    while parent != child
                        // child must have been visited before u
                        && state.pre[child] <= state.pre[u]
                        // u must have been visited before the
                        // children of child?
                        && state.pre[u] <= state.pre[child] +
                                           state.num_descendants[child] - 1
                    {
                        parent = child;
                        child = state.next_on_path[child];
                    }

                    state.absorb_path(w, state.next_on_path[w], Some(parent));

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

    let path = PathBuf::from(&args[1]);

    let gfa = parse_gfa(&path).unwrap();

    let algraph = ALGraph::from_gfa(&gfa);

    println!("# segments: {}", gfa.segments.len());
    println!("# links: {}", gfa.links.len());
    println!("# names: {}", algraph.name_map.len());
    println!("# nodes: {}", algraph.graph.len());

    let mut state = State::initialize(&algraph.graph);

    let nodes: Vec<_> = algraph.graph.keys().collect();

    for &n in nodes {
        if !state.visited.contains(&n) {
            three_edge_connect(&algraph.graph, &mut state, n, 0);
            state.add_component(n);
        }
    }

    println!("# of components: {}", state.sigma.len());

    for (_k, v) in state.sigma {
        print!("component: ");
        for s in v {
            if s != 0 {
                print!(" {}", algraph.inv_name_map[&s]);
            }
        }

        println!();
    }
}
