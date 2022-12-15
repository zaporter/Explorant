#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
docker run -v "$SCRIPT_DIR/shared:/shared" explorant/synoptic $1
