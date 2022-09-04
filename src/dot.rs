use std::{
    collections::HashMap,
    fs::{self},
    path::Path,
    process::Command,
};

use crate::graph;

pub fn write_graph<P: AsRef<Path>>(nodes: Vec<graph::Node>, edges: Vec<graph::Edge>, path: P) {
    write_graph_with_clusters(nodes, edges, HashMap::new(), path)
}

pub fn write_graph_with_clusters<P: AsRef<Path>>(
    nodes: Vec<graph::Node>,
    edges: Vec<graph::Edge>,
    clusters: HashMap<String, Vec<String>>,
    path: P,
) {
    println!(
        "Writing graph with {} nodes and {} edges to {}",
        nodes.len(),
        edges.len(),
        path.as_ref().to_str().unwrap_or("Unknown path")
    );
    let mut lines: Vec<String> = vec!["digraph G {".to_owned()];

    let mut nodes_by_id: HashMap<String, graph::Node> = HashMap::new();
    for node in nodes {
        nodes_by_id.insert(node.id.clone(), node);
    }

    for (cluster_name, cluster_products) in clusters.iter() {
        lines.push(format!("subgraph cluster_{} {{", cluster_name));
        lines.push("  color=\"white\"".to_owned());
        for product_id in cluster_products {
            if let Some(node) = nodes_by_id.remove(product_id) {
                lines.push(to_dot_node(&node))
            }
        }
        lines.push("}".to_owned());
    }

    for node in nodes_by_id.values() {
        lines.push(to_dot_node(node));
    }
    for edge in &edges {
        lines.push(to_dot_edge(edge));
    }
    lines.push("}".to_owned());

    fs::write(path.as_ref(), lines.join("\n")).expect("Unable to write file");
    Command::new("dot")
        .args(["-O", path.as_ref().to_str().unwrap(), "-Tsvg"])
        .output()
        .expect("failed to execute process");
}

fn to_dot_node(node: &graph::Node) -> String {
    format!(
        "{} [label=\"{}\\n{:.2}/s\\n{:.2} {}\",color=\"{}\",URL=\"./{}.dot.svg\"];",
        node.id,
        node.name,
        node.amount,
        node.source_amount,
        node.source_name,
        to_dot_color(&node.color),
        node.id
    )
}

fn to_dot_edge(edge: &graph::Edge) -> String {
    format!(
        "{} -> {} [label=\"{:.1} {}/s ({:.1})\",color=\"{}\"];",
        edge.from,
        edge.to,
        edge.amount,
        edge.from,
        edge.source_amount,
        to_dot_color(&edge.color),
    )
}

pub fn to_dot_color(color: &graph::Color) -> &'static str {
    match color {
        graph::Color::Green => "darkgreen",
        graph::Color::Yellow => "orange",
        graph::Color::Red => "red",
        graph::Color::Blue => "blue",
        graph::Color::Black => "black",
    }
}
