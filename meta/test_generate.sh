#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

REPO_ROOT="$(realpath "$(dirname "$0")/..")"

TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/cosmwasm-template.XXXXXXXXX")
PROJECT_NAME="test-gen"

(
  echo "Navigating to $TMP_DIR"
  cd "$TMP_DIR"

  GIT_BRANCH=$(git -C "$REPO_ROOT" branch --show-current)

  echo "Generating project from local repository (branch $GIT_BRANCH) ..."
  cargo generate --git "$REPO_ROOT" --name "$PROJECT_NAME" --branch "$GIT_BRANCH"

  (
    cd "$PROJECT_NAME"
    echo "This is what was generated"
    ls -lA

    echo "Building wasm ..."
    cargo wasm
    echo "Running tests ..."
    cargo unit-test
    cargo integration-test
    echo "Creating schema ..."
    cargo schema
  )
)
