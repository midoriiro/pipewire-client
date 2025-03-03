use crate::error::Error;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Input,
    Output,
}

impl From<Direction> for pipewire::spa::utils::Direction {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Input => pipewire::spa::utils::Direction::Input,
            Direction::Output => pipewire::spa::utils::Direction::Output,
        }
    }
}

pub fn dict_ref_to_hashmap(dict: &pipewire::spa::utils::dict::DictRef) -> HashMap<String, String> {
    dict
        .iter()
        .map(move |(k, v)| {
            let k = String::from(k).clone();
            let v = String::from(v).clone();
            (k, v)
        })
        .collect::<HashMap<_, _>>()
}

pub fn debug_dict_ref(dict: &pipewire::spa::utils::dict::DictRef) {
    for (key, value) in dict.iter() {
        println!("{} => {}", key ,value);
    }
    println!("\n");
}



pub struct Backoff {
    attempts: u32,
    maximum_attempts: u32,
    wait_duration: std::time::Duration,
    initial_wait_duration: std::time::Duration,
    maximum_wait_duration: std::time::Duration,
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new(
            300, // 300 attempts * 100ms = 30s
            std::time::Duration::from_millis(100),
            std::time::Duration::from_millis(100)
        )
    }
}

impl Backoff {
    pub fn constant(milliseconds: u128) -> Self {
        let attempts = milliseconds / 100;
        Self::new(
            attempts as u32,
            std::time::Duration::from_millis(100),
            std::time::Duration::from_millis(100)
        )
    }
}

impl Backoff {
    pub fn new(
        maximum_attempts: u32,
        initial_wait_duration: std::time::Duration,
        maximum_wait_duration: std::time::Duration
    ) -> Self {
        Self {
            attempts: 0,
            maximum_attempts,
            wait_duration: initial_wait_duration,
            initial_wait_duration,
            maximum_wait_duration,
        }
    }
    
    pub fn reset(&mut self) {
        self.attempts = 0;
        self.wait_duration = self.initial_wait_duration;
    }

    pub fn retry<F, O, E>(&mut self, mut operation: F) -> Result<O, Error>
    where
        F: FnMut() -> Result<O, E>,
        E: std::error::Error
    {
        self.reset();
        loop {
            let error = match operation() {
                Ok(value) => return Ok(value),
                Err(value) => value
            };
            std::thread::sleep(self.wait_duration);
            self.wait_duration = self.maximum_wait_duration.min(self.wait_duration * 2);
            self.attempts += 1;
            if self.attempts < self.maximum_attempts {
                continue;
            }
            return Err(Error {
                description: format!("Backoff timeout: {}", error.to_string()),
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Bytes,
    KB,
    MB,
    GB,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    value: f64,
    unit: Unit,
}

impl Size {
    const fn new(value: f64, unit: Unit) -> Self {
        Self { value, unit }
    }

    pub const fn from_bytes(bytes: f64) -> Self {
        Self::new(bytes, Unit::Bytes)
    }

    pub const fn from_kb(kb: f64) -> Self {
        Self::new(kb, Unit::KB)
    }

    pub const fn from_mb(mb: f64) -> Self {
        Self::new(mb, Unit::MB)
    }

    pub const fn from_gb(gb: f64) -> Self {
        Self::new(gb, Unit::GB)
    }
}

impl From<String> for Size {
    fn from(value: String) -> Self {
        let value = value.trim();
        let unit = value.chars().last().unwrap();
        let unit = match unit {
            'b' => Unit::Bytes,
            'k' => Unit::KB,
            'm' => Unit::MB,
            'g' => Unit::GB,
            _ => panic!("Invalid unit {:?}. Only b, k, m, g are supported.", unit),
        };
        let value = value.chars().take(value.len() - 2).collect::<String>();
        let value = value.parse::<f64>().unwrap();
        Self::new(value, unit)
    }
}

impl From<Size> for u64 {
    fn from(value: Size) -> Self {
        match value.unit {
            Unit::Bytes => value.value as u64,
            Unit::KB => (value.value * 1024.0) as u64,
            Unit::MB => (value.value * 1024.0 * 1024.0) as u64,
            Unit::GB => (value.value * 1024.0 * 1024.0 * 1024.0) as u64,
        }
    }
}

impl From<Size> for i64 {
    fn from(value: Size) -> Self {
        let value: u64 = value.into();
        value as i64
    }
}

pub struct HexSlice<'a>(pub &'a [u8]);

impl<'a> Display for HexSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.0 {
            write!(f, "{:0>2x}", byte)?;
        }
        Ok(())
    }
}