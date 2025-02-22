#!/usr/bin/env bash
cd "$(dirname "$0")/../" || exit 1
set -euro pipefail

npx --yes wrangler dev --port 5000 --host '0.0.0.0'
