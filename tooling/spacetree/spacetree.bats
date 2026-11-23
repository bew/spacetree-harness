# Test suite for the `spacetree` binary
#
# % Uses BATS testing system
# docs: https://bats-core.readthedocs.io/
# repo: https://github.com/bats-core/bats-core
#
# Run tests with: `bats $this_file [--filter foobar]`

bats_require_minimum_version 1.5.0

SCRIPT_DIR="$(dirname "$BATS_TEST_FILENAME")"
# The suite drives a binary rather than the checkout: point it at one with `SPACETREE_BIN`
# (the Nix build does), or build the default target with `cargo build` first.
SCRIPT_PATH="${SPACETREE_BIN:-$SCRIPT_DIR/target/debug/spacetree}"

# Each test gets its own spacetree root named `test-tree-root`, whose `.spacetree-repo`
# symlink points at a sibling directory playing the spacetree repo.
function setup() {
    TREE_ROOT="$BATS_TEST_TMPDIR/test-tree-root"
    REPO_DIR="$BATS_TEST_TMPDIR/test-tree-repo"
    mkdir -p "$TREE_ROOT" "$REPO_DIR"
    ln -s ../test-tree-repo "$TREE_ROOT/.spacetree-repo"
}

# ------------------------------------------------------------------------------
# Tests: defaults

@test "defaults: prints the spacetree name, root and repo as JSON" {
    cd "$TREE_ROOT"

    run -0 "$SCRIPT_PATH" root --json

    [[ "$(jq -r '.name' <<< "$output")" == "test-tree-root" ]]
    [[ "$(jq -r '.path' <<< "$output")" == "$(pwd -P)" ]]
    [[ "$(jq -r '.spacetree_repo' <<< "$output")" == "$(cd "$REPO_DIR" && pwd -P)" ]]
}

@test "defaults: resolves the root from a nested subdirectory" {
    mkdir -p "$TREE_ROOT/group/repo"
    cd "$TREE_ROOT/group/repo"

    run -0 "$SCRIPT_PATH" root --json

    [[ "$(jq -r '.name' <<< "$output")" == "test-tree-root" ]]
    [[ "$(jq -r '.path' <<< "$output")" == "$(cd "$TREE_ROOT" && pwd -P)" ]]
    [[ "$(jq -r '.spacetree_repo' <<< "$output")" == "$(cd "$REPO_DIR" && pwd -P)" ]]
}

@test "defaults: emits valid single-line JSON" {
    cd "$TREE_ROOT"

    run -0 "$SCRIPT_PATH" root --json

    run -0 jq -e 'has("name") and has("path") and has("spacetree_repo")' <<< "$output"
    [[ "$(printf '%s' "$output" | wc -l | tr -d ' ')" -eq 0 ]]
}

# ------------------------------------------------------------------------------
# Tests: usage

@test "usage: --help lists the root subcommand" {
    run -0 "$SCRIPT_PATH" --help

    [[ "$output" == *"root"* ]]
}

# ------------------------------------------------------------------------------
# Tests: error

@test "error: root without --json fails" {
    cd "$TREE_ROOT"

    run -1 --separate-stderr "$SCRIPT_PATH" root

    [[ "$stderr" == *"--json"* ]]
}

@test "error: fails when no spacetree marker is found" {
    rm -f "$TREE_ROOT/.spacetree-repo"
    cd "$TREE_ROOT"

    run -1 --separate-stderr "$SCRIPT_PATH" root --json

    [[ "$stderr" == *".spacetree-repo"* ]]
}

@test "error: fails when the marker is not a symlink" {
    rm -f "$TREE_ROOT/.spacetree-repo"
    touch "$TREE_ROOT/.spacetree-repo"
    cd "$TREE_ROOT"

    run -1 --separate-stderr "$SCRIPT_PATH" root --json

    [[ "$stderr" == *".spacetree-repo"* ]]
    [[ "$stderr" == *"not a symlink"* ]]
}

@test "error: fails when the marker target does not exist" {
    rm -f "$TREE_ROOT/.spacetree-repo"
    ln -s ../missing-repo "$TREE_ROOT/.spacetree-repo"
    cd "$TREE_ROOT"

    run -1 --separate-stderr "$SCRIPT_PATH" root --json

    [[ "$stderr" == *".spacetree-repo"* ]]
    [[ "$stderr" == *"missing-repo"* ]]
}

# ------------------------------------------------------------------------------
# Tests: edge

@test "edge: warns but resolves when the marker target is absolute" {
    rm -f "$TREE_ROOT/.spacetree-repo"
    ln -s "$(cd "$REPO_DIR" && pwd -P)" "$TREE_ROOT/.spacetree-repo"
    cd "$TREE_ROOT"

    run -0 --separate-stderr "$SCRIPT_PATH" root --json

    [[ "$(jq -r '.spacetree_repo' <<< "$output")" == "$(cd "$REPO_DIR" && pwd -P)" ]]
    [[ "$stderr" == *"WARNING"* ]]
    [[ "$stderr" == *"absolute"* ]]
}

@test "edge: prefers the nearest spacetree when markers are nested" {
    local inner="$TREE_ROOT/inner"
    local inner_repo="$TREE_ROOT/inner-repo"
    mkdir -p "$inner/sub" "$inner_repo"
    ln -s ../inner-repo "$inner/.spacetree-repo"
    cd "$inner/sub"

    run -0 "$SCRIPT_PATH" root --json

    [[ "$(jq -r '.name' <<< "$output")" == "inner" ]]
    [[ "$(jq -r '.path' <<< "$output")" == "$(cd "$inner" && pwd -P)" ]]
    [[ "$(jq -r '.spacetree_repo' <<< "$output")" == "$(cd "$inner_repo" && pwd -P)" ]]
}
