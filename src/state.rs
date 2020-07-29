use crate::graph::BTreeGraph;

#[derive(Default, Debug, Clone)]
pub struct State {
    pub degrees: Vec<isize>,
    pub next_sigma: Vec<usize>,
    pub next_on_path: Vec<usize>,
    pub visited: Vec<bool>,
    pub pre: Vec<usize>,
    pub lowpt: Vec<usize>,
    pub count: usize,
    pub num_descendants: Vec<usize>,
    pub path_u: usize,
    pub sigma: Vec<Vec<usize>>,
}

impl State {
    pub fn initialize(graph: &BTreeGraph) -> State {
        let num_nodes = graph.len();

        State {
            count: 1,
            next_sigma: vec![0; num_nodes],
            next_on_path: vec![0; num_nodes],
            pre: vec![0; num_nodes],
            lowpt: vec![0; num_nodes],
            num_descendants: vec![1; num_nodes],
            degrees: vec![0; num_nodes],
            visited: vec![false; num_nodes],
            sigma: Vec::new(),
            path_u: 0,
        }
    }

    pub fn mut_recur(&mut self, w: usize) {
        assert!(w < self.visited.len());
        unsafe {
            *self.visited.get_unchecked_mut(w) = true;
            *self.next_sigma.get_unchecked_mut(w) = w;
            *self.next_on_path.get_unchecked_mut(w) = w;
            *self.pre.get_unchecked_mut(w) = self.count;
            *self.lowpt.get_unchecked_mut(w) = self.count;
        }
        self.count += 1;
    }

    pub fn components(&self) -> &Vec<Vec<usize>> {
        &self.sigma
    }

    pub fn is_back_edge(&self, u: usize, v: usize) -> bool {
        self.pre[u] > self.pre[v]
    }

    pub fn is_null_path(&self, u: usize) -> bool {
        self.next_on_path[u] == u
    }

    pub fn absorb_path(
        &mut self,
        root: usize,
        path: usize,
        end: Option<usize>,
    ) {
        if Some(root) != end {
            let mut current = root;
            let mut step = path;
            while current != step {
                unsafe {
                    *self.degrees.get_unchecked_mut(root) +=
                        *self.degrees.get_unchecked_mut(step) - 2;
                    self.next_sigma.swap(root, step);
                    current = step;
                    if Some(step) != end {
                        step = *self.next_on_path.get_unchecked(step);
                    }
                    // self.degrees[root] += self.degrees[step] - 2;
                    // self.next_sigma.swap(root, step);
                    // current = step;
                    // if Some(step) != end {
                    //     step = self.next_on_path[step];
                    // }
                }
            }
        }
    }

    pub fn sigma_iter(&self, start: usize) -> SigmaIter<'_> {
        SigmaIter::new(self, start)
    }

    pub fn add_component(&mut self, start: usize) {
        self.sigma.push(self.sigma_iter(start).collect());
    }
}

// Struct representing an iterator over a node's sigma set
pub struct SigmaIter<'a> {
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
            next_sigma,
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
