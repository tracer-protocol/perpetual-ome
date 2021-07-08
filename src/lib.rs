#![feature(map_first_last)]
#![feature(result_contains_err)]
#![feature(destructuring_assignment)]

#[macro_use]
extern crate enum_display_derive;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

pub mod book;
pub mod order;
pub mod state;
pub mod util;
