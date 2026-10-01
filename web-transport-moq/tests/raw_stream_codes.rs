//! A raw QUIC session sends RESET_STREAM and STOP_SENDING codes as is, so a plain QUIC
//! peer agrees on them. It still reads the HTTP/3-mapped codes an older raw peer sends.

use std::{net::Ipv4Addr, sync::Arc, time::Duration};

use anyhow::{Context as _, Result};
use rcgen::{CertifiedKey, KeyPair};
use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use tokio::time::timeout;
use web_transport_moq::{ReadError, Session, WriteError, noq, proto};
use web_transport_trait::Error as _;

const CODE: u32 = 5;

/// With both crypto features enabled, as `--all-features` does, the crate cannot pick a
/// provider for us, so the test installs one.
fn install_crypto_provider() {
    #[cfg(all(feature = "aws-lc-rs", feature = "ring"))]
    {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    }
}

/// A raw session on the client and the plain QUIC connection it talks to, plus the
/// endpoints that drive them.
async fn raw_and_plain() -> Result<(Session, noq::Connection, [noq::Endpoint; 2])> {
    install_crypto_provider();

    let CertifiedKey { cert, signing_key } =
        rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(KeyPair::serialize_der(
        &signing_key,
    )));

    let server_config = noq::ServerConfig::with_single_cert(vec![cert.der().clone()], key)?;
    let server = noq::Endpoint::server(server_config, (Ipv4Addr::LOCALHOST, 0).into())?;

    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert.der().clone())?;
    let client_config = noq::ClientConfig::with_root_certificates(Arc::new(roots))?;
    let client = noq::Endpoint::client((Ipv4Addr::LOCALHOST, 0).into())?;

    let connecting = client.connect_with(client_config, server.local_addr()?, "localhost")?;
    let accept = async { anyhow::Ok(server.accept().await.context("no connection")?.await?) };
    let (client_conn, server_conn) =
        tokio::try_join!(async { anyhow::Ok(connecting.await?) }, accept)?;

    Ok((Session::raw(client_conn), server_conn, [client, server]))
}

/// Open a uni stream from the plain peer and accept it on the session.
async fn plain_to_raw(
    session: &Session,
    plain: &noq::Connection,
) -> Result<(noq::SendStream, web_transport_moq::RecvStream)> {
    let mut send = plain.open_uni().await?;
    send.write_all(b"x").await?;
    let mut recv = timeout(Duration::from_secs(5), session.accept_uni()).await??;
    let mut buf = [0u8; 1];
    recv.read_exact(&mut buf).await?;
    Ok((send, recv))
}

/// Open a uni stream from the session and accept it on the plain peer.
async fn raw_to_plain(
    session: &Session,
    plain: &noq::Connection,
) -> Result<(web_transport_moq::SendStream, noq::RecvStream)> {
    let mut send = session.open_uni().await?;
    send.write_all(b"x").await?;
    let mut recv = timeout(Duration::from_secs(5), plain.accept_uni()).await??;
    let mut buf = [0u8; 1];
    recv.read_exact(&mut buf).await?;
    Ok((send, recv))
}

async fn read_err(recv: &mut web_transport_moq::RecvStream) -> ReadError {
    let mut buf = [0u8; 1];
    timeout(Duration::from_secs(5), recv.read(&mut buf))
        .await
        .expect("no reset")
        .expect_err("read after a reset")
}

async fn write_err(send: &mut web_transport_moq::SendStream) -> WriteError {
    timeout(Duration::from_secs(5), async {
        loop {
            if let Err(err) = send.write(&[0u8; 1024]).await {
                return err;
            }
        }
    })
    .await
    .expect("no stop")
}

#[tokio::test]
async fn reset_is_sent_as_is() -> Result<()> {
    let (session, plain, _endpoints) = raw_and_plain().await?;
    let (mut send, mut recv) = raw_to_plain(&session, &plain).await?;

    send.reset(CODE)?;

    let mut buf = [0u8; 1];
    let err = timeout(Duration::from_secs(5), recv.read(&mut buf)).await?;
    assert!(
        matches!(err, Err(noq::ReadError::Reset(code)) if code == CODE.into()),
        "{err:?}"
    );
    Ok(())
}

#[tokio::test]
async fn stop_is_sent_as_is() -> Result<()> {
    let (session, plain, _endpoints) = raw_and_plain().await?;
    let (send, mut recv) = plain_to_raw(&session, &plain).await?;

    recv.stop(CODE)?;

    let stopped = timeout(Duration::from_secs(5), send.stopped()).await??;
    assert_eq!(stopped, Some(CODE.into()));
    Ok(())
}

#[tokio::test]
async fn peer_reset_is_read_as_is() -> Result<()> {
    let (session, plain, _endpoints) = raw_and_plain().await?;
    let (mut send, mut recv) = plain_to_raw(&session, &plain).await?;

    send.reset(CODE.into())?;

    let err = read_err(&mut recv).await;
    assert_eq!(err.stream_error(), Some(CODE), "{err}");
    Ok(())
}

#[tokio::test]
async fn peer_stop_is_read_as_is() -> Result<()> {
    let (session, plain, _endpoints) = raw_and_plain().await?;
    let (mut send, mut recv) = raw_to_plain(&session, &plain).await?;

    recv.stop(CODE.into())?;

    let err = write_err(&mut send).await;
    assert_eq!(err.stream_error(), Some(CODE), "{err}");
    assert_eq!(send.stopped().await?, Some(CODE));
    Ok(())
}

/// An older raw peer maps its codes into the HTTP/3 range, which still decodes.
#[tokio::test]
async fn legacy_peer_codes_are_unmapped() -> Result<()> {
    let (session, plain, _endpoints) = raw_and_plain().await?;
    let legacy = noq::VarInt::try_from(proto::error_to_http3(CODE))?;

    let (mut send, mut recv) = plain_to_raw(&session, &plain).await?;
    send.reset(legacy)?;
    let err = read_err(&mut recv).await;
    assert_eq!(err.stream_error(), Some(CODE), "{err}");

    let (mut send, mut recv) = raw_to_plain(&session, &plain).await?;
    recv.stop(legacy)?;
    let err = write_err(&mut send).await;
    assert_eq!(err.stream_error(), Some(CODE), "{err}");
    assert_eq!(send.stopped().await?, Some(CODE));
    Ok(())
}
