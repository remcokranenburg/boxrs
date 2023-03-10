#!/bin/bash

set -eu

git config user.name github-actions
git config user.email github-actions@github.com

rm -r coverage/ || true
cp -r ../main/target/debug/coverage/ coverage/
git add .
git commit -m "Update coverage report for $GITHUB_SHA"
git push