#!/bin/bash
for file in ramdisk/findings-dir/crashes/id*; do
    echo "checking ${file}"
    ./fuzz-bin "$file"
    if [[ $? -ne 0 ]]; then
        echo "FAIL: case ${file}"
    fi
done
