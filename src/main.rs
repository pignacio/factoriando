use std::{error::Error, collections::{HashMap, HashSet}};

use crate::product::{Product, ProductRow};

pub mod product;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut reader = csv::Reader::from_path("data/product.csv")?;
    let rows: Result<Vec<ProductRow>, _> = reader.deserialize()
        //.map(|x: Result<ProductRow, csv::Error>| x.and_then(|p| p.to_domain() ))
        .collect();
    
    let products: Vec<Product> = rows.unwrap().into_iter().map(|p| p.to_domain()).collect();

    let mut product_by_id : HashMap<String, Product> = HashMap::new();
    
    for product in products {
        product_by_id.insert(product.id.clone(), product.clone());
    }

    let sorted_products = topological_sort(&product_by_id);

    let mut amount_per_second: HashMap<String, f32> = HashMap::new();

    amount_per_second.insert("red_potion".to_owned(), 0.5);
    amount_per_second.insert("green_potion".to_owned(), 0.5);
    amount_per_second.insert("black_potion".to_owned(), 0.5);
    amount_per_second.insert("cyan_potion".to_owned(), 0.5);
    

    for product in &sorted_products {
        let amount = amount_per_second.get(&product.id).unwrap_or(&0.0).clone();
        for (child, child_amount) in product.dependencies.iter() {
            let current_amount = amount_per_second.get(child).unwrap_or(&0.0);
            let new_amount = current_amount + (child_amount * amount / product.quantity as f32);
            amount_per_second.insert(child.to_owned(), new_amount);
        }
    }

    for product in &sorted_products {
        let amount = amount_per_second.get(&product.id).unwrap_or(&0.0).clone();
        if amount > 0. {
            println!(" * {}: {}/s ({} sources)", product.id, amount, amount * product.craft_duration / product.craft_type.best_craft_speed() / product.quantity as f32);
        }
    }

    Ok(())
}


fn topological_sort(products: &HashMap<String, Product>) -> Vec<Product> {
    let mut edges: HashMap<String, HashSet<String>> = HashMap::new();
    let mut anti_edges: HashMap<String, HashSet<String>> = HashMap::new();
    for product in products.values() {
        for child in product.dependencies.keys() {
            edges.entry(product.id.to_owned()).or_insert(HashSet::new()).insert(child.to_owned());
            anti_edges.entry(child.to_owned()).or_insert(HashSet::new()).insert(product.id.to_owned());
        }
    }

    let mut inorder = Vec::new();

    let mut remaining: HashSet<String> = edges.keys()
        .map(|s| s.to_owned())
        .filter(|p| !anti_edges.contains_key(p) )
        .collect();

    while !remaining.is_empty() {
        let id = remaining.iter().next().unwrap().to_owned();
        println!("topo_sort: Adding {}", id);
        remaining.remove(&id);
        inorder.push(products[&id].clone());

        let childs = edges.entry(id.clone()).or_default().clone();
        edges.entry(id.clone()).or_default().clear();
        
        for child in childs {
            if !edges.values().any(|e| e.contains(&child)) {
                remaining.insert(child.clone());
            }
        }
    }

    if edges.values().any(|e| !e.is_empty()) {
        panic!("Graph has cycles")
    }

    inorder
}