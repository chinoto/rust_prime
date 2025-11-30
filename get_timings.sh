#!/usr/bin/env bash

limit="${1:-1e7}"

binaries=(
    prime
    prime_buffer
    prime_buffer_mt_sc
    prime_buffer_mt_sc_atomic
    prime_buffer_mt_sc_cow
    prime_buffer_mt_ptc
    prime_mt_range_map
    prime_mt_rayon
    prime_mt_orx
    prime_mt_chili
)

echo "\
| (Kernel + User) / Real Seconds | CPU % | Binary |
| -----------------------------: | ----: | :----- |"

for bin in "${binaries[@]}"; do
    # If stdout is a terminal, show the binary name immediately.
    [ -t 1 ] && echo -n "${bin}"
    timing=$((/usr/bin/time -f '(%S + %U) / %e | %P' "./target/release/${bin}" "${limit}" > /dev/null) 2>&1)
    # Set `clear` to a carriage return if stdout is a terminal, otherwise an empty string.
    clear=''
    if [ -t 1 ]; then
        clear=$(echo -e '\r')
    fi
    echo "${clear}| ${timing} | ${bin} |"
done
