# Changelog (moq-dev fork)

Releases of `moq-noq`, `moq-noq-proto`, `moq-noq-udp`, and `web-transport-moq`.
All four share one version. [CHANGELOG.md](CHANGELOG.md) is upstream's and
covers the parent. Each entry names the parent commit it carries and every
change the fork carries on top, with its upstream status, so an advisory
against the parent can be checked against a release.

## 2.0.3

Parent: n0-computer/noq [`1a26a8b0`](https://github.com/n0-computer/noq/commit/1a26a8b064d21e316fe6769f068617975bd8a27b), unchanged since 1.3.0.

- [#32](https://github.com/moq-dev/noq/pull/32) reset the PTO backoff when Initial or Handshake keys are discarded (RFC 9002 A.11), so a client's Initial backoff no longer delays the probe for a lost Finished and the server no longer idles out first. Also in 1.3.5 ([#33](https://github.com/moq-dev/noq/pull/33)). Upstream quinn and n0-computer/noq have the same gap; not offered upstream yet.

No API change.

## 2.0.2

Parent: n0-computer/noq [`1a26a8b0`](https://github.com/n0-computer/noq/commit/1a26a8b064d21e316fe6769f068617975bd8a27b), unchanged since 1.3.0.

- [#28](https://github.com/moq-dev/noq/pull/28) `web-transport-moq` reports a max datagram size of 0 instead of panicking when the peer did not negotiate datagrams. Also in 1.3.4 ([#29](https://github.com/moq-dev/noq/pull/29)). Fork-only.

No API change.

## 2.0.1

Parent: n0-computer/noq [`1a26a8b0`](https://github.com/n0-computer/noq/commit/1a26a8b064d21e316fe6769f068617975bd8a27b), unchanged since 1.3.0.

- [#25](https://github.com/moq-dev/noq/pull/25) bound the stream and CRYPTO reassembly buffers at 1024 chunks after compaction, closing the connection with `INTERNAL_ERROR` past it ([RUSTSEC-2026-0185](https://rustsec.org/advisories/RUSTSEC-2026-0185.html)). Carries [quinn-rs/quinn#2694](https://github.com/quinn-rs/quinn/pull/2694), [#2789](https://github.com/quinn-rs/quinn/pull/2789), and [#2814](https://github.com/quinn-rs/quinn/pull/2814), matching quinn-proto 0.11.18, plus a fix so compacting an unordered stream keeps its unread data (upstream noq starts compaction at `bytes_read`, which the new count trigger made reachable). Also in 1.3.3 ([#26](https://github.com/moq-dev/noq/pull/26)). Upstream: [n0-computer/noq#828](https://github.com/n0-computer/noq/pull/828) (open).
- [#19](https://github.com/moq-dev/noq/pull/19) report a stream reset before its WebTransport header as the reset (`WebTransportError::ReadError`), logged at debug, instead of `UnknownSession` at WARN. `UnknownSession` now means only a missing or mismatched session ID. Also in 1.3.3 ([#20](https://github.com/moq-dev/noq/pull/20)). Fork-only.
- [#23](https://github.com/moq-dev/noq/pull/23) keep a `web-transport-moq` session's HTTP/3 control and QPACK streams open until its close capsule is delivered, so browsers see the close code and reason instead of "Connection lost.". Also in 1.3.3 ([#24](https://github.com/moq-dev/noq/pull/24)). Fork-only.
- [#27](https://github.com/moq-dev/noq/pull/27) send a raw QUIC session's (`Session::raw`) stream reset and stop codes as is, instead of mapping them into the HTTP/3 WebTransport range, and read a peer's code as is. Not in 1.3.3. Fork-only.

No API change. Wire:

- A peer that exceeds the reassembly cap is now closed.
- Raw QUIC stream codes are compatible in one direction only: 2.0.1 still reads an older raw peer's mapped codes, but an older raw peer reads 2.0.1's codes as `InvalidReset` / `InvalidStopped`. HTTP/3 sessions are unchanged.

## 2.0.0

Parent: n0-computer/noq [`1a26a8b0`](https://github.com/n0-computer/noq/commit/1a26a8b064d21e316fe6769f068617975bd8a27b), unchanged since 1.3.0.

Carries everything in 1.3.2, plus:

- [#13](https://github.com/moq-dev/noq/pull/13) implement `web-transport-trait` 0.5 in `web-transport-moq`. Fork-only.

### Breaking

- `web-transport-moq` implements `web-transport-trait` 0.5 instead of 0.4.
- `web-transport-moq` no longer re-exports `web-transport-trait` as `generic`; depend on `web-transport-trait` directly.

`moq-noq`, `moq-noq-proto`, and `moq-noq-udp` are unchanged from 1.3.2, bumped only because all four share one version.

## 1.3.2

Parent: n0-computer/noq [`1a26a8b0`](https://github.com/n0-computer/noq/commit/1a26a8b064d21e316fe6769f068617975bd8a27b), unchanged since 1.3.0.

- [#11](https://github.com/moq-dev/noq/pull/11) report a raw QUIC peer's close code from `web-transport-moq` sessions, instead of none. Fork-only.
- [#12](https://github.com/moq-dev/noq/pull/12) make BBR respond to classic ECN: CE exits Startup, stops a bandwidth probe, or lowers the short-term model, once per recovery episode, and no longer counts as a loss. Not offered upstream yet; it builds on the `PacketId` series.

No API change.

## 1.3.1

Parent: n0-computer/noq [`1a26a8b0`](https://github.com/n0-computer/noq/commit/1a26a8b064d21e316fe6769f068617975bd8a27b), unchanged since 1.3.0.

### BBR correctness

- [#3](https://github.com/moq-dev/noq/pull/3) identify BBR packets by space and number, so a Handshake ACK no longer samples an Initial packet with the same number.
- [#4](https://github.com/moq-dev/noq/pull/4) fold each ACK into BBR's model once, after the whole ACK completes.
- [#5](https://github.com/moq-dev/noq/pull/5) tell the controller of application starvation before the next send, not at the next ACK.
- [#6](https://github.com/moq-dev/noq/pull/6) finish bandwidth-probe feedback once: a finished probe ages the max-bw window once and stops classifying losses as probe feedback.
- [#7](https://github.com/moq-dev/noq/pull/7) recalibrate startup pacing from the first measured RTT instead of the 1ms placeholder.
- [#8](https://github.com/moq-dev/noq/pull/8) mark packets sent during ProbeRTT application-limited, so its reduced rate cannot lower the bandwidth model.
- [#9](https://github.com/moq-dev/noq/pull/9) keep BBR's undo snapshot for the whole recovery episode, so a spurious loss restores the model, window, and probe.

### API

Additive; existing `Controller` implementations compile unchanged.

- `congestion::PacketId` and `congestion::Space` identify a packet by space and number.
- `Controller` gains `on_packet_space_sent`, `on_packet_space_acked`, `on_packet_space_lost`, and `on_congestion_event_space`. Each defaults to the number-only callback it replaces, which is now deprecated.
- `Controller::on_congestion_event` has a default body, so it is no longer required.
- `Controller::on_app_limited(in_flight)` reports a transmit poll with nothing to send.

### Upstream status

- #7 carries [n0-computer/noq#802](https://github.com/n0-computer/noq/pull/802) (open) with authorship kept, plus a fix for its one-ACK-late trigger and 1ms floor; the bug is [n0-computer/noq#800](https://github.com/n0-computer/noq/issues/800).
- The rest are not offered yet: they build on the fork-only `PacketId` callbacks, and the series goes upstream together once its shape settles.

## 1.3.0

Parent: n0-computer/noq [`1a26a8b0`](https://github.com/n0-computer/noq/commit/1a26a8b064d21e316fe6769f068617975bd8a27b).

- [#1](https://github.com/moq-dev/noq/pull/1) publish as `moq-noq`, `moq-noq-proto`, and `moq-noq-udp`. Fork-only: packaging.
- [#2](https://github.com/moq-dev/noq/pull/2) add `web-transport-moq`, the WebTransport adapter over the fork. Fork-only: packaging.
