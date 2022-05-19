#!/bin/bash
for target in $(rustc --print target-list); do
    echo -n "$target "
    rustc -Z unstable-options --print target-spec-json --target $target | jq -r '"\(."max-atomic-width" | if(. == null) then 32 else . end) \(."atomic-cas" | if(. == null) then true else . end)"'
done