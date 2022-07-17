// Copyright 2022 nitepone <luna@night.horse>

extern crate rand;

pub mod error;
mod game;

use crate::game::StdMinrsGame;

fn main() {
    let mut _game = StdMinrsGame::new(8, 8, 10).unwrap();
    println!("Testing one two, this thing on?");
}
