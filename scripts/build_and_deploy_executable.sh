#!/bin/bash

# This script has to be run from the project's root dir.
# If needed, call reset_dirs.sh on the remote to create the directory structure.

echo "🔨 Start building executable. This can take a while.\n"

# Note: target architecture defined in Cross.toml (see https://github.com/cross-rs/cross/wiki/Configuration#buildenv)
# If run for the first time, it might also be needed to add it to rustup: `rustup target add aarch64-unknown-linux-gnu`
# Note also that debug/release configuration loads the corresponding settings in Rocket.toml (e.g. ip address). This is not cross specific.

# Debug version for quicker builds. If using this, the path to the binary in deploy_executable.sh has to be adjusted.
# cross build -vv --target x86_64-unknown-linux-gnu

# Release version
cross build -vv --target x86_64-unknown-linux-gnu --release

echo "🛫 Finished building executable. Start deployment.."

sh ./scripts/deploy_executable.sh
