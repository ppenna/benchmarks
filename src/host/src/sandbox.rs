// Copyright(c) Microsoft Corporation.
// Licensed under the MIT License.

//==================================================================================================
// Imports
//==================================================================================================

use ::anyhow::Result;
use ::hyperlight_host::{
    func::{
        HostFunction0,
        HostFunction1,
    },
    sandbox_state::{
        sandbox::EvolvableSandbox,
        transition::Noop,
    },
    HyperlightError,
    MultiUseSandbox,
    UninitializedSandbox,
};
use ::std::sync::{
    mpsc,
    Arc,
    Mutex,
};

//==================================================================================================
// Structures
//==================================================================================================

#[derive(Debug, Clone)]
pub struct Sandbox {
    filepath: String,
}

impl Sandbox {
    pub fn new(filepath: &str) -> Self {
        Self {
            filepath: filepath.to_string(),
        }
    }

    pub fn run(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        let (input_tx, input_rx) = mpsc::channel::<Vec<u8>>();
        let (output_tx, output_rx) = mpsc::channel::<Vec<u8>>();

        let vmbus_write = move |data: Vec<u8>| -> Result<i32, HyperlightError> {
            // write to output_tx
            output_tx.send(data).unwrap();

            Ok(0)
        };

        let vmbus_read = move || -> Result<Vec<u8>, HyperlightError> {
            // read from input_rx
            let data = input_rx.recv().unwrap();

            Ok(data)
        };

        let vmbus_write_host_fn = Arc::new(Mutex::new(vmbus_write));
        let vmbus_read_host_fn = Arc::new(Mutex::new(vmbus_read));

        // Create an uninitialized sandbox with a guest binary
        let mut sandbox: UninitializedSandbox = UninitializedSandbox::new(
            hyperlight_host::GuestBinary::FilePath(self.filepath.to_string()),
            None, // default configuration
            None, // default run options
            None, // default host print function
        )?;

        vmbus_read_host_fn.register(&mut sandbox, "VmbusRead")?;
        vmbus_write_host_fn.register(&mut sandbox, "VmbusWrite")?;

        input_tx.send(data).unwrap();

        // Initialize sandbox to be able to call host functions
        let _multi_use_sandbox: MultiUseSandbox = sandbox.evolve(Noop::default())?;

        let data = output_rx.recv().unwrap();

        Ok(data)
    }
}
