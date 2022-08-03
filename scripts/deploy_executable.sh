#!/bin/bash

# deploys the release api binary and dependencies (teal, configuration files)

# if needed, call reset_dirs.sh on the remote to create the directory structure.

######################
# variables
######################

# expects these directories to exist
API_CONTAINER_DIR="/usr/capi/apicontainer"
TEAL_DIR="$API_CONTAINER_DIR/teal/teal_template"
API_DIR="$API_CONTAINER_DIR/api"

REMOTE_USER="root"

LOCAL_TEAL_DIR="/Users/ivanschuetz/dev/repo/github/capi/teal/teal_template/"
LOCAL_API_DIR="/Users/ivanschuetz/dev/repo/github/capi/api"
LOCAL_API_BINARY="./target/x86_64-unknown-linux-gnu/release/api"
# debug version
# LOCAL_API_BINARY="./target/x86_64-unknown-linux-gnu/debug/api"

######################
# deploy
######################

# smart contracts
echo "📃 Deploying teal: $LOCAL_TEAL_DIR"
rsync -r --exclude='.git' --exclude='target' $LOCAL_TEAL_DIR/* $REMOTE_USER@143.244.177.249:$TEAL_DIR

# api executable
echo "💾 Deploying api executable: $LOCAL_API_BINARY"
# release version
rsync $LOCAL_API_BINARY $REMOTE_USER@143.244.177.249:$API_DIR
# debug version
# rsync ./target/x86_64-unknown-linux-gnu/debug/api $REMOTE_USER@143.244.177.249:$API_DIR

# files
echo "🗃️ Deploying configuration files.."
rsync ./log_config.yml root@143.244.177.249:$API_DIR
rsync ./.env root@143.244.177.249:$API_DIR
rsync ./Rocket.toml root@143.244.177.249:$API_DIR

# note: ensure that there are no local dependencies in cargo toml (other than to the directories we're uploading), e.g algonaut
echo "🎉 Finished deploying. To start api, SSH to remote, cd to api dir: $API_DIR, and run the executable: ./api"
