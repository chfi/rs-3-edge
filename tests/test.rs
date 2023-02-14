use three_edge_connected::{algorithm, Graph};

/// Tests the correctness of the algorithm by running it against
/// graphs for which the 3EC components are known

fn complete_graph(n: usize) -> Graph<usize> {
    let mut edges = Vec::new();

    for i in 0..n {
        for j in i..n {
            if i != j {
                edges.push((i, j));
            }
        }
    }

    Graph::from_edges(edges.into_iter())
}

fn bipartite_graph(k: usize, l: usize) -> Graph<usize> {
    let mut edges = Vec::new();

    for a in 0..k {
        for b in 0..l {
            let pa = 2 * a;
            let pb = 2 * b + 1;

            edges.push((pa, pb));
        }
    }

    Graph::from_edges(edges.into_iter())
}

/// The complete graph with 3 vertices is not 3EC-connected
#[test]
fn k_3() {
    let graph = complete_graph(3);

    let comps = algorithm::find_components(&graph.graph);
    // println!("{comps:#?}");
    for (ix, comp) in comps.iter().enumerate() {
        println!("{ix}\t{comp:?}");
    }

    // since the graph is not 3ECC, the three vertices should
    // each be in their own singleton component
    assert_eq!(comps.len(), 3);
}

/// The complete graph with 4 vertices has one 3ECC
#[test]
fn k_4() {
    let graph = complete_graph(4);

    let comps = algorithm::find_components(&graph.graph);

    for (ix, comp) in comps.iter().enumerate() {
        println!("{ix}\t{comp:?}");
    }

    // four vertices in one 3ECC
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].len(), 4);
}

/// The complete bipartite graph K_3_3 also has one 3ECC
#[test]
fn k_3_3() {
    let graph = bipartite_graph(3, 3);

    println!("{:#?}", &graph.graph);

    let comps = algorithm::find_components(&graph.graph);

    for (ix, comp) in comps.iter().enumerate() {
        println!("{ix}\t{comp:?}");
    }
}

#[test]
fn two_k_4() {
    let graph = complete_graph(4);
}
