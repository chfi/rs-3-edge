use std::env;
use std::io::Write;
use std::path::PathBuf;

use structopt::StructOpt;

use three_edge_connected::algorithm;
use three_edge_connected::graph::Graph;
use three_edge_connected::state::State;

/// Prints each component, one per row, with space-delimited GFA
/// segment names, in the BTreeMap node order
fn write_components<T: Write>(
    stream: &mut T,
    inv_names: &[String],
    components: &[Vec<usize>],
) {
    for component in components {
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

fn main() {
    let args: Vec<_> = env::args().collect();
    let path = PathBuf::from(&args[1]);
    let algraph = ALGraph::from_gfa_file(&path);

    let mut state = State::initialize(&algraph.graph);

    algorithm::three_edge_connect(&algraph.graph, &mut state);

    let mut stdo = std::io::stdout();
    write_components(&mut stdo, &algraph.inv_names, state.components());
}
