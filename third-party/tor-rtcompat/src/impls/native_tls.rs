//! Implementation for using `native_tls`

use crate::traits::{CertifiedConn, TlsConnector, TlsProvider};

use async_trait::async_trait;
use futures::{AsyncRead, AsyncWrite};
use native_tls_crate as native_tls;
use std::{
    convert::TryInto,
    io::{Error as IoError, Result as IoResult},
};

/// A [`TlsProvider`] that uses `native_tls`.
///
/// It supports wrapping any reasonable stream type that implements `AsyncRead` + `AsyncWrite`.
#[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
#[derive(Default, Clone)]
#[non_exhaustive]
pub struct NativeTlsProvider {}

impl<S> CertifiedConn for async_native_tls::TlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn peer_certificate(&self) -> IoResult<Option<Vec<u8>>> {
        let cert = self.peer_certificate();
        match cert {
            Ok(Some(c)) => {
                let der = c
                    .to_der()
                    .map_err(|e| IoError::new(std::io::ErrorKind::Other, e))?;
                Ok(Some(der))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(IoError::new(std::io::ErrorKind::Other, e)),
        }
    }
}

/// An implementation of [`TlsConnector`] built with `native_tls`.
pub struct NativeTlsConnector<S> {
    /// The inner connector object.
    connector: async_native_tls::TlsConnector,
    /// Phantom data to ensure proper variance.
    _phantom: std::marker::PhantomData<fn(S) -> S>,
}

#[async_trait]
impl<S> TlsConnector<S> for NativeTlsConnector<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Conn = async_native_tls::TlsStream<S>;

    async fn negotiate_unvalidated(&self, stream: S, sni_hostname: &str) -> IoResult<Self::Conn> {
        let conn = self
            .connector
            .connect(sni_hostname, stream)
            .await
            .map_err(|e| IoError::new(std::io::ErrorKind::Other, e))?;
        Ok(conn)
    }
}

impl<S> TlsProvider<S> for NativeTlsProvider
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Connector = NativeTlsConnector<S>;

    type TlsStream = async_native_tls::TlsStream<S>;

    fn tls_connector(&self) -> Self::Connector {
        let mut builder = native_tls::TlsConnector::builder();
        // These function names are scary, but they just mean that we
        // aren't checking whether the signer of this cert
        // participates in the web PKI, and we aren't checking the
        // hostname in the cert.
        builder
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true);

        let connector = builder.try_into().expect("Couldn't build a TLS connector!");

        NativeTlsConnector {
            connector,
            _phantom: std::marker::PhantomData,
        }
    }
}
