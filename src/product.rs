use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]

pub struct ProductRow {
    pub id: String,
    pub name: String,
    pub craft_duration: f32,
    pub craft_type: CraftType,
    pub quantity: i32,
    dependencies: String
}

impl ProductRow {
    pub fn to_domain(self) -> Product {
        Product { 
            id: self.id, 
            name: self.name, 
            craft_duration: self.craft_duration, 
            craft_type: self.craft_type,  
            quantity: self.quantity,
            dependencies: ProductRow::build_dependencies(&self.dependencies) }
    }

    fn build_dependencies(dependencies: &str) -> HashMap<String, f32> {
        if dependencies.is_empty() {
            return HashMap::new();
        }
        dependencies.split(";")
            .map(|s| {
                let split: Vec<&str> = s.split(":").collect();
                (split[0].to_owned(), split[1].parse::<f32>().map_err(|e| format!("Could not parse {}", split[1])).unwrap())
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub craft_duration: f32,
    pub craft_type: CraftType,
    pub quantity: i32,
    pub dependencies: HashMap<String, f32>
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CraftType {
    Ore,
    Smelt,
    Assemble,
    Chemical,
}

impl CraftType {
    pub fn best_craft_speed(&self) -> f32 {
        match &self {
            &CraftType::Ore => 0.5,
            &CraftType::Smelt => 2.,
            &CraftType::Assemble => 0.75,
            &CraftType::Chemical => 1.0,
        }
    }
}