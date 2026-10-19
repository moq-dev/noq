//! Retry validation and handshake progress after discarded packets.

use assert_matches::assert_matches;

use crate::{ConnectionError, ConnectionHandle, ConnectionId, Event};

use super::util::{Inbound, Pair, client_config, server_config, subscribe, validate_incoming};

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
use wasm_bindgen_test::wasm_bindgen_test as test;

/// Rebuilds a Retry datagram with `src_cid`, under a valid integrity tag
fn forge_retry(retry: &[u8], src_cid: &[u8], orig_dst_cid: ConnectionId) -> Vec<u8> {
    let version = u32::from_be_bytes(retry[1..5].try_into().unwrap());
    let dst_cid_end = 6 + retry[5] as usize;
    let src_cid_end = dst_cid_end + 1 + retry[dst_cid_end] as usize;
    let mut packet = retry[..dst_cid_end].to_vec();
    packet.push(src_cid.len() as u8);
    packet.extend_from_slice(src_cid);
    packet.extend_from_slice(&retry[src_cid_end..retry.len() - 16]);
    let tag = server_config()
        .crypto
        .retry_tag(version, orig_dst_cid, &packet);
    packet.extend_from_slice(&tag);
    packet
}

/// Rewrites the server's Retry datagram, given the client's original DCID, into the datagrams
/// the client receives instead
type Mangle = fn(&[u8], ConnectionId) -> Vec<Vec<u8>>;

/// A Retry whose integrity tag fails
fn bad_tag(retry: &[u8], _: ConnectionId) -> Vec<Vec<u8>> {
    let mut retry = retry.to_vec();
    *retry.last_mut().unwrap() ^= 0xff;
    vec![retry]
}

/// A Retry naming the client's original DCID as the server's new CID
fn same_cid(retry: &[u8], orig_dst_cid: ConnectionId) -> Vec<Vec<u8>> {
    vec![forge_retry(retry, &orig_dst_cid, orig_dst_cid)]
}

/// The server's Retry, then a second, different one
fn second(retry: &[u8], orig_dst_cid: ConnectionId) -> Vec<Vec<u8>> {
    vec![retry.to_vec(), forge_retry(retry, &[0x22; 8], orig_dst_cid)]
}

/// Starts a connection the server Retries, queueing for the client what `mangle` makes of the
/// Retry, and returning the client's original DCID
fn retried(mangle: Mangle) -> (Pair, ConnectionHandle, ConnectionId) {
    let mut pair = Pair::default();
    pair.server.handle_incoming = Box::new(validate_incoming);
    let client_ch = pair.begin_connect(client_config());
    pair.drive_client();
    let (time, initial) = pair.server.inbound.pop_first().unwrap();
    let packet = &initial.packet;
    let orig_dst_cid = ConnectionId::new(&packet[6..6 + packet[5] as usize]);
    pair.server.inbound.push(time, initial);
    pair.drive_server();
    let (time, retry) = pair.client.inbound.pop_last().unwrap();
    assert!(pair.client.inbound.is_empty());
    assert_eq!(retry.packet[0] & 0xf0, 0xf0, "a Retry");
    for datagram in mangle(&retry.packet, orig_dst_cid) {
        pair.client.inbound.push(
            time,
            Inbound {
                packet: datagram.as_slice().into(),
                ..retry.clone()
            },
        );
    }
    (pair, client_ch, orig_dst_cid)
}

/// The client's next Initial keeps its original DCID unless it accepted a Retry
#[test]
fn invalid_retry_is_discarded() {
    let _guard = subscribe();
    for (mangle, accepted) in [
        (bad_tag as Mangle, false),
        (same_cid, false),
        (second, true),
    ] {
        let (mut pair, _, orig_dst_cid) = retried(mangle);
        pair.drive_client();
        while pair.server.inbound.is_empty() {
            pair.time = pair.client.next_wakeup().unwrap();
            pair.drive_client();
        }
        let (_, initial) = pair.server.inbound.pop_first().unwrap();
        let packet = &initial.packet;
        let dst_cid = ConnectionId::new(&packet[6..6 + packet[5] as usize]);
        assert_eq!(dst_cid != orig_dst_cid, accepted);
    }
}

/// The client follows the first Retry and ignores the second
#[test]
fn second_retry_is_discarded() {
    let _guard = subscribe();
    let (mut pair, client_ch, _) = retried(second);
    pair.drive();
    let server_ch = pair.server.assert_accept();
    pair.lax_finish_connect(client_ch, server_ch);
}

/// A discarded Retry does not stop the client from following the server's genuine Retry of its
/// retransmitted Initial
#[test]
fn retry_after_invalid_retry() {
    let _guard = subscribe();
    for mangle in [bad_tag as Mangle, same_cid] {
        let (mut pair, client_ch, _) = retried(mangle);
        pair.drive();
        let server_ch = pair.server.assert_accept();
        pair.lax_finish_connect(client_ch, server_ch);
    }
}

/// A discarded Retry does not stop the client from processing a Version Negotiation packet
#[test]
fn version_negotiation_after_invalid_retry() {
    let _guard = subscribe();
    for mangle in [bad_tag as Mangle, same_cid] {
        let (mut pair, client_ch, orig_dst_cid) = retried(mangle);
        let (time, retry) = pair.client.inbound.pop_last().unwrap();
        pair.client.inbound.push(time, retry.clone());
        let packet = &retry.packet;
        // Echo the client's CIDs, offering only a reserved version
        let dst_cid_end = 6 + packet[5] as usize;
        let mut vn = vec![0x80, 0, 0, 0, 0];
        vn.extend_from_slice(&packet[5..dst_cid_end]);
        vn.push(orig_dst_cid.len() as u8);
        vn.extend_from_slice(&orig_dst_cid);
        vn.extend_from_slice(&[0x0a, 0x1a, 0x2a, 0x3a]);
        pair.client.inbound.push(
            time,
            Inbound {
                packet: vn.as_slice().into(),
                ecn: None,
                ..retry
            },
        );
        pair.drive_client();
        assert_matches!(
            pair.client_conn_mut(client_ch).poll(),
            Some(Event::ConnectionLost {
                reason: ConnectionError::VersionMismatch
            })
        );
    }
}
