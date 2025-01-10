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
    data_size: usize,
    invocations: u32,
    iterations: u16,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Args {
    const OPT_HELP: &'static str = "-help";
    const OPT_CONFIG_JSON: &'static str = "-config";
    const OPT_DATA_SIZE : &'static str = "-data_size";
    const OPT_INVOCATIONS: &'static str = "-invocations";
    const OPT_ITERATIONS: &'static str = "-iterations";

    pub fn parse(args: Vec<String>) -> Result<Self> {
        let mut config_json: String = String::new();
        let mut data_size: usize = 1024;
        let mut invocations: u32 = 1000;
        let mut iterations: u16 = 5;

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
                Self::OPT_DATA_SIZE => {
                    i += 1;
                    data_size = args[i].parse::<usize>().unwrap();
                }
                Self::OPT_INVOCATIONS => {
                    i += 1;
                    invocations = args[i].parse::<u32>().unwrap();
                }
                Self::OPT_ITERATIONS => {
                    i += 1;
                    iterations = args[i].parse::<u16>().unwrap();
                }
                _ => {
                    return Err(anyhow::anyhow!("invalid argument"));
                },
            }

            i += 1;
        }

        Ok(Self {
            config: config_json,
            data_size,
            invocations,
            iterations
        })
    }

    pub fn usage(program_name: &str) {
        println!(
            "Usage: {} {} <config.json> [{} <data_size> {} <invocations> {} <iterations> ]",
            program_name,
            Self::OPT_CONFIG_JSON,
            Self::OPT_DATA_SIZE,
            Self::OPT_INVOCATIONS,
            Self::OPT_ITERATIONS
        );
    }

    pub fn config(&self) -> &str {
        &self.config
    }

    pub fn data_size(&self) -> usize {
        self.data_size
    }

    pub fn invocations(&self) -> u32 {
        self.invocations
    }

    pub fn iterations(&self) -> u16 {
        self.iterations
    }
}
