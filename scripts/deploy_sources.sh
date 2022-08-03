#!/bin/bash

# We keep this script just for historical purposes. Might delete it.
# This uploads the api as source. Can be useful if cross compiling doesn't work.

# if needed, call reset_dirs.sh on the remote to create the directory structure.

######################
# variables
######################

# expects these directories to exist
API_CONTAINER_DIR="/usr/capi/apicontainer"
TEAL_DIR="$API_CONTAINER_DIR/teal"
API_DIR="$API_CONTAINER_DIR/api"

REMOTE_USER="root"

LOCAL_TEAL_DIR="/Users/ivanschuetz/dev/repo/github/capi/teal/teal_template/"
LOCAL_API_DIR="/Users/ivanschuetz/dev/repo/github/capi/api"

echo $LOCAL_TEAL_DIR

######################
# deploy
######################

# smart contracts
rsync -r --exclude='.git' --exclude='target' $LOCAL_TEAL_DIR/* $REMOTE_USER@143.244.177.249:$TEAL_DIR
# GLOBIGNORE='*.git:mbase/.git:target' scp -r $LOCAL_TEAL_DIR/* $REMOTE_USER@143.244.177.249:$TEAL_DIR # couldn't get globignore working

# api (source - we'll start it via cargo run)
rsync -r --exclude='.git' --exclude='target' $LOCAL_API_DIR/* $REMOTE_USER@143.244.177.249:$API_DIR
# GLOBIGNORE='*.git:target' scp -r $LOCAL_API_DIR/* $REMOTE_USER@143.244.177.249:$API_DIR # couldn't get globignore working

# note: ensure that there are no local dependencies in cargo toml (other than to the directories we're uploading), e.g algonaut
echo "Finished deploying. To start api, SSH to remote, cd to api dir: $API_DIR, stop if already running, and run \`cargo run\`.".
