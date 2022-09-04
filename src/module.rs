use std::fmt::Display;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Module {
    Speed1,
    Speed2,
    Speed3,
    Productivity1,
    Productivity2,
    Productivity3,
}

impl Module {
    fn speed_bonus(&self) -> f32 {
        match self {
            Module::Speed1 => 0.2,
            Module::Speed2 => 0.3,
            Module::Speed3 => 0.5,
            Module::Productivity1 => -0.05,
            Module::Productivity2 => -0.10,
            Module::Productivity3 => -0.15,
        }
    }

    fn productivity_bonus(&self) -> f32 {
        match self {
            Module::Speed1 => 0.0,
            Module::Speed2 => 0.0,
            Module::Speed3 => 0.0,
            Module::Productivity1 => 0.04,
            Module::Productivity2 => 0.06,
            Module::Productivity3 => 0.10,
        }
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &str = match self {
            Module::Speed1 => "S1",
            Module::Speed2 => "S2",
            Module::Speed3 => "S3",
            Module::Productivity1 => "P1",
            Module::Productivity2 => "P2",
            Module::Productivity3 => "P3",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone)]
pub struct ModuleList<const N: usize> {
    modules: Vec<Module>,
}

impl<const N: usize> ModuleList<N> {
    pub fn new() -> Self {
        ModuleList {
            modules: Vec::new(),
        }
    }

    pub fn add_module(&mut self, module: Module) {
        if N <= self.modules.len() {
            panic!("Cannot add module. Reached limit of {} modules", N);
        }
        self.modules.push(module);
    }

    pub fn speed_bonus(&self) -> f32 {
        self.modules
            .iter()
            .map(|m| m.speed_bonus())
            .reduce(|x, y| x + y)
            .unwrap_or(0.0)
    }

    pub fn productivity_bonus(&self) -> f32 {
        self.modules
            .iter()
            .map(|m| m.productivity_bonus())
            .reduce(|x, y| x + y)
            .unwrap_or(0.0)
    }
}

impl<const N: usize> Display for ModuleList<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(")?;
        for x in 0..N {
            if x < self.modules.len() {
                self.modules[x].fmt(f)?;
            } else {
                f.write_str(".")?;
            }
        }
        f.write_str(")")
    }
}
