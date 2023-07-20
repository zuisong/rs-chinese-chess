#![allow(dead_code)]
#![feature(lazy_cell)]
extern crate engine;

use engine::engine::UCCIEngine;

fn main() {
    let p = module_path!();

    dbg!(p);

    UCCIEngine::new(include_str!("../../BOOK.DAT").into()).start();
}
