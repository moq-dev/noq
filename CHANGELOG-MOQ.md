# Changelog (moq-dev fork)

Releases of `moq-noq`, `moq-noq-proto`, `moq-noq-udp`, and `web-transport-moq`.
All four share one version. [CHANGELOG.md](CHANGELOG.md) is upstream's and
covers the parent. Each entry names the parent commit it carries and every
change the fork carries on top, with its upstream status, so an advisory
against the parent can be checked against a release.

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
