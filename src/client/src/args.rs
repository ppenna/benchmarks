// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Imports
//==================================================================================================

use ::anyhow::Result;

//==================================================================================================
// Structures
//==================================================================================================

///
/// # Description
///
/// This structure packs the command-line arguments that were passed to the program.
///
pub struct Args {
    /// Socket address to connect to.
    connect_sockaddr: String,
    /// Inter-arrival time.
    frequency: u128,
    /// duration.
    duration: u64,
    /// Data size.
    size: usize,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Args {
    /// Command-line option for printing the help message.
    const OPT_HELP: &'static str = "-help";
    /// Package injection frequency.
    const OPT_FREQUENCY: &'static str = "-frequency";
    /// duration.
    const OPT_DURATION: &'static str = "-duration";
    /// Socket address to connect to.
    const OPT_CONNECT_SOCKADDR: &'static str = "-connect";
    /// Data size.
    const OPT_SIZE: &'static str = "-size";

    ///
    /// # Description
    ///
    /// Parses the command-line arguments that were passed to the program.
    ///
    /// # Parameters
    ///
    /// - `args`: Command-line arguments.
    ///
    /// # Returns
    ///
    /// Upon success, the function returns the parsed command-line arguments that were passed to the
    /// program. Upon failure, the function returns an error.
    ///
    pub fn parse(args: Vec<String>) -> Result<Self> {
        trace!("parse(): parsing command-line arguments...");

        let mut server_sockaddr: String = String::new();
        let mut interarrival: u128 = 0;
        let mut duration: u64 = 0;
        let mut size: usize = 0;

        let mut i: usize = 1;
        while i < args.len() {
            match args[i].as_str() {
                Self::OPT_HELP => {
                    Self::usage(args[0].as_str());
                    return Err(anyhow::anyhow!("help message"));
                },
                Self::OPT_FREQUENCY => {
                    i += 1;
                    interarrival = match args[i].parse::<u128>() {
                        Ok(num) => num,
                        Err(_) => {
                            return Err(anyhow::anyhow!("invalid package injection frequency"));
                        },
                    };
                },
                Self::OPT_CONNECT_SOCKADDR => {
                    i += 1;
                    server_sockaddr = args[i].clone();
                },
                Self::OPT_DURATION => {
                    i += 1;
                    duration = match args[i].parse::<u64>() {
                        Ok(num) => num,
                        Err(_) => {
                            return Err(anyhow::anyhow!("invalid duration"));
                        },
                    };
                },
                Self::OPT_SIZE => {
                    i += 1;
                    size = match args[i].parse::<usize>() {
                        Ok(num) => num,
                        Err(_) => {
                            return Err(anyhow::anyhow!("invalid data size"));
                        },
                    };
                },
                arg => {
                    return Err(anyhow::anyhow!("invalid argument (arg={})", arg));
                },
            }

            i += 1;
        }

        Ok(Self {
            frequency: interarrival,
            duration,
            connect_sockaddr: server_sockaddr,
            size,
        })
    }

    ///
    /// # Description
    ///
    /// Prints program usage.
    ///
    /// # Parameters
    ///
    /// - `program_name`: Name of the program.
    ///
    pub fn usage(program_name: &str) {
        println!(
            "Usage: {} {} <injection-frequency> {} <server-sockaddr> {} <duration>",
            program_name,
            Self::OPT_FREQUENCY,
            Self::OPT_CONNECT_SOCKADDR,
            Self::OPT_DURATION
        );
    }

    ///
    /// # Description
    ///
    /// Returns the package-injection frequency.
    ///
    /// # Returns
    ///
    /// The package-injection frequency.
    ///
    pub fn frequency(&self) -> u128 {
        self.frequency
    }

    ///
    /// # Description
    ///
    /// Returns the socket address to connect to.
    ///
    /// # Returns
    ///
    /// The socket address to connect to.
    ///
    pub fn connect_sockaddr(&self) -> String {
        self.connect_sockaddr.to_string()
    }

    ///
    /// # Description
    ///
    /// Returns the duration.
    ///
    /// # Returns
    ///
    /// The duration.
    ///
    pub fn duration(&self) -> u64 {
        self.duration
    }

    ///
    /// # Description
    ///
    /// Returns the data size.
    ///
    /// # Returns
    ///
    /// The data size.
    ///
    pub fn size(&self) -> usize {
        self.size
    }
}
