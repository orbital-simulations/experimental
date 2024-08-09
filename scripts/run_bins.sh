#!/bin/bash

# Get the list of all examples using cargo
bins_list=$(cargo run --bin 2>&1 | grep "Available binaries:" -A 1000 | grep -v "Available binaries:")

if [ -z "$bins_list" ]; then
    echo "No binaries found."
    exit 1
fi

for bin_name in $bins_list; do
    echo "Running binary: $bin_name"

    cargo run --bin "$bin_name"

    # Check if the last command was successful
    if [ $? -ne 0 ]; then
        echo "Binary $bin_name failed."
    else
        echo "Binary $bin_name ran successfully."
    fi
    echo "--------------------------------"
done
