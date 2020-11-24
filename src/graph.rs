use std::io::prelude::*;

use fxhash::FxHashMap;

use bstr::io::*;

use gfa::parser::{GFAParser, GFAParserBuilder};

pub type AdjacencyList = Vec<usize>;
pub type FxMapGraph = FxHashMap<usize, AdjacencyList>;

/// An adjacency list representation of a generic graph, including the
/// map required to go from node index to the original node name. The
/// `N` type parameter is the node name in the original graph, e.g.
/// `BString` for GFA graphs, or `usize` for graphs that use integer
/// names.
pub struct Graph<N> {
    pub graph: FxMapGraph,
    pub inv_names: Vec<N>,
}

impl Graph<usize> {
    /// Construct an adjacency graph from an iterator over the edges
    /// of an existing graph. Both the input and output have `usize`
    /// node IDs, but `from_edges` performs a transformation to ensure
    /// all the node IDs are consecutive starting from 0.
    pub fn from_edges<I>(input: I) -> Graph<usize>
    where
        I: Iterator<Item = (usize, usize)>,
    {
        let mut graph: FxHashMap<usize, AdjacencyList> = FxHashMap::default();
        let mut name_map: FxHashMap<usize, usize> = FxHashMap::default();
        let mut inv_names = Vec::new();

        let mut get_ix = |name: usize| {
            if let Some(ix) = name_map.get(&name) {
                *ix
            } else {
                let ix = name_map.len();
                name_map.insert(name, ix);
                inv_names.push(name);
                ix
            }
        };

        for (from, to) in input {
            let from_ix = get_ix(from);
            let to_ix = get_ix(to);

            graph.entry(from_ix).or_default().push(to_ix);
            graph.entry(to_ix).or_default().push(from_ix);
        }

        Graph { graph, inv_names }
    }
}

impl Graph<Vec<u8>> {
    /// Constructs an adjacency list representation of the given GFA
    /// file input stream, parsing the GFA line-by-line and only
    /// keeping the links. Returns the graph as an adjacency list and
    /// a map from graph indices to GFA segment names.
    pub fn from_gfa_reader<T: BufRead>(reader: &mut T) -> Graph<Vec<u8>> {
        let lines = &mut reader.byte_lines();

        let parser: GFAParser<Vec<u8>, ()> = GFAParserBuilder {
            links: true,
            ..GFAParserBuilder::none()
        }
        .build();

        let gfa_lines =
            lines.filter_map(move |l| parser.parse_gfa_line(&l.unwrap()).ok());

        let mut graph: FxHashMap<usize, AdjacencyList> = FxHashMap::default();
        let mut name_map: FxHashMap<Vec<u8>, usize> = FxHashMap::default();
        let mut inv_names = Vec::new();

        let mut get_ix = |name: &[u8]| {
            if let Some(ix) = name_map.get(name) {
                *ix
            } else {
                let ix = name_map.len();
                name_map.insert(name.into(), ix);
                inv_names.push(name.into());
                ix
            }
        };

        for line in gfa_lines {
            if let gfa::gfa::Line::Link(link) = line {
                let from_ix = get_ix(link.from_segment.as_ref());
                let to_ix = get_ix(link.to_segment.as_ref());

                graph.entry(from_ix).or_default().push(to_ix);
                graph.entry(to_ix).or_default().push(from_ix);
            }
        }

        Graph { graph, inv_names }
    }
}

impl<N: Clone> Graph<N> {
    /// Given a vector of graph components (as produced by
    pub fn invert_components(
        &self,
        components: Vec<Vec<usize>>,
    ) -> Vec<Vec<N>> {
        components.into_iter().filter_map(|c|{
            let len = c.len();
            if len > 1 {
                let names: Vec<_> = c.iter()
                    .filter_map(|j| self.inv_names.get(*j))
                    .cloned()
                    .collect();

                assert_eq!(len,
                           names.len(),
                           "Could not find inverse name when inverting graph components");
                Some(names)
            } else {
                None
            }
        }).collect()
    }
}
