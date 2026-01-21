#!/bin/bash

set -e

# --- Helper Functions ---
run_wasm() {
    echo "$1" | wazero run ./rs-detect-language.wasm -- "${@:2}"
}

# --- Example Functions ---

ex1_hello() {
    echo "--- Running ex1_hello ---"
    run_wasm "hello, world" --max-output-languages 1
}

ex2_invalid_dist() {
    echo "--- Running ex2_invalid_dist ---"
    if run_wasm "hello" --minimum-relative-distance 1.1; then
        echo "Expected error, but got none"
        return 1
    else
        echo "Got expected error"
    fi
}

ex3_0_output_lang() {
    echo "--- Running ex3_0_output_lang ---"
    output=$(run_wasm "hello, world" --max-output-languages 0)
    if [ -n "$output" ]; then
        echo "Expected no output, but got: $output"
        return 1
    else
        echo "Got no output as expected"
    fi
}

# --- Main Dispatcher ---

main() {
    local examples="ex1_hello ex2_invalid_dist ex3_0_output_lang"
    if [ -z "$1" ]; then
        for example in $examples; do
            "$example"
        done
    else
        "$1"
    fi
}

main "$@"