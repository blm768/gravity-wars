#![feature(try_from)]

extern crate cgmath;
extern crate gltf;
extern crate rgb;
extern crate wasm_bindgen;
extern crate web_sys;

pub mod glue;
pub mod mapgen;
pub mod rendering;
pub mod state;