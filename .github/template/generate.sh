#!/bin/bash

# You will need to install yq >= 4.16 to use this tool.
# brew install yq

set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd "$DIR"


HEADER="""
# ================= THIS FILE IS AUTOMATICALLY GENERATED =================
#
# Please run generate.sh and commit after editing the workflow templates.
#
# ========================================================================
"""

# Generate workflow for main branch
echo "$HEADER" > ../workflows/main.yml
# shellcheck disable=SC2016
yq ea '. as $item ireduce ({}; . * $item )' template.yml main-override.yml | yq eval '... comments=""' - >> ../workflows/main.yml
echo "$HEADER" >> ../workflows/main.yml

# Generate workflow for pull requests
echo "$HEADER" > ../workflows/pull-request.yml
# shellcheck disable=SC2016
yq ea '. as $item ireduce ({}; . * $item )' template.yml pr-override.yml | yq eval '... comments=""' - >> ../workflows/pull-request.yml
echo "$HEADER" >> ../workflows/pull-request.yml

if [ "$1" == "--check" ] ; then
 if ! git diff --exit-code; then
    echo "Please run generate.sh and commit after editing the workflow templates."
    exit 1
 fi
fi