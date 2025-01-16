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
    listen_sockaddr: String,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Args {
    const OPT_HELP: &'static str = "-help";
    const OPT_LISTEN_SOCKADDR: &'static str = "-listen";

    pub fn parse(args: Vec<String>) -> Result<Self> {
        let mut http_sockaddr: String = String::new();

        let mut i: usize = 1;
        while i < args.len() {
            match args[i].as_str() {
                Self::OPT_HELP => {
                    Self::usage(args[0].as_str());
                    return Err(anyhow::anyhow!("wrong usage"));
                },
                Self::OPT_LISTEN_SOCKADDR => {
                    i += 1;
                    http_sockaddr = args[i].clone();
                },
                _ => {
                    return Err(anyhow::anyhow!("invalid argument"));
                },
            }

            i += 1;
        }

        Ok(Self {
            listen_sockaddr: http_sockaddr,
        })
    }

    pub fn usage(program_name: &str) {
        println!(
            "Usage: {} {} <sockaddr>",
            program_name,
            Self::OPT_LISTEN_SOCKADDR,
        );
    }

    pub fn listen_sockaddr(&self) -> &str {
        &self.listen_sockaddr
    }
}
