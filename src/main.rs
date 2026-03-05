use std::collections::HashMap;
use std::env;
use std::fs;
use serde::Deserialize;

mod settings;

fn main() {
    if env::args().len() < 2 {
        println!("Usage: [executable] [settings file path]");
        return
    }
}