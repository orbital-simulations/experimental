#!/bin/bash

# Get the list of all examples using cargo
example_list=$(cargo run --example 2>&1 | grep "Available examples:" -A 1000 | grep -v "Available examples:")

if [ -z "$example_list" ]; then
    echo "No examples found."
    exit 1
fi

for example_name in $example_list; do
    echo "Running example: $example_name"

    cargo run --example "$example_name"

    # Check if the last command was successful
    if [ $? -ne 0 ]; then
        echo "Example $example_name failed."
    else
        echo "Example $example_name ran successfully."
    fi
    echo "--------------------------------"
done
