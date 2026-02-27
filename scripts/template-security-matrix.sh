#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(mktemp -d /tmp/kaido_security_matrix_XXXXXX)}"
TEMPLATES=(mint vesting escrow treasury marketplace staking oracle referral dex lending governance streaming custom)

echo "workspace=$ROOT"

run_mode() {
  local mode="$1"
  echo "mode=$mode"

  for t in "${TEMPLATES[@]}"; do
    local out="$ROOT/$mode/$t"
    mkdir -p "$out"

    local args=(
      cargo run -q -p kaido -- generate
      --template "$t"
      --namespace audit
      --project-name "${t}_proj"
      -o "$out"
      --skip-verify
    )

    if [[ "$t" == "custom" ]]; then
      args+=(
        --purpose spend
        --features signature-auth,datum-continuity,value-preservation,reference-safety
        --datum "admin:ByteArray,balance:Int"
        --redeemer "Deposit(amount:Int),Withdraw(amount:Int)"
      )
    fi

    if ! "${args[@]}" >/tmp/kaido_matrix_gen_"$mode"_"$t".log 2>&1; then
      echo "$mode|$t|generate=FAIL"
      continue
    fi

    if [[ "$mode" == "without_profile" ]]; then
      rm -f "$out/.aikido.toml"
    fi

    if ! (cd "$out" && aiken check >/tmp/kaido_matrix_check_"$mode"_"$t".log 2>&1); then
      echo "$mode|$t|aiken_check=FAIL"
      continue
    fi

    local scan
    scan=$(cd "$out" && aikido . --format json --quiet 2>/tmp/kaido_matrix_aikido_"$mode"_"$t".err || true)

    local high_critical
    high_critical=$(printf '%s' "$scan" | jq '[.findings[] | select((.severity|ascii_downcase)=="high" or (.severity|ascii_downcase)=="critical")] | length')

    local detectors
    detectors=$(printf '%s' "$scan" | jq -r '[.findings[] | select((.severity|ascii_downcase)=="high" or (.severity|ascii_downcase)=="critical") | .detector] | unique | join(",")')

    echo "$mode|$t|high_critical=$high_critical|detectors=$detectors"
  done
}

run_mode "with_profile"
run_mode "without_profile"
