# Capture procedure — Linux peer (ticket 001-01)

How to watch for IPv4 fragments on the peer while the macOS / Windows host
runs the don't-fragment tests (001-04, 001-09). IPv4 fragmentation is
forbidden in the runtime; the fallback path MTU is 1 280 bytes (C-06), and a
default packet is a 1 252-byte UDP payload (C-137), well under a 1 500-byte
MTU — so a clean run must show 0 fragments.

## One command: list any IPv4 fragment received from the probe host

```sh
IFACE=eth0   # the peer's interface facing the machine under test
PORT=9000    # the probe UDP port (sender default: 127.0.0.1:9000)

tcpdump -ni "$IFACE" 'ip[6:2] & 0x3fff != 0'   # lists fragments; add -w fragments.pcap to save
```

`ip[6:2] & 0x3fff != 0` matches every IPv4 fragment: the 3-bit flags field
(MF = more fragments, DF is masked off) plus the 13-bit fragment offset.
Each matching packet is printed with its source address; on an unfragmented
run the command prints nothing (0 fragments). Capture needs root or
`CAP_NET_RAW` on the peer — that is a measurement step, not a CI test
("unprivileged" applies to per-change CI, C-146).

The same filter can be scoped to the probe port by combining it with the
traffic filter:

```sh
tcpdump -ni "$IFACE" "(udp port $PORT) or (ip[6:2] & 0x3fff != 0)"
```

## Saving the results

Save the `.pcap` next to the report under `doc/design/reports/` (C-130):

```sh
tcpdump -ni "$IFACE" -w fragments.pcap 'ip[6:2] & 0x3fff != 0'
# run the don't-fragment tests, then stop tcpdump with Ctrl-C and move the
# capture with the report, e.g.:
mv fragments.pcap doc/design/reports/001-04-fragments.pcap
```

Reference the saved `.pcap` path from the report that cites it. To check the
saved capture afterwards without privileges:

```sh
tcpdump -nr doc/design/reports/001-04-fragments.pcap | wc -l   # 0 on a clean run
```
