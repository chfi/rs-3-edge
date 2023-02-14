use three_edge_connected::{algorithm, Graph};

/// Tests the correctness of the algorithm by running it against
/// graphs for which the 3EC components are known

fn k_graph_edges(offset: usize, n: usize) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();

    let start = offset;
    let end = offset + n;

    for i in start..end {
        for j in i..end {
            if i != j {
                edges.push((i, j));
            }
        }
    }

    edges
}

fn bridged_k_graphs(k_a: usize, k_b: usize, bridges: usize) -> Graph<usize> {
    let a_edges = k_graph_edges(0, k_a);
    let last_a = a_edges.last().unwrap().1;

    let first_b = last_a + 1;

    let mut b_edges = k_graph_edges(first_b, k_b);

    let mut edges = a_edges;
    edges.append(&mut b_edges);

    for _ in 0..bridges {
        edges.push((last_a, first_b));
    }

    Graph::from_edges(edges.into_iter())
}

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
    let graph = bridged_k_graphs(4, 4, 1);

    println!("{:#?}", &graph.graph);

    let comps = algorithm::find_components(&graph.graph);

    for (ix, comp) in comps.iter().enumerate() {
        println!("{ix}\t{comp:?}");
    }

    // two K4 graphs connected by a single bridge consists
    // of two 3EC components
    assert_eq!(comps.len(), 2);
    assert!(comps.iter().all(|comp| comp.len() == 4));
}

#[test]
fn two_k_4_parallel() {
    // two bridges between two K4 graphs is still two 3ECCs
    let graph = bridged_k_graphs(4, 4, 2);

    println!("{:#?}", &graph.graph);
    let comps = algorithm::find_components(&graph.graph);
    for (ix, comp) in comps.iter().enumerate() {
        println!("{ix}\t{comp:?}");
    }

    assert_eq!(comps.len(), 2);
    assert!(comps.iter().all(|comp| comp.len() == 4));

    // however, three parallel edges makes it a single 3ECC
    let graph = bridged_k_graphs(4, 4, 3);

    println!("{:#?}", &graph.graph);
    let comps = algorithm::find_components(&graph.graph);
    for (ix, comp) in comps.iter().enumerate() {
        println!("{ix}\t{comp:?}");
    }

    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].len(), 8);
}
