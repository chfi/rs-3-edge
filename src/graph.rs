use bstr::io::*;
use bstr::{BStr, BString};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io::prelude::*;

use gfa::parser::{GFAParser, GFAParsingConfig};

pub type AdjacencyList = Vec<usize>;
pub type BTreeGraph = BTreeMap<usize, AdjacencyList>;

/// An adjacency list representation of a GFA graph, including the
/// map required to go from node index to GFA segment name
pub struct Graph {
    pub graph: BTreeGraph,
    pub inv_names: Vec<BString>,
}

impl Graph {
    /// Constructs an adjacency list representation of the given GFA
    /// input stream, parsing the GFA line-by-line and only keeping
    /// the links. Returns the graph as an adjacency list and a map
    /// from graph indices to GFA segment names.
    pub fn from_gfa_reader<T: BufRead>(reader: &mut T) -> Graph {
        let lines = &mut reader.byte_lines();

        let conf = GFAParsingConfig {
            links: true,
            ..GFAParsingConfig::none()
        };
        let parser: GFAParser<()> = GFAParser::with_config(conf);
        let gfa_lines =
            lines.filter_map(move |l| parser.parse_line(&l.unwrap()));

        let mut graph: BTreeMap<usize, AdjacencyList> = BTreeMap::new();
        let mut name_map: HashMap<BString, usize> = HashMap::new();
        let mut inv_names = Vec::new();

        let mut get_ix = |name: &BStr| {
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
