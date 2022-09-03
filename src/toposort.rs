use std::collections::{HashMap, HashSet};

use crate::{error::Error, product::Product};

pub fn topological_sort(
    product_by_id: &HashMap<String, Product>,
    products: &[Product],
) -> Result<Vec<Product>, Error> {
    let mut edges: HashMap<String, HashSet<String>> = HashMap::new();
    let mut anti_edges: HashMap<String, HashSet<String>> = HashMap::new();
    for product in product_by_id.values() {
        for child in product.dependencies.keys() {
            edges
                .entry(product.id.to_owned())
                .or_insert_with(HashSet::new)
                .insert(child.to_owned());
            anti_edges
                .entry(child.to_owned())
                .or_insert_with(HashSet::new)
                .insert(product.id.to_owned());
        }
    }

    let mut inorder = Vec::new();

    let mut remaining: HashSet<String> = edges
        .keys()
        .map(|s| s.to_owned())
        .filter(|p| !anti_edges.contains_key(p))
        .collect();

    while !remaining.is_empty() {
        let id = &products
            .iter()
            .rev()
            .find(|p| remaining.contains(&p.id))
            .ok_or_else(|| Error::Simple(format!("Invalid remaining items: {:?}", remaining)))?
            .id;
        println!("topo_sort: Adding {}", id);
        remaining.remove(id);
        inorder.push(product_by_id[id].clone());

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

    inorder.reverse();
    Ok(inorder)
}
