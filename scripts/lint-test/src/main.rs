// Temporary test file for verifying ra-check.sh produces diagnostics.
// Delete after confirming lint integration works.
//
// Expected warnings:
//   clippy  — needless_return, let_and_return, manual_map
//   bevy    — missing_reflect, panicking_methods

#![feature(register_tool)]
#![register_tool(bevy)]

use bevy::prelude::*;

// --- bevy_lint: missing_reflect -----------------------------------------------

#[derive(Component, Debug)]
pub struct UnreflectedComponent {
    pub value: i32,
}

#[derive(Resource)]
pub struct UnreflectedResource {
    pub data: String,
}

// --- bevy_lint: panicking_methods ---------------------------------------------

fn panicking_system(query: Query<&Transform>) {
    let _tf = query.single().unwrap();
}

// --- clippy: needless_return --------------------------------------------------

fn redundant_return() -> i32 {
    return 42;
}

// --- clippy: let_and_return ---------------------------------------------------

fn let_and_return() -> i32 {
    let x = 1 + 2;
    x
}

// --- clippy: manual_map -------------------------------------------------------

fn manual_map(opt: Option<i32>) -> Option<String> {
    match opt {
        Some(v) => Some(v.to_string()),
        None => None,
    }
}

fn main() {}
