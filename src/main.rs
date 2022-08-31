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

    let craft_tech_status = CraftTechStatus::new(
        product::Miner::Electric,
        product::Furnace::Steel,
        product::Assembler::Blue,
    );

    let mut amount_per_second: HashMap<String, f32> = load_json("wanted.json")?;
    println!("Wanted amounts before completing dependencies: {:?}", amount_per_second);

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
    let mut dot_edges: Vec<String> = Vec::new();
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
            dot_nodes.insert(product.id.clone(), format!(
                "  {} [label=\"{}\\n{:.2}/s\\n{}\",color=\"{}\"];",
                product.id, product.name, amount, source_string, color
            ));

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
                        let source_amount = amount_for_parent * product.craft_duration / product.craft_type.best_craft_speed(&craft_tech_status) / product.quantity as f32;
                        dot_edges.push(format!(
                            "  {} -> {} [label=\"{:.1} {}/s ({:.1})\",color=\"{}\"];",
                            product.id, parent.id, amount_for_parent, product.id, source_amount, color
                        ));
                    });
            }
        }
    }

    let dot_clusters: HashMap<String, Vec<String>> = load_json("clusters.json")?;

    let mut dot_lines: Vec<String> = Vec::new();
    dot_lines.push("digraph G {".to_owned());
    
    for (cluster_name, cluster_products) in dot_clusters.iter() {
        dot_lines.push(format!("subgraph cluster_{} {{", cluster_name));
        dot_lines.push("  color=\"white\"".to_owned());
        for product_id in cluster_products {
            dot_nodes.remove(product_id).map(|line| dot_lines.push(line));
        }
        dot_lines.push("}".to_owned());
    }

    for line in dot_nodes.values() {
        dot_lines.push(line.clone());
    };
    dot_lines.extend(dot_edges);
    dot_lines.push("}".to_owned());

    fs::write("graph.dot", dot_lines.join("\n")).expect("Unable to write file");

    Command::new("dot")
        .args(["-O", "graph.dot", "-Tsvg"])
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
