# Birdcage

<div align="center">

[![GitHub](https://img.shields.io/github/license/phylum-dev/birdcage)][license]
[![GitHub issues](https://img.shields.io/github/issues/phylum-dev/birdcage)][issues]
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)][CoC]
[![Crate](https://img.shields.io/crates/v/birdcage)](https://crates.io/crates/birdcage)
[![Documentation](https://docs.rs/birdcage/badge.svg)](https://docs.rs/birdcage)

[license]: https://github.com/phylum-dev/birdcage/blob/main/LICENSE
[issues]: https://github.com/phylum-dev/birdcage/issues
[CoC]: https://github.com/phylum-dev/birdcage/blob/main/CODE_OF_CONDUCT.md

[![Birdcage logo](./assets/Birdcage.png)][protection]

</div>

## About

Birdcage is a cross-platform embeddable sandboxing library allowing restrictions
to Filesystem and Network operations using native operating system APIs.

Birdcage was originally developed for use by the [Phylum CLI] as an extra layer
of [protection] against potentially malicious dependencies (see the [blog post]
for details). To better protect yourself from these security risks, [sign up
now]!

[phylum cli]: https://github.com/phylum-dev/cli
[protection]: https://www.phylum.io/defend-developers
[blog post]: https://blog.phylum.io/sandboxing-package-installations-arms-developers-with-defense-against-open-source-attacks-and-unintended-consequences/
[sign up now]: https://www.phylum.io/

Birdcage focuses **only** on Filesystem and Network operations. It **is not** a
complete sandbox preventing all side-effects or permanent damage. Applications
can still execute most system calls, which is especially dangerous when
execution is performed as root. Birdcage should be combined with other security
mechanisms, especially if you are executing known-malicious code.

## Example

An example for using Birdcage's API can be found in `./examples/sandbox`, which
runs an application with CLI-configurable restrictions applied.

Trying to run without any exceptions will produce an error:

```bash
$ cargo run --example sandbox -- echo "Hello, Sandbox\!"
Error: Os { code: 13, kind: PermissionDenied, message: "Permission denied" }
```

Running the same command with explicit permissions allows execution:

```bash
$ cargo run --example sandbox -- -e /usr/bin/echo -e /usr/lib echo "Hello, Sandbox\!"
Hello, Sandbox!
```

### Basic usage

```rust
use std::collections::HashMap;
use birdcage::{Birdcage, Exception, Sandbox};
use birdcage::process::Command;

// Create a new sandbox
let mut sandbox = Birdcage::new();

// Allow access to read a file
sandbox.add_exception(Exception::Read("/etc/hosts".into()))?;

// Allow networking
sandbox.add_exception(Exception::Networking)?;

// Set custom environment variables (replaces all existing environment)
let mut custom_env = HashMap::new();
custom_env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
custom_env.insert("HOME".to_string(), "/tmp".to_string());
sandbox.add_exception(Exception::CustomEnvironment(custom_env))?;

// Spawn a sandboxed process
let mut command = Command::new("/usr/bin/whoami");
let child = sandbox.spawn(command)?;
```

Check out `cargo run --example sandbox -- --help` for more information on how to
use the example.

## Supported Platforms

 - Linux via [namespaces]
 - macOS via `sandbox_init()` (aka Seatbelt)

[namespaces]: https://man7.org/linux/man-pages/man7/namespaces.7.html
