// Copyright(c) Microsoft Corporation.
// Licensed under the MIT License.

//==================================================================================================
// Modules
//==================================================================================================

mod logging;

//==================================================================================================
// Imports
//==================================================================================================

use ::anyhow::Result;
use ::hyperlight_common::flatbuffer_wrappers::function_types::{
    ParameterValue,
    ReturnType,
};
use ::hyperlight_host::{
    func::{
        HostFunction0,
        ReturnValue,
    },
    sandbox_state::{
        sandbox::EvolvableSandbox,
        transition::Noop,
    },
    HyperlightError,
    MultiUseSandbox,
    UninitializedSandbox,
};
use ::std::{
    sync::{
        Arc,
        Mutex,
    },
    thread,
};

//==================================================================================================
// Standalone Functions
//==================================================================================================

fn main() -> Result<()> {
    logging::initialize(false);
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <guest_binary_path>", args[0]);
        std::process::exit(1);
    }
    let filepath: String = args[1].clone();

    // Create an uninitialized sandbox with a guest binary
    let mut uninitialized_sandbox: UninitializedSandbox = UninitializedSandbox::new(
        hyperlight_host::GuestBinary::FilePath(filepath),
        None, // default configuration
        None, // default run options
        None, // default host print function
    )?;

    // Register a host function
    fn sleep_5_secs() -> hyperlight_host::Result<()> {
        thread::sleep(std::time::Duration::from_secs(5));
        Ok(())
    }

    let host_function: Arc<Mutex<fn() -> hyperlight_host::Result<()>>> =
        Arc::new(Mutex::new(sleep_5_secs));

    // Registering a host function makes it available to be called by the guest
    host_function.register(&mut uninitialized_sandbox, "Sleep5Secs")?;
    // Note: This function is unused by the guest code below, it's just here for demonstration purposes

    // Initialize sandbox to be able to call host functions
    let mut multi_use_sandbox: MultiUseSandbox = uninitialized_sandbox.evolve(Noop::default())?;

    // Call a function in the guest
    let message: String = "Hello, World! I am executing inside of a VM :)\n".to_string();
    // in order to call a function it first must be defined in the guest and exposed so that
    // the host can call it
    let result: Result<ReturnValue, HyperlightError> = multi_use_sandbox
        .call_guest_function_by_name(
            "PrintOutput",
            ReturnType::Int,
            Some(vec![ParameterValue::String(message.clone())]),
        );

    assert!(result.is_ok());

    Ok(())
}
