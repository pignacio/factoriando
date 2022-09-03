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
        dependencies.split(';')
            .map(|s| {
                let split: Vec<&str> = s.split(':').collect();
                (split[0].to_owned(), split[1].parse::<f32>().map_err(|_e| format!("Could not parse {}", split[1])).unwrap())
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
    pub dependencies: HashMap<String, f32>,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CraftType {
    Ore,
    Smelt,
    Assemble,
    Chemical,
    Silo,
    Launch,
}

impl CraftType {
    pub fn best_craft_speed(&self, tech_status: &CraftTechStatus) -> f32 {
        match &self {
            &CraftType::Ore => tech_status.miner_speed(),
            &CraftType::Smelt => tech_status.furnace_speed(),
            &CraftType::Assemble => tech_status.assembler_speed(),
            &CraftType::Chemical => tech_status.chemical_speed(),
            &CraftType::Silo => tech_status.silo_speed(),
            &CraftType::Launch => 1.0,
        }
    }
}

pub struct CraftTechStatus {
    miner: Miner,
    furnace: Furnace,
    assembler: Assembler,
    chemical: SimpleCraftTech,
    silo: SimpleCraftTech
}

impl CraftTechStatus {
    pub fn new(miner: Miner, furnace: Furnace, assembler: Assembler) -> Self {
        CraftTechStatus { 
            miner, 
            furnace, 
            assembler, 
            chemical: SimpleCraftTech::new("Chemical plant", 1.),
            silo: SimpleCraftTech::new("Rocket Silo", 1.),
        }
    }

    pub fn miner_speed(&self) -> f32 {
        self.miner.speed()
    }

    pub fn furnace_speed(&self) -> f32 {
        self.furnace.speed()
    }

    pub fn assembler_speed(&self) -> f32 {
        self.assembler.speed()
    }

    pub fn chemical_speed(&self) -> f32 {
        self.chemical.speed()
    }

    pub fn silo_speed(&self) -> f32 {
        self.silo.speed()
    }

    pub fn tech_for(&self, product: &Product) -> &dyn CraftTech {
        match product.craft_type {
            CraftType::Ore => &self.miner,
            CraftType::Smelt => &self.furnace,
            CraftType::Assemble => &self.assembler,
            CraftType::Chemical => &self.chemical,
            CraftType::Silo => &self.silo,
            CraftType::Launch => &self.silo,
        }
    }
}

pub trait CraftTech {
    fn name(&self) -> String;
    fn speed(&self) -> f32;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Miner {
    Burner,
    Electric,
}

impl CraftTech for Miner {
    fn name(&self) -> String {
        match self {
            Miner::Burner => "Burner Miner",
            Miner::Electric => "Electric Miner",
        }.to_owned()
    }

    fn speed(&self) -> f32 {
        match self {
            Miner::Burner => 0.25,
            Miner::Electric => 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Furnace {
    Stone,
    Steel,
    Electric,
}

impl CraftTech for Furnace {
    fn name(&self) -> String {
        match self {
            Furnace::Stone => "Stone Furnace",
            Furnace::Steel => "Steel Furnace",
            Furnace::Electric => "Electric Furnace",
        }.to_owned()
    }

    fn speed(&self) -> f32 {
        match self {
            Furnace::Stone => 1.,
            Furnace::Steel => 2.,
            Furnace::Electric => 2.,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Assembler {
    Basic,
    Blue,
    Green,
}

impl CraftTech for Assembler {
    fn name(&self) -> String {
        match self {
            Assembler::Basic => "Basic Assembler",
            Assembler::Blue => "Blue Assembler",
            Assembler::Green => "Green Assembler",
        }.to_owned()
    }

    fn speed(&self) -> f32 {
        match self {
            Assembler::Basic => 0.50,
            Assembler::Blue => 0.75,
            Assembler::Green => 1.25,
        }
    }
}

struct SimpleCraftTech {
    name: String,
    speed: f32,
}

impl SimpleCraftTech {
    pub fn new<S: AsRef<str>>(name: S, speed: f32) -> Self {
        SimpleCraftTech { name: name.as_ref().to_owned(), speed}
    }
}

impl CraftTech for SimpleCraftTech {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn speed(&self) -> f32 {
        self.speed
    }
}