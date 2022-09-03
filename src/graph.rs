use std::fmt::{Write};

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Green,
    Yellow,
    Orange,
    Red,
    Blue,
    Black,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub amount: f32,
    pub source_amount: f32,
    pub source_name: String,
    pub color: Color,
}

impl Node {
    pub fn new<S1, S2, S3>(
        id: S1,
        name: S2,
        amount: f32,
        source_amount: f32,
        source_name: S3,
        color: Color,
    ) -> Self
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        S3: AsRef<str>,
    {
        Node {
            id: id.as_ref().to_owned(),
            name: name.as_ref().to_owned(),
            amount,
            source_amount,
            source_name: source_name.as_ref().to_owned(),
            color,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub amount: f32,
    pub source_amount: f32,
    pub color: Color,
}

impl Edge {
    pub fn new<S1, S2>(from: S1, to: S2, amount: f32, source_amount: f32, color: Color) -> Self
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        Edge {
            from: from.as_ref().to_owned(),
            to: to.as_ref().to_owned(),
            amount,
            source_amount,
            color,
        }
    }
}
