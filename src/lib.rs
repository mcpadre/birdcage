//! Birdcage sandbox.
//!
//! This crate provides a cross-platform API for an embedded sandbox for macOS
//! and Linux.
//!
//! # Example
//!
//! ```rust
//! use std::collections::HashMap;
//! use birdcage::process::Command;
//! use birdcage::{Birdcage, Exception, Sandbox};
//!
//! // Create a new sandbox
//! let mut sandbox = Birdcage::new();
//!
//! // Allow access to our test executable
//! sandbox.add_exception(Exception::ExecuteAndRead("/bin/cat".into())).unwrap();
//! let _ = sandbox.add_exception(Exception::ExecuteAndRead("/lib64".into()));
//! let _ = sandbox.add_exception(Exception::ExecuteAndRead("/lib".into()));
//!
//! // Set custom environment variables (replaces all existing environment)
//! let mut custom_env = HashMap::new();
//! custom_env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
//! custom_env.insert("USER".to_string(), "sandbox_user".to_string());
//! sandbox.add_exception(Exception::CustomEnvironment(custom_env)).unwrap();
//!
//! // Initialize the sandbox; by default everything is prohibited
//! let mut command = Command::new("/bin/cat");
//! command.arg("./Cargo.toml");
//! let mut child = sandbox.spawn(command).unwrap();
//!
//! // Wait for the command to complete
//! let status = child.wait().unwrap();
//! assert!(!status.success()); // Should fail due to file access restrictions
//! ```

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use crate::error::Result;
#[cfg(target_os = "linux")]
use crate::linux::LinuxSandbox;
#[cfg(target_os = "macos")]
use crate::macos::MacSandbox;
use crate::process::{Child, Command};

pub mod error;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
pub mod process;

/// Default platform sandbox.
///
/// This type will automatically pick the default sandbox for each available
/// platform.
#[cfg(target_os = "linux")]
pub type Birdcage = LinuxSandbox;

/// Default platform sandbox.
///
/// This type will automatically pick the default sandbox for each available
/// platform.
#[cfg(target_os = "macos")]
pub type Birdcage = MacSandbox;

pub trait Sandbox: Sized {
    /// Setup the sandboxing environment.
    fn new() -> Self;

    /// Add a new exception to the sandbox.
    ///
    /// Exceptions added for symlinks will also automatically apply to the
    /// symlink's target.
    fn add_exception(&mut self, exception: Exception) -> Result<&mut Self>;

    /// Setup sandbox and spawn a new process.
    ///
    /// This will setup the sandbox in the **CURRENT** process, before launching
    /// the sandboxee. Since most of the restrictions will also be applied to
    /// the calling process, it is recommended to create a separate process
    /// before calling this method. The calling process is **NOT** fully
    /// sandboxed.
    ///
    /// # Errors
    ///
    /// Sandboxing will fail if the calling process is not single-threaded.
    ///
    /// After failure, the calling process might still be affected by partial
    /// sandboxing restrictions.
    fn spawn(self, sandboxee: Command) -> Result<Child>;
}

/// Sandboxing exception rule.
///
/// An exception excludes certain resources from the sandbox, allowing sandboxed
/// applications to still access these resources.
#[derive(Debug, Clone)]
pub enum Exception {
    /// Allow read access to the path and anything beneath it.
    Read(PathBuf),

    /// Allow writing and reading the path and anything beneath it.
    WriteAndRead(PathBuf),

    /// Allow executing and reading the path and anything beneath it.
    ///
    /// This is grouped with reading as a convenience, since execution will
    /// always also require read access.
    ExecuteAndRead(PathBuf),

    /// Allow reading an environment variable.
    Environment(String),

    /// Allow reading **all** environment variables.
    FullEnvironment,

    /// Replace all environment variables with a custom map.
    ///
    /// This completely replaces the environment with the provided variables.
    /// If this exception is set, `Environment` and `FullEnvironment` exceptions
    /// are ignored. If multiple `CustomEnvironment` exceptions are added, the
    /// last one takes precedence.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use birdcage::{Birdcage, Exception, Sandbox};
    ///
    /// let mut custom_env = HashMap::new();
    /// custom_env.insert("PATH".to_string(), "/usr/bin".to_string());
    /// custom_env.insert("HOME".to_string(), "/tmp".to_string());
    ///
    /// let mut sandbox = Birdcage::new();
    /// sandbox.add_exception(Exception::CustomEnvironment(custom_env)).unwrap();
    /// ```
    CustomEnvironment(HashMap<String, String>),

    /// Allow networking.
    Networking,
}

/// Restrict access to environment variables.
pub(crate) fn restrict_env_variables(exceptions: &[String]) {
    restrict_env_variables_with_custom(exceptions, None);
}

/// Restrict access to environment variables, optionally replacing with custom map.
///
/// If `custom_env` is provided, all existing environment variables are cleared
/// and replaced with the variables from the map. Otherwise, variables not in
/// the `exceptions` list are removed.
pub(crate) fn restrict_env_variables_with_custom(
    exceptions: &[String],
    custom_env: Option<&HashMap<String, String>>,
) {
    match custom_env {
        Some(env_map) => {
            // Clear all existing environment variables
            for (key, _) in env::vars() {
                env::remove_var(key);
            }

            // Set custom environment variables
            for (key, value) in env_map {
                env::set_var(key, value);
            }
        }
        None => {
            // Invalid unicode will cause `env::vars()` to panic, so we don't have to worry
            // about them getting ignored.
            for (key, _) in env::vars().filter(|(key, _)| !exceptions.contains(key)) {
                env::remove_var(key);
            }
        }
    }
}
