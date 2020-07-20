use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

use gfa::gfa::GFAParsingConfig;
use gfa::parser::parse_gfa_stream_config;

pub type AdjacencyList = Vec<usize>;
pub type Graph = BTreeMap<usize, AdjacencyList>;

/// An adjacency list representation of a GFA graph, including the
/// maps required to go from GFA segment names to graph node indices,
/// and back
pub struct ALGraph {
    pub graph: Graph,
    pub inv_names: Vec<String>,
}

impl ALGraph {
    /// Constructs an adjacency list representation of the given GFA
    /// parser stream. Returns both the adjacency list and a map from GFA
    /// segment names to corresponding index in the graph.
    pub fn from_gfa_file(path: &PathBuf) -> ALGraph {
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
