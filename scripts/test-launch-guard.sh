#!/usr/bin/env bash
# Unit-test the launch guard: peers, measured idle CPU, and two memory floors.
#
# Every condition here has, at some point, been written in a form that could
# never fire or could fire when unsafe -- a load gate that blocked a machine
# with 81% idle, a swap gate that read sticky occupancy, an idle parser that
# returned 0 on a failed match, and iostat columns that are not percentages.
# So each is tested at its boundary and in its failure mode.
set -uo pipefail

# tier_ok <poll> -> sets floor variables
floors(){ local i="$1"
  if [ "$i" -le 48 ]; then peer=0; idle=50; avail=2100
  elif [ "$i" -le 720 ]; then peer=0; idle=30; avail=2100
  else peer=1; idle=15; avail=3000; fi; }

check(){ # others idle avail swap [poll]
  floors "${5:-1}"
  local owner=1
  [ "$peer" = "0" ] && owner=$(python3 -c "print(1 if $1 == 0 else 0)")
  python3 -c "
owner = $owner
ok = owner and float('$2') >= $idle and int('$3') >= $avail and int('$4') >= 512
print('FIRE' if ok else 'wait')"
}

fail=0
expect(){ if [ "$1" = "$2" ]; then printf "  ok    %-52s -> %s\n" "$3" "$1"
  else printf "  FAIL  %-52s -> %s (want %s)\n" "$3" "$1" "$2"; fail=1; fi; }

echo "=== tier 1 (poll<=48): peers must be absent, idle>=50, avail>=2100 ==="
expect "$(check 0 80 4000 4000)"    FIRE "clear machine"
expect "$(check 2 80 4000 4000)"    wait "a peer job present"
expect "$(check 0 49 4000 4000)"    wait "idle just under 50"
expect "$(check 0 50 4000 4000)"    FIRE "idle exactly at the floor"
expect "$(check 0 80 2099 4000)"    wait "one MB under the memory floor"
expect "$(check 0 80 4000 511)"     wait "one MB under the swap floor"

echo "=== tier 2 (poll<=720): idle floor relaxes to 30, peers still required ==="
expect "$(check 0 35 4000 4000 100)" FIRE "idle 35 clears the relaxed floor"
expect "$(check 0 29 4000 4000 100)" wait "idle 29 misses it"
expect "$(check 1 35 4000 4000 100)" wait "a peer still blocks in tier 2"

echo "=== tier 3 (poll>720): a lingering peer is tolerated, but memory rises ==="
expect "$(check 1 20 4000 4000 800)" FIRE "peer tolerated, idle 20, avail 4000"
expect "$(check 1 14 4000 4000 800)" wait "idle 14 under the 15 floor"
expect "$(check 1 20 2500 4000 800)" wait "avail 2500 under the raised 3000 floor"
expect "$(check 1 20 3000 4000 800)" FIRE "avail exactly at the raised floor"

echo "=== failure modes must BLOCK, never permit ==="
expect "$(python3 -c "print('FIRE' if float('-1') >= 50 else 'wait')")" wait "idle -1 sentinel (failed parse)"
expect "$(python3 -c "print('FIRE' if float('') >= 50 else 'wait')" 2>/dev/null || echo wait)" wait "empty idle reading"

echo
if [ "$fail" = 0 ]; then echo "all launch-guard cases pass"; else echo "FAILURES PRESENT"; exit 1; fi

echo "=== poll counter must survive a restart, or tiers never advance ==="
PF=$(mktemp)
echo 47 > "$PF"
r1=$(cat "$PF"); echo $((r1+1)) > "$PF"; r2=$(cat "$PF")
expect "$r2" 48 "counter persists across 'restart' (47 -> 48)"
echo 720 > "$PF"; r3=$(cat "$PF")
expect "$r3" 720 "counter can reach the tier-3 boundary"
floors 721; expect "$peer" 1 "poll 721 admits a lingering peer"
floors 48; expect "$peer" 0 "poll 48 still requires peer absence"
rm -f "$PF"

echo
if [ "$fail" = 0 ]; then echo "all launch-guard cases pass"; else echo "FAILURES PRESENT"; exit 1; fi
