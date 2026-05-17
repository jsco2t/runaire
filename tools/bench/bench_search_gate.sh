#!/usr/bin/env bash
# bench_search_gate.sh — NFR-002 gate.
#
# Runs the entry-search benchmark and exits non-zero if either the
# substring or wildcard max-sample latency exceeds the budget.
#
# Budget:
#   - Local (M1 / x86_64 dev box): 50ms (tight).
#   - CI cloud runners: 75ms (50% headroom for noisy neighbours).
# Override with `BUDGET_MS=<n>` in the env. CI sets BUDGET_MS=75.
#
# The benchmark harness (`crates/runaire-core/benches/bench_search.rs`)
# emits plain `KEY=VALUE` lines:
#   bench_search_5k_max_ms=3.27
#   bench_search_5k_wildcard_max_ms=4.48
# We assert max ≤ budget for each. Using "max" of 30 samples as a
# conservative proxy for the PRD's "p99" (with N=30, the 99th-percentile
# point is the worst sample); documented in the engineering plan.

set -euo pipefail

BUDGET_MS="${BUDGET_MS:-50}"

cd "$(dirname "$0")/../.."

output=$(cargo bench -p runaire-core --bench bench_search --offline --locked 2>&1 | tail -10)

extract_max() {
    local prefix="$1"
    echo "$output" | awk -F= -v key="${prefix}_max_ms" '$1 == key {print $2}' | tail -1
}

substring_max=$(extract_max "bench_search_5k")
wildcard_max=$(extract_max "bench_search_5k_wildcard")

if [ -z "$substring_max" ] || [ -z "$wildcard_max" ]; then
    echo "bench_search_gate: failed to parse benchmark output" >&2
    echo "$output" >&2
    exit 2
fi

# Float comparison via awk — bash arithmetic only handles integers.
check() {
    local label="$1"
    local value="$2"
    awk -v v="$value" -v b="$BUDGET_MS" -v label="$label" '
        BEGIN {
            if (v > b) {
                printf "FAIL: %s max %.2fms > budget %sms\n", label, v, b > "/dev/stderr"
                exit 1
            }
            printf "OK:   %s max %.2fms (budget %sms)\n", label, v, b
            exit 0
        }
    '
}

# Run both checks; only exit non-zero at the end so the developer sees
# both numbers in the failure log.
status=0
check "substring" "$substring_max" || status=$?
check "wildcard " "$wildcard_max"  || status=$?
exit "$status"
