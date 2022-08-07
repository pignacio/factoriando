use std::{collections::HashMap, error::Error, fs, process::Command, io::Write};

use colored::Colorize;
use product::CraftTechStatus;

use crate::product::{Product, ProductRow};

pub mod product;
pub mod toposort;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), Box<dyn Error>> {
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

    let sorted_products = toposort::topological_sort(&product_by_id, &products);

    let craft_tech_status = CraftTechStatus::new(
        product::Miner::Electric,
        product::Furnace::Steel,
        product::Assembler::Blue,
    );

    let mut amount_per_second: HashMap<String, f32> = HashMap::new();
    amount_per_second.insert("red_potion".to_owned(), 0.5);
    amount_per_second.insert("green_potion".to_owned(), 0.5);
    amount_per_second.insert("black_potion".to_owned(), 0.5);
    amount_per_second.insert("cyan_potion".to_owned(), 0.5);
    amount_per_second.insert("purple_potion".to_owned(), 0.5);

    for product in sorted_products.iter().rev() {
        let amount = amount_per_second.get(&product.id).unwrap_or(&0.0).clone();
        for (child, child_amount) in product.dependencies.iter() {
            let current_amount = amount_per_second.get(child).unwrap_or(&0.0);
            let new_amount = current_amount + (child_amount * amount / product.quantity as f32);
            amount_per_second.insert(child.to_owned(), new_amount);
        }
    }

    let mut dot_lines: Vec<String> = Vec::new();
    dot_lines.push("digraph G {".to_owned());
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
                
            };
            dot_lines.push(format!(
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
                    .map(|amount_for_parent| {
                        println!("   - {:.1}/s for {}", amount_for_parent, parent.id);
                        let color = if amount_for_parent > 0.5 * amount {
                            "red"
                        } else if amount_for_parent > 0.25 * amount {
                            "orange"
                        } else {
                            "darkgreen"
                        };
                        dot_edges.push(format!("  {} -> {} [label=\"{:.1} {}/s\",color=\"{}\"];", product.id, parent.id, amount_for_parent, product.id, color));
                    });
            }
        }
    }
    dot_lines.extend(dot_edges);
    dot_lines.push("}".to_owned());


    fs::write("graph.dot", dot_lines.join("\n")).expect("Unable to write file");

    Command::new("dot")
        .args(["-O", "graph.dot", "-Tsvg"])
        .output()
        .expect("failed to execute process");

    Ok(())
}
