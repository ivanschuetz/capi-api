#!/bin/bash

# to be executed on the remote server
# removes the api and teal directories and its contents and creates the directory structure again
# needed if for some reason it's needed to "nuke"

cd /usr/capi

rm -rf apicontainer

cd /usr/capi/
mkdir apicontainer
cd apicontainer
# note that the teal path in the api is hardcoded: it's the same for local and remote environment
# meaning, these paths shouldn't be changed (without changing the code)
mkdir teal
mkdir teal/teal_template
mkdir api
