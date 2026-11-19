#!/usr/bin/env bash
# Extract the exact packed S2 rows from a separately pinned and validated D11.
set -euo pipefail
artifact=${1:?usage: extract-rows.sh model.d11}
dd=dd
if command -v gdd > /dev/null; then dd=gdd; fi
[[ $(head -c 8 "$artifact") == UORLUT01 ]]
# Decode the header length using the wire byte order, independent of the host.
read -r -a length_bytes <<< "$(od -An -j 8 -N 8 -tu1 "$artifact")"
[[ ${#length_bytes[@]} == 8 ]]
(( length_bytes[4] == 0 && length_bytes[5] == 0 &&
   length_bytes[6] == 0 && length_bytes[7] == 0 ))
length=$((length_bytes[0] | length_bytes[1] << 8 |
          length_bytes[2] << 16 | length_bytes[3] << 24))
(( length > 0 && length <= 67108864 ))
header=$("$dd" if="$artifact" iflag=skip_bytes,count_bytes skip=16 count="$length" status=none)
[[ ${#header} == "$length" ]]
section=$(jq -ce '
  select(.shape.vocabulary == 4096)
  | [.tables[] | select(.name == "discrete.trunk.s2.top4.log2.v1")]
  | select(length == 1)
  | first
  | select(.kind == "u32" and .len * 4 == .span.bytes)
  | .span
  | select(.offset >= 0 and .offset % 64 == 0 and .bytes >= 96 and (.bytes - 96) % 20 == 0)
' <<< "$header")
offset=$(jq -r '.offset' <<< "$section")
bytes=$(jq -r '.bytes' <<< "$section")
start=$(((16 + length + 63) / 64 * 64 + offset + 96))
file_bytes=$(wc -c < "$artifact")
(( start + bytes - 96 <= file_bytes ))
# GNU dd copies this range without byte-sized syscalls.
"$dd" if="$artifact" iflag=skip_bytes,count_bytes skip="$start" count="$((bytes - 96))" status=none
