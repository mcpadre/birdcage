//! Custom environment variable integration test.

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use birdcage::{Birdcage, Exception, Sandbox};

use crate::TestSetup;

pub fn setup(_tempdir: PathBuf) -> TestSetup {
    // Setup initial environment
    env::set_var("EXISTING_VAR", "should_be_removed");
    env::set_var("ANOTHER_EXISTING", "also_removed");

    // Create custom environment
    let mut custom_env = HashMap::new();
    custom_env.insert("CUSTOM_VAR".to_string(), "custom_value".to_string());
    custom_env.insert("ANOTHER_CUSTOM".to_string(), "another_value".to_string());
    custom_env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());

    // Activate our sandbox.
    let mut sandbox = Birdcage::new();
    sandbox.add_exception(Exception::CustomEnvironment(custom_env)).unwrap();

    TestSetup { sandbox, data: String::new() }
}

pub fn validate(_data: String) {
    // Should only have custom environment variables
    let env_vars: HashMap<String, String> = env::vars().collect();
    
    // Check that we have exactly the expected variables
    assert_eq!(env_vars.len(), 3, "Expected exactly 3 environment variables, got: {:?}", env_vars);
    
    // Check specific values
    assert_eq!(env_vars.get("CUSTOM_VAR"), Some(&"custom_value".to_string()));
    assert_eq!(env_vars.get("ANOTHER_CUSTOM"), Some(&"another_value".to_string()));
    assert_eq!(env_vars.get("PATH"), Some(&"/usr/bin:/bin".to_string()));
    
    // Check that original variables are gone
    assert!(env_vars.get("EXISTING_VAR").is_none(), "EXISTING_VAR should have been removed");
    assert!(env_vars.get("ANOTHER_EXISTING").is_none(), "ANOTHER_EXISTING should have been removed");
}