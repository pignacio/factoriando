use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::Path,
    process::Command,
};

use colored::Colorize;
use product::CraftTechStatus;
use serde::de::DeserializeOwned;

use crate::{
    error::Error,
    product::{Product, ProductRow},
};

mod error;
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

    let products: Vec<Product> = rows.unwrap().into_iter().map(|p| p.to_domain()).collect();

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

    let craft_tech_status = CraftTechStatus::new(
        product::Miner::Electric,
        product::Furnace::Steel,
        product::Assembler::Blue,
    );

    let mut amount_per_second: HashMap<String, f32> = load_json("wanted.json")?;
    println!(
        "Wanted amounts before completing dependencies: {:?}",
        amount_per_second
    );

    for product in sorted_products.iter().rev() {
        let amount: f32 = amount_per_second.get(&product.id).unwrap_or(&0.0).clone();
        println!("Adding dependencies for {} {}w/s", amount, product.name);
        for (child, child_amount) in product.dependencies.iter() {
            let current_amount = amount_per_second.get(child).unwrap_or(&0.0);
            let new_amount = current_amount + (child_amount * amount / product.quantity as f32);
            amount_per_second.insert(child.to_owned(), new_amount);
        }
    }

    let mut dot_nodes: HashMap<String, String> = HashMap::new();
    let mut dot_edges: HashMap<(String, String), String> = HashMap::new();
    for product in &sorted_products {
        let amount = amount_per_second.get(&product.id).unwrap_or(&0.0).clone();

        if amount > 0. {
            let source_amount = amount * product.craft_duration
                / product.craft_type.best_craft_speed(&craft_tech_status)
                / product.quantity as f32;
            let source_string = format!(
                "{:.1} {}s",
                source_amount,
                craft_tech_status.tech_for(product).name()
            );

            let color = match product.craft_type {
                product::CraftType::Ore => "orange",
                product::CraftType::Smelt => "red",
                product::CraftType::Assemble => "blue",
                product::CraftType::Chemical => "darkgreen",
                _ => "black",
            };
            dot_nodes.insert(
                product.id.clone(),
                format!(
                    "  {} [label=\"{}\\n{:.2}/s\\n{}\",color=\"{}\",URL=\"./{}.dot.svg\"];",
                    product.id, product.name, amount, source_string, color, product.id
                ),
            );

            println!(
                " * {}: {:.2}/s ({})",
                product.id.green(),
                amount,
                source_string
            );

            for parent in &sorted_products {
                amount_per_second
                    .get(&parent.id)
                    .and_then(|parent_amount| {
                        parent
                            .dependencies
                            .get(&product.id)
                            .map(|q| parent_amount * q / parent.quantity as f32)
                    })
                    .filter(|amount| *amount > 0.0)
                    .map(|amount_for_parent| {
                        println!("   - {:.1}/s for {}", amount_for_parent, parent.id);
                        let color = if amount_for_parent > 0.5 * amount {
                            "red"
                        } else if amount_for_parent > 0.25 * amount {
                            "orange"
                        } else {
                            "darkgreen"
                        };
                        let source_amount = amount_for_parent * product.craft_duration
                            / product.craft_type.best_craft_speed(&craft_tech_status)
                            / product.quantity as f32;
                        dot_edges.insert(
                            (product.id.to_owned(), parent.id.to_owned()),
                            format!(
                                "  {} -> {} [label=\"{:.1} {}/s ({:.1})\",color=\"{}\"];",
                                product.id,
                                parent.id,
                                amount_for_parent,
                                product.id,
                                source_amount,
                                color
                            ),
                        );
                    });
            }
        }
    }

    let dot_clusters: HashMap<String, Vec<String>> = load_json("clusters.json")?;
    let graph_dir = Path::new("graphs");
    if !graph_dir.exists() {
        fs::create_dir(graph_dir).unwrap();
    }

    for (product_id, product_line) in &dot_nodes {
        let mut lines = Vec::new();
        lines.push("digraph G {".to_owned());
        lines.push(product_line.to_owned());

        for dependency in product_by_id[product_id].dependencies.keys() {
            if let Some(dep_line) = dot_nodes.get(dependency) {
                lines.push(dep_line.to_owned());
            }
            if let Some(edge_line) = dot_edges.get(&(dependency.to_owned(), product_id.to_owned()))
            {
                lines.push(edge_line.to_owned());
            }
        }
        for independency in independencies.get(product_id).unwrap_or(&Vec::new()) {
            if let Some(dep_line) = dot_nodes.get(independency) {
                lines.push(dep_line.to_owned());
            }
            if let Some(edge_line) =
                dot_edges.get(&(product_id.to_owned(), independency.to_owned()))
            {
                lines.push(edge_line.to_owned());
            }
        }

        lines.push("}".to_owned());

        let dot_path = graph_dir.join(format!("{}.dot", product_id));
        fs::write(&dot_path, lines.join("\n")).expect("Unable to write file");
        Command::new("dot")
            .args(["-O", &dot_path.to_str().unwrap(), "-Tsvg"])
            .output()
            .expect("failed to execute process");
    }

    let mut dot_lines: Vec<String> = Vec::new();
    dot_lines.push("digraph G {".to_owned());

    for (cluster_name, cluster_products) in dot_clusters.iter() {
        dot_lines.push(format!("subgraph cluster_{} {{", cluster_name));
        dot_lines.push("  color=\"white\"".to_owned());
        for product_id in cluster_products {
            dot_nodes
                .remove(product_id)
                .map(|line| dot_lines.push(line));
        }
        dot_lines.push("}".to_owned());
    }

    for line in dot_nodes.values() {
        dot_lines.push(line.clone());
    }
    dot_lines.extend(
        dot_edges
            .values()
            .map(|s| s.to_owned())
            .collect::<Vec<String>>(),
    );
    dot_lines.push("}".to_owned());

    let all_path = graph_dir.join("main.dot");
    fs::write(&all_path, dot_lines.join("\n")).expect("Unable to write file");

    Command::new("dot")
        .args(["-O", all_path.to_str().unwrap(), "-Tsvg"])
        .output()
        .expect("failed to execute process");

    Ok(())
}

fn load_json<T: DeserializeOwned, P: AsRef<Path> + ?Sized>(path: &P) -> Result<T, Error> {
    let file = File::open(path)
        .map_err(|e| Error::from(e, path.as_ref().to_str().unwrap_or("(unknown path)")))?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}
