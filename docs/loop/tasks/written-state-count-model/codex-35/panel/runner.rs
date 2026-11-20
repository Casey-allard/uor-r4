#![forbid(unsafe_code)]

mod batch;
mod common;

fn main() -> common::Result<()> {
    batch::run(&std::env::args().skip(1).collect::<Vec<_>>())
}
