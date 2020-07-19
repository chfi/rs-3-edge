use std::env;
use std::path::PathBuf;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::VecDeque;

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
        // println!("sigma - {}", self.next_sigma.len())
        // println!("before iter");
        let x: BTreeSet<usize> = self.sigma_iter(start).collect();
        // println!("length: {}", x.len());
        self.sigma.insert(start, x);
    }
}

// struct Stack {
//     inst: Inst,
// };

type Stack2 = VecDeque<Inst>;

#[derive(Debug)]
enum Inst {
    Init(usize, usize),
    Loop(usize, usize, usize),
    // Recurse
    Return(usize, usize, usize),
}

// fn &mut St

fn init_inst(state: &mut State, graph: &Graph, inst: Inst, stack: &mut Stack2) {
    if let Inst::Init(w, v) = inst {
        // println!("|{}|visiting|{}|{}|-|", state.count, w, v);
        state.visited.insert(w);
        state.next_sigma[w] = w;
        state.next_on_path[w] = w;
        state.pre[w] = state.count;
        state.lowpt[w] = state.count;
        state.num_descendants[w] = 1;
        state.count += 1;

        let mut neighbors: Vec<_> = graph[&w].iter().collect();
        neighbors.reverse();
        for edge in neighbors {
            // println!("pushin Loop{}, {}, {}", w, v, edge);
            stack.push_front(Inst::Loop(w, v, *edge));
        }
    } else {
        panic!("oh no!");
    }
}

fn loop_inst(state: &mut State, graph: &Graph, inst: Inst, stack: &mut Stack2) {
    if let Inst::Loop(w, v, u) = inst {
        // println!("|{}|looping|{}|{}|{}|", state.count, w, v, u);
        state.degrees[w] += 1;

        if !state.visited.contains(&u) {
            // println!("|{}|unvisited|{}|{}|{}|", state.count, w, v, u);
            // here we want to go deeper...
            // if printing {
            //     println!("|recursing with|{}| {}|", u, w);
            // }

            stack.push_front(Inst::Return(w, v, u));
            stack.push_front(Inst::Init(u, w));
        // stack2.push_front((u, w));

        // return;
        } else {
            // println!("|{}|previously visited|{}|{}|{}|", state.count, w, v, u);
            // (w, u) outgoing back-edge of w, i.e. dfs(w) > dfs(u)
            if u != v && state.is_back_edge(w, u) {
                if state.pre[u] < state.lowpt[w] {
                    state.absorb_path(w, state.next_on_path[w], None);

                    // P_w in paper
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

                    // P_w[w..u] in paper
                    state.absorb_path(w, state.next_on_path[w], Some(parent));

                    if state.is_null_path(parent) {
                        state.next_on_path[w] = w;
                    } else {
                        state.next_on_path[w] = state.next_on_path[parent];
                    }
                }
            }

            // println!("|{}|visited, absorbed|{}|{}|{}|", state.count, w, v, u);
        }
    } else {
        panic!("this is bad!");
    }
}

