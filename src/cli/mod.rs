use std::{fmt::Display, process, str::FromStr};

mod load;
mod parse;
mod root;
mod server;
mod validate;

pub use parse::root_config as parse;
