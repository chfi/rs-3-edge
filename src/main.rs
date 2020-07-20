use std::env;
use std::path::PathBuf;

use three_edge_connected::algorithm;
use three_edge_connected::graph::ALGraph;
use three_edge_connected::state::State;

/// Prints each component, one per row, with space-delimited GFA
/// segment names, in the BTreeMap node order
fn print_components(inv_name_arr: &[String], sigma: &[(usize, Vec<usize>)]) {
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

    algorithm::three_edge_connect(&algraph.graph, &mut state);

    print_components(&algraph.inv_names, state.components());
}
