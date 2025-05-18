use std::{
    ops::{Add, AddAssign, Deref},
    str::FromStr,
};

use crate::error::{UnknownError, unknown_error_expected};

#[derive(Debug, Clone)]
pub enum MatchMode {
    Normal,
    Edges,
}

impl FromStr for MatchMode {
    type Err = UnknownError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "n" | "N" | "Normal" | "normal" => Ok(MatchMode::Normal),
            "e" | "E" | "Edges" | "edges" => Ok(MatchMode::Edges),
            value => Err(UnknownError {
                name: "MatchMode".into(),
                value: value.into(),
                expected: unknown_error_expected!(
                    "n" | "N" | "Normal" | "normal" => "Normal",
                    "e" | "E" | "Edges" | "edges" => "Edges"
                ),
            }),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Add for Position {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Position {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<'a> Add<&'a Position> for Position {
    type Output = Self;
    fn add(self, other: &'a Position) -> Self {
        Position {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<'a> AddAssign<&'a Position> for Position {
    fn add_assign(&mut self, rhs: &'a Position) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

#[cfg(target_arch = "wasm32")]
impl Position {
    pub fn to_json(self) -> js_sys::Object {
        let obj = js_sys::Object::new();

        js_sys::Reflect::set(
            &obj,
            &wasm_bindgen::JsValue::from_str("x"),
            &wasm_bindgen::JsValue::from(self.x),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &wasm_bindgen::JsValue::from_str("y"),
            &wasm_bindgen::JsValue::from(self.y),
        )
        .ok();

        obj
    }
}

#[derive(Debug, Default)]
pub struct OverlapScore {
    pub score: u64,
    pub flipped: bool,
    pub position: Position,
}

impl Deref for OverlapScore {
    type Target = Position;

    fn deref(&self) -> &Self::Target {
        &self.position
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CheckDirection {
    Vertical,
    Horizontal,
    Sideways,
}

impl FromStr for CheckDirection {
    type Err = UnknownError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v" | "V" | "Vertical" | "vertical" => Ok(CheckDirection::Vertical),
            "h" | "H" | "Horizontal" | "horizontal" => Ok(CheckDirection::Horizontal),
            "s" | "S" | "Sideways" | "sideways" => Ok(CheckDirection::Sideways),
            value => Err(UnknownError {
                name: "CheckDirection".into(),
                value: value.into(),
                expected: unknown_error_expected!(
                    "v" | "V" | "Vertical" | "vertical" => "Vertical",
                    "h" | "H" | "Horizontal" | "horizontal" => "Horizontal",
                    "s" | "S" | "Sideways" | "sideways" => "Sideways"
                ),
            }),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Order {
    Ordered,
    Unordered,
}

impl FromStr for Order {
    type Err = UnknownError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "o" | "O" | "Ordered" | "ordered" => Ok(Order::Ordered),
            "u" | "U" | "Unordered" | "unordered" => Ok(Order::Unordered),
            value => Err(UnknownError {
                name: "CheckDirection".into(),
                value: value.into(),
                expected: unknown_error_expected!(
                    "o" | "O" | "Ordered" | "ordered" => "Ordered",
                    "u" | "U" | "Unordered" | "unordered" => "Unordered"
                ),
            }),
        }
    }
}
