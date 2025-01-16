// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Imports
//==================================================================================================

use ::anyhow::Result;

//==================================================================================================
// Structures
//==================================================================================================

pub struct Args {
    config: String,
    // This defines the minimum memory limit that the evaluation will allow the system to go, before stopping for each sandbox
    memory_limit: u64,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Args {
    const OPT_HELP: &'static str = "-help";
    const OPT_CONFIG_JSON: &'static str = "-config";
    const OPT_MEMORY_LIMIT: &'static str = "-memory-limit";

    pub fn parse(args: Vec<String>) -> Result<Self> {
        let mut config_json: String = String::new();
        let mut memory_limit: u64 = 512;

        let mut i: usize = 1;
        while i < args.len() {
            match args[i].as_str() {
                Self::OPT_HELP => {
                    Self::usage(args[0].as_str());
                    return Err(anyhow::anyhow!("wrong usage"));
                },
                Self::OPT_CONFIG_JSON => {
                    i += 1;
                    config_json = args[i].clone();
                },
                Self::OPT_MEMORY_LIMIT => {
                    i += 1;
                    memory_limit = args[i].parse::<u64>().unwrap();
                }
                _ => {
                    return Err(anyhow::anyhow!("invalid argument"));
                },
            }

            i += 1;
        }

        Ok(Self {
            config: config_json,
            memory_limit,
        })
    }

    pub fn usage(program_name: &str) {
        println!(
            "Usage: {} {} <config.json> {} [memory_limit_in_mb]",
            program_name,
            Self::OPT_CONFIG_JSON,
            Self::OPT_MEMORY_LIMIT,
        );
    }

    pub fn config(&self) -> &str {
        &self.config
    }

    pub fn memory_limit(&self) -> u64 {
        self.memory_limit
    }
}
