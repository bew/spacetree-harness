# Test suite for `spacetree` script
#
# % Uses BATS testing system
# docs: https://bats-core.readthedocs.io/
# repo: https://github.com/bats-core/bats-core
#
# Run tests with: `bats $this_file [--filter foobar]`

bats_require_minimum_version 1.5.0

SCRIPT_DIR="$(dirname "$BATS_TEST_FILENAME")"
SCRIPT_PATH="$SCRIPT_DIR/spacetree"

# Each test gets its own spacetree root named `test-tree-root` with the marker file
function setup() {
    TREE_ROOT="$BATS_TEST_TMPDIR/test-tree-root"
    mkdir -p "$TREE_ROOT"
    touch "$TREE_ROOT/.spacetree-root"
}

# ------------------------------------------------------------------------------
# Tests: defaults

@test "defaults: prints the spacetree name and path as JSON" {
    cd "$TREE_ROOT"

    run -0 "$SCRIPT_PATH" root --json

    [[ "$(jq -r '.name' <<< "$output")" == "test-tree-root" ]]
    [[ "$(jq -r '.path' <<< "$output")" == "$(pwd -P)" ]]
}

@test "defaults: resolves the root from a nested subdirectory" {
    mkdir -p "$TREE_ROOT/group/repo"
    cd "$TREE_ROOT/group/repo"

    run -0 "$SCRIPT_PATH" root --json

    [[ "$(jq -r '.name' <<< "$output")" == "test-tree-root" ]]
    [[ "$(jq -r '.path' <<< "$output")" == "$(cd "$TREE_ROOT" && pwd -P)" ]]
}

@test "defaults: emits valid single-line JSON" {
    cd "$TREE_ROOT"

    run -0 "$SCRIPT_PATH" root --json

    run -0 jq -e 'has("name") and has("path")' <<< "$output"
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
    rm -f "$TREE_ROOT/.spacetree-root"
    cd "$TREE_ROOT"

    run -1 --separate-stderr "$SCRIPT_PATH" root --json

    [[ "$stderr" == *".spacetree-root"* ]]
}

# ------------------------------------------------------------------------------
# Tests: edge

@test "edge: prefers the nearest spacetree when markers are nested" {
    local inner="$TREE_ROOT/inner"
    mkdir -p "$inner/sub"
    touch "$inner/.spacetree-root"
    cd "$inner/sub"

    run -0 "$SCRIPT_PATH" root --json

    [[ "$(jq -r '.name' <<< "$output")" == "inner" ]]
    [[ "$(jq -r '.path' <<< "$output")" == "$(cd "$inner" && pwd -P)" ]]
}
