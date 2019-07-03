#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate failure;
#[macro_use] extern crate rocket;

pub mod http;
pub mod file_manager;