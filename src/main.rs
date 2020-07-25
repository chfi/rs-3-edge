use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

use bstr::BString;
use structopt::StructOpt;

use three_edge_connected::algorithm;
use three_edge_connected::graph::Graph;
use three_edge_connected::state::State;

/// Finds the 3-edge-connected components in a graph. Input must be a
/// bridgeless graph in the GFA format. Output is a list of
/// 3-edge-connected components, one per line, as space-delimited
/// lists of segment names.
#[derive(StructOpt, Debug)]
struct Opt {
    /// If true, read input GFA on stdin.
    #[structopt(short, required_unless("in-file"))]
    stdin: bool,

    /// GFA file to use, must be present if not reading from stdin.
    #[structopt(short, long, parse(from_os_str), required_unless("stdin"))]
    in_file: Option<PathBuf>,

    /// Output file. If empty, writes on stdout.
    #[structopt(short, long, parse(from_os_str))]
    out_file: Option<PathBuf>,
}

/// Prints each component, one per row, with space-delimited GFA
/// segment names, in the node index order
fn write_components<T: Write>(
    stream: &mut T,
    inv_names: &[BString],
    components: &[Vec<usize>],
) {
    for component in components {
        if component.len() > 1 {
            component.iter().enumerate().for_each(|(i, j)| {
                if i > 0 {
                    write!(stream, "\t{}", inv_names[*j]).unwrap();
                } else {
                    write!(stream, "{}", inv_names[*j]).unwrap();
                }
            });
            writeln!(stream).unwrap();
        }
    }
}

fn main() {
    let opt = Opt::from_args();

    let mut in_handle: Box<dyn BufRead> = {
        match opt.in_file {
            None => Box::new(BufReader::new(std::io::stdin())),
            Some(path) => {
                let file = File::open(&path).unwrap_or_else(|_| {
                    panic!("Could not open file {:?}", path)
                });
                Box::new(BufReader::new(file))
            }
        }
    };

    let graph = Graph::from_gfa_reader(&mut in_handle);

    let mut state = State::initialize(&graph.graph);

    algorithm::three_edge_connect(&graph.graph, &mut state);

    let mut out_handle: Box<dyn Write> = {
        match opt.out_file {
            None => Box::new(BufWriter::new(std::io::stdout())),
            Some(path) => {
                let fout = File::create(&path).unwrap_or_else(|_| {
                    panic!("Could not create file {:?}", path)
                });
                Box::new(BufWriter::new(fout))
            }
        }
    };

    write_components(&mut out_handle, &graph.inv_names, state.components());
}
