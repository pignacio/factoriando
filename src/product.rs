use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::module::ModuleList;

#[derive(Debug, Serialize, Deserialize)]

pub struct ProductRow {
    pub id: String,
    pub name: String,
    pub craft_duration: f32,
    pub craft_type: CraftType,
    pub quantity: i32,
    dependencies: String,
}

impl ProductRow {
    pub fn into_domain(self) -> Product {
        Product {
            id: self.id,
            name: self.name,
            craft_duration: self.craft_duration,
            craft_type: self.craft_type,
            quantity: self.quantity,
            dependencies: ProductRow::build_dependencies(&self.dependencies),
        }
    }

    fn build_dependencies(dependencies: &str) -> HashMap<String, f32> {
        if dependencies.is_empty() {
            return HashMap::new();
        }
        dependencies
            .split(';')
            .map(|s| {
                let split: Vec<&str> = s.split(':').collect();
                (
                    split[0].to_owned(),
                    split[1]
                        .parse::<f32>()
                        .map_err(|_e| format!("Could not parse {}", split[1]))
                        .unwrap(),
                )
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
            CraftType::Ore => tech_status.miner_speed(),
            CraftType::Smelt => tech_status.furnace_speed(),
            CraftType::Assemble => tech_status.assembler_speed(),
            CraftType::Chemical => tech_status.chemical_speed(),
            CraftType::Silo => tech_status.silo_speed(),
            CraftType::Launch => 1.0,
        }
    }

    pub fn craft_tech<'a>(&self, tech_status: &'a CraftTechStatus) -> &'a dyn CraftTech {
        match &self {
            CraftType::Ore => &tech_status.miner,
            CraftType::Smelt => &tech_status.furnace,
            CraftType::Assemble => &tech_status.assembler,
            CraftType::Chemical => &tech_status.chemical,
            CraftType::Silo => &tech_status.silo,
            CraftType::Launch => &tech_status.silo,
        }
    }
}

pub struct CraftTechStatus {
    miner: Miner,
    furnace: Furnace,
    assembler: Assembler,
    chemical: SimpleCraftTech,
    silo: SimpleCraftTech,
    override_by_product: HashMap<String, Box<dyn CraftTech>>,
}

impl CraftTechStatus {
    pub fn new(miner: Miner, furnace: Furnace, assembler: Assembler) -> Self {
        CraftTechStatus {
            miner,
            furnace,
            assembler,
            chemical: SimpleCraftTech::new("Chemical plant", 1., 1.),
            silo: SimpleCraftTech::new("Rocket Silo", 1., 1.),
            override_by_product: HashMap::new(),
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

    pub fn add_override<S: AsRef<str>>(&mut self, product_id: S, tech: Box<dyn CraftTech>) {
        self.override_by_product
            .insert(product_id.as_ref().to_owned(), tech);
    }

    pub fn tech_for(&self, product: &Product) -> &dyn CraftTech {
        self.override_by_product
            .get(&product.id)
            .map(|b| b.as_ref())
            .unwrap_or_else(|| self.default_tech_for(product))
    }

    fn default_tech_for(&self, product: &Product) -> &dyn CraftTech {
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
    fn productivity(&self) -> f32;
    fn add_module(&mut self, module: crate::module::Module);
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
        }
        .to_owned()
    }

    fn speed(&self) -> f32 {
        match self {
            Miner::Burner => 0.25,
            Miner::Electric => 0.5,
        }
    }

    fn productivity(&self) -> f32 {
        1.0
    }

    fn add_module(&mut self, _module: crate::module::Module) {
        panic!("Modules are not supported in miners");
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
        }
        .to_owned()
    }

    fn speed(&self) -> f32 {
        match self {
            Furnace::Stone => 1.,
            Furnace::Steel => 2.,
            Furnace::Electric => 2.,
        }
    }

    fn productivity(&self) -> f32 {
        1.0
    }

    fn add_module(&mut self, _module: crate::module::Module) {
        panic!("Modules in furnaces not supported")
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Assembler {
    Basic,
    Blue { modules: ModuleList<2> },
    Green { modules: ModuleList<4> },
}

impl Assembler {
    pub fn blue() -> Self {
        Assembler::Blue {
            modules: ModuleList::new(),
        }
    }

    pub fn green() -> Self {
        Assembler::Green {
            modules: ModuleList::new(),
        }
    }

    fn base_speed(&self) -> f32 {
        match self {
            Assembler::Basic => 0.50,
            Assembler::Blue { .. } => 0.75,
            Assembler::Green { .. } => 1.25,
        }
    }

    fn speed_multiplier(&self) -> f32 {
        match self {
            Assembler::Basic => 0.0,
            Assembler::Blue { modules } => 1. + modules.speed_bonus(),
            Assembler::Green { modules } => 1. + modules.speed_bonus(),
        }
    }
}

impl CraftTech for Assembler {
    fn name(&self) -> String {
        return match self {
            Assembler::Basic => "Basic Assembler".to_owned(),
            Assembler::Blue { modules } => format!("Blue Assembler {}", modules),
            Assembler::Green { modules } => format!("Green Assembler {}", modules),
        };
    }

    fn speed(&self) -> f32 {
        self.base_speed() * self.speed_multiplier()
    }

    fn productivity(&self) -> f32 {
        match self {
            Assembler::Basic => 1.0,
            Assembler::Blue { modules } => 1. + modules.productivity_bonus(),
            Assembler::Green { modules } => 1. + modules.productivity_bonus(),
        }
    }

    fn add_module(&mut self, module: crate::module::Module) {
        return match self {
            Assembler::Basic => panic!("Cannot add modules to basic assembler"),
            Assembler::Blue { modules } => modules.add_module(module),
            Assembler::Green { modules } => modules.add_module(module),
        };
    }
}

struct SimpleCraftTech {
    name: String,
    speed: f32,
    productivity: f32,
}

impl SimpleCraftTech {
    pub fn new<S: AsRef<str>>(name: S, speed: f32, productivity: f32) -> Self {
        SimpleCraftTech {
            name: name.as_ref().to_owned(),
            speed,
            productivity,
        }
    }
}

impl CraftTech for SimpleCraftTech {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn speed(&self) -> f32 {
        self.speed
    }

    fn productivity(&self) -> f32 {
        self.productivity
    }

    fn add_module(&mut self, _module: crate::module::Module) {
        panic!("Cannot add module to SimpleCraftTech");
    }
}