fn return_inst(state: &mut State, inst: Inst) {
    if let Inst::Return(w, v, u) = inst {
        // println!("|{}|past recursion|{}|{}|{}|", state.count, w, v, u);
        state.num_descendants[w] += state.num_descendants[u];

        if state.degrees[u] <= 2 {
            state.degrees[w] += state.degrees[u] - 2;

            state.add_component(u);

            if state.is_null_path(u) {
                // P_u = w, in the paper
                state.path_u = w;
            } else {
                // P_u = P_u - u, in the paper
                // the path w + P_u is now null?
                state.path_u = state.next_on_path[u];
            }
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
    // println!("|{}|unvisited, absorbed|{}|{}|{}|", state.count, w, v, u);
    } else {
        panic!("shouldn't happen!");
    }
}

// fn three_edge_step1(stacks: &mut graph: &Graph, state: &mut State, w: usize, v: usize) {

// }

fn three_edge_connect_new(graph: &Graph, state: &mut State) {
    // function, w, v
    // let mut stack: VecDeque<(usize, usize, usize)> = VecDeque::new();
    let mut stack: Stack2 = VecDeque::new();

    // let k = graph.keys().take(1).next().unwrap();

    // type Stack = VecDeque<(usize, usize)>;
    type Stack = VecDeque<(usize, usize, usize)>;
    type WV = (usize, usize);
    type UWV = (usize, usize, usize);
    // let mut v = 0;

    let nodes: Vec<_> = graph.keys().collect();

    let connect = |state: &mut State, stack: &mut Stack2| {
        // println!("{}\t-\tconnect", state.count);
        // println!("{:?}", stack);
        while let Some(inst) = stack.pop_front() {
            match inst {
                // init
                Inst::Init(w, v) => {
                    init_inst(state, graph, Inst::Init(w, v), stack);
                }
                // looping
                Inst::Loop(w, v, u) => {
                    loop_inst(state, graph, Inst::Loop(w, v, u), stack);
                }
                // returning
                Inst::Return(w, v, u) => {
                    return_inst(state, Inst::Return(w, v, u));
                }
            }
        }
    };

    for &n in nodes {
        if !state.visited.contains(&n) {
            stack.push_front(Inst::Init(n, 0));
            // step1_stack.push_front((n, 0));
            // println!("     stack - stack");
            // println!("         1 - 2");
            connect(state, &mut stack);
            // connect(state, &mut step1_stack, &mut step2_stack);
            /*
            while !step1_stack.is_empty() || !step2_stack.is_empty() {
                if !step1_stack.is_empty() {
                    let (w, v) = step1_stack.pop_front().unwrap();
                    step1_fun(state, (w, v), &mut step1_stack, &mut step2_stack);
                } else if !step2_stack.is_empty() {
                    let (u, v) = step2_stack.pop_front().unwrap();
                    step2_fun(state, (u, v));
                }
            }
            */
            state.add_component(n);
        }
    }
    // match (step1_stack.pop_front(), step2_stack.pop_front()) {
    //     (Some((w, v)), None) => {
    //         step1_fun(state, (w, v), &mut step1_stack);
    //     }
    //     (None, Some((u, v))) => {
    //         step2_fun(state, (u, v));
    //     }
    //     (Some((w1, v1)), Some((w2, v2))) => {
    //         // step1_fun
    //     }
    //     (_, _) => {}
    // }
    // }

    // state
}

fn three_edge_connect(graph: &Graph, state: &mut State, w: usize, v: usize) {
    let printing = true;

    // println!("|{}|visiting|{}|{}|-|", state.count, w, v);
    state.visited.insert(w);
    state.next_sigma[w] = w;
    state.next_on_path[w] = w;
    state.pre[w] = state.count;
    state.lowpt[w] = state.count;
    state.num_descendants[w] = 1;
    state.count += 1;

    // println!("{}", state.count);

    let edges = &graph[&w];

    for edge in edges {
        let u = *edge;
        // println!("|{}|looping|{}|{}|{}|", state.count, w, v, u);
        state.degrees[w] += 1;

        if !state.visited.contains(&u) {
            // println!("|{}|unvisited|{}|{}|{}|", state.count, w, v, u);
            // if printing {
            //     println!("|recursing from|{}|{}|", w, v);
            // }
            three_edge_connect(graph, state, u, w);
            // println!("|{}|past recursion|{}|{}|{}|", state.count, w, v, u);
            // if printing {
            //     println!("|past recursion|{}|{}|", w, v);
            // }
            state.num_descendants[w] += state.num_descendants[u];

            if state.degrees[u] <= 2 {
                state.degrees[w] += state.degrees[u] - 2;

                state.add_component(u);

                if state.is_null_path(u) {
                    // P_u = w, in the paper
                    state.path_u = w;
                } else {
                    // P_u = P_u - u, in the paper
                    // the path w + P_u is now null?
                    state.path_u = state.next_on_path[u];
                }
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
        // println!("|{}|unvisited, absorbed|{}|{}|{}|", state.count, w, v, u);
        } else {
            // println!("|{}|previously visited|{}|{}|{}|", state.count, w, v, u);
            // (w, u) outgoing back-edge of w, i.e. dfs(w) > dfs(u)
            if u != v && state.is_back_edge(w, u) {
                if state.pre[u] < state.lowpt[w] {
                    state.absorb_path(w, state.next_on_path[w], None);

                    // P_w in paper
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

                    // P_w[w..u] in paper
                    state.absorb_path(w, state.next_on_path[w], Some(parent));

                    if state.is_null_path(parent) {
                        state.next_on_path[w] = w;
                    } else {
                        state.next_on_path[w] = state.next_on_path[parent];
                    }
                }
            }

            // println!("|{}|visited, absorbed|{}|{}|{}|", state.count, w, v, u);
        }

        // if printing {
        //     println!("|end of if|{}|{}|", w, v);
        // }
    }
    // println!("|{}|returning|{}|{}|-|", state.count, w, v);
}

/// Prints each component, one per row, with space-delimited GFA
/// segment names, in the arbitrary HashMap order
fn print_components(
    inv_name_map: &HashMap<usize, String>,
    sigma: &BTreeMap<usize, BTreeSet<usize>>,
) {
    for (_k, component) in sigma {
        for (i, j) in component.iter().enumerate() {
            if i > 0 {
                print!("\t");
            }
            let name = &inv_name_map[&j];
            print!("{}", name);
        }
        println!();
    }
}

fn print_components_compat(
    inv_name_map: &HashMap<usize, String>,
    sigma: &BTreeMap<usize, BTreeSet<usize>>,
) {
    println!("# of components: {}", sigma.len());
    for (_k, v) in sigma {
        print!("component: ");
        for s in v {
            if *s != 0 {
                print!(" {}", inv_name_map[s]);
            }
        }
        println!();
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    let path = PathBuf::from(&args[1]);

    let gfa = parse_gfa(&path).unwrap();

    let algraph = ALGraph::from_gfa(&gfa);

    let mut state = State::initialize(&algraph.graph);

    let nodes: Vec<_> = algraph.graph.keys().collect();

    three_edge_connect_new(&algraph.graph, &mut state);
    // for &n in nodes {
    //     if !state.visited.contains(&n) {
    //         three_edge_connect(&algraph.graph, &mut state, n, 0);
    //         state.add_component(n);
    //     }
    // }

    // println!("# components: {}", state.sigma.len());

    print_components(&algraph.inv_name_map, &state.sigma);
    // print_components_compat(&algraph.inv_name_map, &state.sigma);
}
