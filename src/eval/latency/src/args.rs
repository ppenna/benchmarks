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
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Args {
    const OPT_HELP: &'static str = "-help";
    const OPT_CONFIG_JSON: &'static str = "-config";

    pub fn parse(args: Vec<String>) -> Result<Self> {
        let mut config_json: String = String::new();

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
                _ => {
                    return Err(anyhow::anyhow!("invalid argument"));
                },
            }

            i += 1;
        }

        Ok(Self {
            config: config_json,
        })
    }

    pub fn usage(program_name: &str) {
        println!(
            "Usage: {} {} <config.json>",
            program_name,
            Self::OPT_CONFIG_JSON,
        );
    }

    pub fn config(&self) -> &str {
        &self.config
    }
}
