#![feature(phase)]
#![feature(default_type_params)]

#[phase(plugin, link)]
extern crate log;

#[phase(plugin, link)]
extern crate common;

pub mod protocol;
