use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use colored::Colorize;
use product::CraftTechStatus;
use serde::de::DeserializeOwned;

use crate::{
    error::Error,
    module::Module,
    product::{CraftTech, Product, ProductRow},
};

mod dot;
mod error;
mod graph;
mod module;
mod product;
mod toposort;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), Error> {
    let mut reader = csv::Reader::from_path("data/product.csv")?;
    let rows: Result<Vec<ProductRow>, _> = reader
        .deserialize()
        //.map(|x: Result<ProductRow, csv::Error>| x.and_then(|p| p.to_domain() ))
        .collect();

    let products: Vec<Product> = rows.unwrap().into_iter().map(|p| p.into_domain()).collect();

    let mut product_by_id: HashMap<String, Product> = HashMap::new();

    for product in &products {
        product_by_id.insert(product.id.clone(), product.clone());
    }

    let sorted_products = toposort::topological_sort(&product_by_id, &products)?;

    let mut independencies: HashMap<String, Vec<String>> = HashMap::new();

    for product in &sorted_products {
        for dependency in product.dependencies.keys() {
            independencies
                .entry(dependency.to_owned())
                .or_insert(Vec::new())
                .push(product.id.to_owned())
        }
    }

    let mut productive_assembler = product::Assembler::green();
    productive_assembler.add_module(Module::Productivity3);
    productive_assembler.add_module(Module::Productivity3);
    productive_assembler.add_module(Module::Productivity3);
    productive_assembler.add_module(Module::Productivity3);
    let mut craft_tech_status = CraftTechStatus::new(
        product::Miner::Electric,
        product::Furnace::Steel,
        product::Assembler::blue(),
    );

    craft_tech_status.add_override("low_density_structure", Box::new(productive_assembler));

    let mut amount_per_second: HashMap<String, f32> = load_json("wanted.json")?;
    println!(
        "Wanted amounts before completing dependencies: {:?}",
        amount_per_second
    );

    for product in sorted_products.iter().rev() {
        let amount: f32 = *amount_per_second.get(&product.id).unwrap_or(&0.0);
        println!("Adding dependencies for {} {}w/s", amount, product.name);
        for (child, child_amount) in product.dependencies.iter() {
            let child_tech = craft_tech_status.tech_for(product_by_id.get(child).unwrap());
            let current_amount = amount_per_second.get(child).unwrap_or(&0.0);
            let new_amount = current_amount
                + (child_amount * amount / child_tech.productivity() / product.quantity as f32);
            amount_per_second.insert(child.to_owned(), new_amount);
        }
    }

    let mut nodes: HashMap<String, graph::Node> = HashMap::new();
    let mut edges: HashMap<(String, String), graph::Edge> = HashMap::new();
    for product in &sorted_products {
        let amount = *amount_per_second.get(&product.id).unwrap_or(&0.0);

        let craft_tech = craft_tech_status.tech_for(product);

        if amount > 0. {
            let source_amount =
                amount * product.craft_duration / craft_tech.speed() / product.quantity as f32;
            let source_string = format!("{:.1} {}s", source_amount, craft_tech.name());

            let color = match product.craft_type {
                product::CraftType::Ore => graph::Color::Yellow,
                product::CraftType::Smelt => graph::Color::Red,
                product::CraftType::Assemble => graph::Color::Blue,
                product::CraftType::Chemical => graph::Color::Green,
                _ => graph::Color::Black,
            };
            nodes.insert(
                product.id.clone(),
                graph::Node::new(
                    &product.id,
                    &product.name,
                    amount,
                    source_amount,
                    craft_tech.name(),
                    color,
                ),
            );

            println!(
                " * {}: {:.2}/s ({})",
                product.id.green(),
                amount,
                source_string
            );

            for parent in &sorted_products {
                if let Some(amount_for_parent) = amount_per_second
                    .get(&parent.id)
                    .and_then(|parent_amount| {
                        parent
                            .dependencies
                            .get(&product.id)
                            .map(|q| parent_amount * q / parent.quantity as f32)
                    })
                    .filter(|amount| *amount > 0.0)
                {
                    println!("   - {:.1}/s for {}", amount_for_parent, parent.id);
                    let color = if amount_for_parent > 0.5 * amount {
                        graph::Color::Red
                    } else if amount_for_parent > 0.25 * amount {
                        graph::Color::Yellow
                    } else {
                        graph::Color::Green
                    };
                    let source_amount = amount_for_parent * product.craft_duration
                        / product.craft_type.best_craft_speed(&craft_tech_status)
                        / product.quantity as f32;

                    let key = (product.id.to_owned(), parent.id.to_owned());
                    edges.insert(
                        key,
                        graph::Edge::new(
                            &product.id,
                            &parent.id,
                            amount_for_parent,
                            source_amount,
                            color,
                        ),
                    );
                }
            }
        }
    }

    let dot_clusters: HashMap<String, Vec<String>> = load_json("clusters.json")?;
    let graph_dir = Path::new("graphs");
    if !graph_dir.exists() {
        fs::create_dir(graph_dir).unwrap();
    }

    for (product_id, product_node) in &nodes {
        let mut these_nodes = Vec::new();
        let mut these_edges = Vec::new();

        these_nodes.push(product_node.clone());

        for dependency in product_by_id[product_id].dependencies.keys() {
            if let Some(dep_node) = nodes.get(dependency) {
                these_nodes.push(dep_node.clone());
            }
            if let Some(dep_edge) = edges.get(&(dependency.to_owned(), product_id.to_owned())) {
                these_edges.push(dep_edge.clone());
            }
        }

        for independency in independencies.get(product_id).unwrap_or(&Vec::new()) {
            if let Some(indep_node) = nodes.get(independency) {
                these_nodes.push(indep_node.clone());
            }
            if let Some(indep_edge) = edges.get(&(product_id.to_owned(), independency.to_owned())) {
                these_edges.push(indep_edge.clone());
            }
        }

        let dot_path = graph_dir.join(format!("{}.dot", product_id));

        dot::write_graph(these_nodes, these_edges, dot_path);
    }

    let all_path = graph_dir.join("main.dot");

    dot::write_graph_with_clusters(
        nodes.values().cloned().collect(),
        edges.values().cloned().collect(),
        dot_clusters,
        all_path,
    );

    Ok(())
}

fn load_json<T: DeserializeOwned, P: AsRef<Path> + ?Sized>(path: &P) -> Result<T, Error> {
    let file = File::open(path)
        .map_err(|e| Error::from(e, path.as_ref().to_str().unwrap_or("(unknown path)")))?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}
