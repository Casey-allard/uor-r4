#!/usr/bin/env bash
# Unit-test the monitor's trigger condition against synthetic states, so a
# never-firing condition is caught here rather than discovered after 8 hours.
set -uo pipefail
check(){ # <others> <load> -> prints FIRE or wait
  local others="$1" load="$2"
  local quiet; quiet=$(python3 -c "print(1 if float('$load') < 4.0 else 0)" 2>/dev/null || echo 0)
  if [ "$others" = "0" ] && [ "$quiet" = "1" ]; then echo FIRE; else echo "wait"; fi
}
printf "  others=2 load=10.85 -> %s (expect wait)\n" "$(check 2 10.85)"
printf "  others=2 load=2.0   -> %s (expect wait: another job)\n" "$(check 2 2.0)"
printf "  others=0 load=12.0  -> %s (expect wait: load high)\n" "$(check 0 12.0)"
printf "  others=0 load=1.5   -> %s (expect FIRE)\n" "$(check 0 1.5)"
printf "  others=0 load=3.99  -> %s (expect FIRE)\n" "$(check 0 3.99)"
printf "  others=0 load=4.01  -> %s (expect wait)\n" "$(check 0 4.01)"
# tiered bar: strict for the first STRICT_POLLS, relaxed after
tier(){ local i="$1" load="$2"
  local limit; if [ "$i" -le 48 ]; then limit=4.0; else limit=8.0; fi
  python3 -c "print('FIRE' if float('$load') < $limit else 'wait')"; }
printf "  poll 10  load=5.0 -> %s (expect wait: still strict)\n" "$(tier 10 5.0)"
printf "  poll 49  load=5.0 -> %s (expect FIRE: relaxed tier)\n" "$(tier 49 5.0)"
printf "  poll 100 load=9.0 -> %s (expect wait: 9 >= 8)\n" "$(tier 100 9.0)"
printf "  poll 100 load=7.9 -> %s (expect FIRE)\n" "$(tier 100 7.9)"
# and the live state, through the same expression
others=$(pgrep -f geometric-stack | wc -l | tr -d ' ')
load=$(uptime | sed -E 's/.*averages: ([0-9.]+).*/\1/')
printf "  LIVE others=%s load=%s -> %s\n" "$others" "$load" "$(check "$others" "$load")"
