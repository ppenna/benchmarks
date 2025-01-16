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
    guest: String,
    init_sandbox_size: usize,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Args {
    const OPT_HELP: &'static str = "-help";
    const OPT_LISTEN_SOCKADDR: &'static str = "-listen";
    const OPT_GUEST: &'static str = "-guest";
    const OPT_INIT_SANDBOX_SIZE: &'static str = "-init-sandbox-size";

    pub fn parse(args: Vec<String>) -> Result<Self> {
        let mut http_sockaddr: String = String::new();
        let mut guest: String = String::new();
        let mut init_sandbox_size: usize = 0;

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
                Self::OPT_GUEST => {
                    i += 1;
                    guest = args[i].clone();
                },
                Self::OPT_INIT_SANDBOX_SIZE => {
                    i += 1;
                    init_sandbox_size = args[i].parse::<usize>().unwrap();
                },
                _ => {
                    return Err(anyhow::anyhow!("invalid argument"));
                },
            }

            i += 1;
        }

        Ok(Self {
            listen_sockaddr: http_sockaddr,
            guest,
            init_sandbox_size,
        })
    }

    pub fn usage(program_name: &str) {
        println!(
            "Usage: {} {} <sockaddr> {} <filepath> {} <init-sandbox-size>",
            program_name,
            Self::OPT_LISTEN_SOCKADDR,
            Self::OPT_GUEST,
            Self::OPT_INIT_SANDBOX_SIZE,
        );
    }

    pub fn listen_sockaddr(&self) -> &str {
        &self.listen_sockaddr
    }

    pub fn guest(&self) -> &str {
        &self.guest
    }

    pub fn init_sandbox_size(&self) -> usize {
        self.init_sandbox_size
    }
}
