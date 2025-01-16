use anyhow::Result;

pub trait Sandbox {
    // This function is used to setup network or any other operation that needs to be performed 
    // ahead of the creation of the sandbox
    fn presetup(&mut self) -> Result<()>;
    // Start the sandbox
    fn start(&mut self) -> Result<()>;
    // Kills the sandbox
    fn kill(&mut self) -> Result<()>;
    // Clean up the sandbox
    fn cleanup(&mut self) -> Result<()>;

    // Properties of the sandbox
    fn get_target_ip(&self) -> String;
    fn get_target_port(&self) -> u16;
    fn get_name(&self) -> String;
}