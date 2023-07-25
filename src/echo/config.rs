use metron::LogLevel;
use serde::{Deserialize, Serialize};

use crate::runtime;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    // TODO: Why bother making this common between configs? Seems like a bad
    // idea This class is really the user interface and if it doesn't make sense
    // to expose runtime configuration for the server then I shouldn't. There
    // will be an idiomatic way of allowing the runtime configuration to be
    // specified for some commands and not others.
    pub runtime: runtime::Config,
    pub log_level: LogLevel,
}
