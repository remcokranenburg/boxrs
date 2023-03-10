#!/bin/bash

set -eu

cp -r target/debug/coverage/ /tmp/coverage/
git checkout artifacts
rm -r coverage/
cp -r /tmp/coverage/ coverage/
git add . && git commit -m "Update coverage report for $GITHUB_SHA"
git push