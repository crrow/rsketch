//! Safe Rust bindings for macOS Virtualization.framework
//!
//! This crate provides a safe, ergonomic interface to Apple's Virtualization.framework,
//! which allows creating and managing virtual machines on macOS 11+.

use objc2::rc::Retained;
use objc2::AnyThread;
use objc2_virtualization::{
    VZVirtualMachine, VZVirtualMachineConfiguration, VZVirtualMachineState,
};

/// A virtual machine instance.
///
/// This wraps `VZVirtualMachine` from the Virtualization.framework.
pub struct VirtualMachine {
    inner: Retained<VZVirtualMachine>,
}

impl VirtualMachine {
    /// Creates a new virtual machine with the given configuration.
    ///
    /// # Arguments
    /// * `config` - The virtual machine configuration
    ///
    /// # Returns
    /// A new `VirtualMachine` instance, or an error if creation fails.
    pub fn new(config: VirtualMachineConfiguration) -> Result<Self, Error> {
        let vm = unsafe {
            VZVirtualMachine::initWithConfiguration(
                VZVirtualMachine::alloc(),
                &config.inner,
            )
        };

        Ok(Self { inner: vm })
    }

    /// Returns the current state of the virtual machine.
    pub fn state(&self) -> MachineState {
        let state = unsafe { self.inner.state() };
        MachineState::from_vz_state(state)
    }

    /// Checks if the virtual machine can be started.
    pub fn can_start(&self) -> bool {
        unsafe { self.inner.canStart() }
    }

    /// Starts the virtual machine.
    ///
    /// # Errors
    /// Returns an error if the VM cannot be started or if startup fails.
    pub fn start(&self) -> Result<(), Error> {
        if !self.can_start() {
            return Err(Error::InvalidState);
        }

        // TODO: Implement async completion handler with block2
        // unsafe {
        //     self.inner.startWithCompletionHandler(block);
        // }

        Ok(())
    }

    /// Pauses the virtual machine.
    pub fn pause(&self) -> Result<(), Error> {
        // TODO: Implement with completion handler
        Ok(())
    }

    /// Resumes a paused virtual machine.
    pub fn resume(&self) -> Result<(), Error> {
        // TODO: Implement with completion handler
        Ok(())
    }

    /// Stops the virtual machine.
    pub fn stop(&self) -> Result<(), Error> {
        // TODO: Implement with completion handler
        Ok(())
    }
}

/// Virtual machine configuration.
///
/// This wraps `VZVirtualMachineConfiguration` from the Virtualization.framework.
pub struct VirtualMachineConfiguration {
    inner: Retained<VZVirtualMachineConfiguration>,
}

impl VirtualMachineConfiguration {
    /// Creates a new virtual machine configuration.
    pub fn new() -> Self {
        let config = unsafe { VZVirtualMachineConfiguration::new() };
        Self { inner: config }
    }

    /// Validates the configuration.
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> Result<(), Error> {
        let result = unsafe { self.inner.validateWithError() };

        match result {
            Ok(()) => Ok(()),
            Err(_err) => Err(Error::InvalidConfiguration),
        }
    }

    /// Sets the number of CPUs.
    ///
    /// # Arguments
    /// * `count` - The number of virtual CPUs
    pub fn set_cpu_count(&mut self, count: usize) {
        unsafe {
            self.inner.setCPUCount(count);
        }
    }

    /// Sets the memory size in bytes.
    ///
    /// # Arguments
    /// * `size` - Memory size in bytes
    pub fn set_memory_size(&mut self, size: u64) {
        unsafe {
            self.inner.setMemorySize(size);
        }
    }
}

impl Default for VirtualMachineConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

/// Virtual machine state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineState {
    /// The virtual machine is stopped.
    Stopped,
    /// The virtual machine is running.
    Running,
    /// The virtual machine is paused.
    Paused,
    /// The virtual machine encountered an error.
    Error,
    /// The virtual machine is starting.
    Starting,
    /// The virtual machine is pausing.
    Pausing,
    /// The virtual machine is resuming.
    Resuming,
    /// The virtual machine is stopping.
    Stopping,
    /// The virtual machine is saving.
    Saving,
    /// The virtual machine is restoring.
    Restoring,
}

impl MachineState {
    fn from_vz_state(state: VZVirtualMachineState) -> Self {
        match state {
            VZVirtualMachineState::Stopped => Self::Stopped,
            VZVirtualMachineState::Running => Self::Running,
            VZVirtualMachineState::Paused => Self::Paused,
            VZVirtualMachineState::Error => Self::Error,
            VZVirtualMachineState::Starting => Self::Starting,
            VZVirtualMachineState::Pausing => Self::Pausing,
            VZVirtualMachineState::Resuming => Self::Resuming,
            VZVirtualMachineState::Stopping => Self::Stopping,
            VZVirtualMachineState::Saving => Self::Saving,
            VZVirtualMachineState::Restoring => Self::Restoring,
            _ => Self::Error,
        }
    }
}

/// Errors that can occur when working with virtual machines.
#[derive(Debug)]
pub enum Error {
    /// The virtual machine is in an invalid state for the requested operation.
    InvalidState,
    /// The virtual machine configuration is invalid.
    InvalidConfiguration,
    /// An operation failed.
    OperationFailed,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidState => write!(f, "Virtual machine is in an invalid state"),
            Self::InvalidConfiguration => write!(f, "Virtual machine configuration is invalid"),
            Self::OperationFailed => write!(f, "Virtual machine operation failed"),
        }
    }
}

impl std::error::Error for Error {}
