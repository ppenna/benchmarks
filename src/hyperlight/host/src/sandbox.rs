// Copyright(c) Microsoft Corporation.
// Licensed under the MIT License.

//==================================================================================================
// Imports
//==================================================================================================

use ::anyhow::Result;
use hyperlight_host::func::ReturnType;
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

pub struct Sandbox {
    filepath: String,
    input_tx: Option<mpsc::Sender<Vec<u8>>>,
    output_rx: Option<mpsc::Receiver<Vec<u8>>>,
    sandbox: Option<MultiUseSandbox>,
}

impl Sandbox {
    pub fn new(filepath: &str) -> Self {
        Self {
            filepath: filepath.to_string(),
            input_tx: None,
            output_rx: None,
            sandbox: None,
        }
    }

    pub fn init(&mut self) -> Result<()> {
        // Create an uninitialized sandbox with a guest binary
        let mut sandbox: UninitializedSandbox = UninitializedSandbox::new(
            hyperlight_host::GuestBinary::FilePath(self.filepath.to_string()),
            None, // default configuration
            None, // default run options
            None, // default host print function
        )?;

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

        vmbus_read_host_fn.register(&mut sandbox, "VmbusRead")?;
        vmbus_write_host_fn.register(&mut sandbox, "VmbusWrite")?;

        self.input_tx = Some(input_tx);
        self.output_rx = Some(output_rx);

        let multi_use_sandbox: MultiUseSandbox = sandbox.evolve(Noop::default())?;

        self.sandbox = Some(multi_use_sandbox);

        Ok(())
    }

    pub fn run(&mut self, data: Vec<u8>) -> Result<Vec<u8>> {


        self.input_tx.as_mut().unwrap().send(data).unwrap();

        // Initialize sandbox to be able to call host functions
        let sandbox = self.sandbox.as_mut().unwrap();
        let _return_value = sandbox.call_guest_function_by_name("GuestFunction", ReturnType::Void,         Some(Vec::new()))?;
        let output_rx = self.output_rx.as_mut().unwrap();
        let data = output_rx.recv().unwrap();

        Ok(data)
    }
}
