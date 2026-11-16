#![no_std]

pub mod dispatch;
pub mod ghost;
pub mod keymap;
pub mod protocol;
pub mod settings;

#[cfg(test)]
#[path = "../tests/dispatch.rs"]
mod dispatch_tests;
