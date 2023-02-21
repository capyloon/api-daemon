//! Define a response type for directory requests.

use std::str;

use tor_linkspec::{LoggedChanTarget, OwnedChanTarget};
use tor_proto::circuit::{ClientCirc, UniqId};

use crate::{RequestError, RequestFailedError};

/// A successful (or at any rate, well-formed) response to a directory
/// request.
#[derive(Debug, Clone)]
pub struct DirResponse {
    /// An HTTP status code.
    status: u16,
    /// The decompressed output that we got from the directory cache.
    output: Vec<u8>,
    /// The error, if any, that caused us to stop getting this response early.
    error: Option<RequestError>,
    /// Information about the directory cache we used.
    source: Option<SourceInfo>,
}

/// Information about the source of a directory response.
///
/// We use this to remember when a request has failed, so we can
/// abandon the circuit.
#[derive(Debug, Clone, derive_more::Display)]
#[display(fmt = "{} via {}", cache_id, circuit)]
pub struct SourceInfo {
    /// Unique identifier for the circuit we're using
    circuit: UniqId,
    /// Identity of the directory cache that provided us this information.
    cache_id: LoggedChanTarget,
}

impl DirResponse {
    /// Construct a new DirResponse from its parts
    pub(crate) fn new(
        status: u16,
        error: Option<RequestError>,
        output: Vec<u8>,
        source: Option<SourceInfo>,
    ) -> Self {
        DirResponse {
            status,
            output,
            error,
            source,
        }
    }

    /// Construct a new successful DirResponse from its body.
    pub fn from_body(body: impl AsRef<[u8]>) -> Self {
        Self::new(200, None, body.as_ref().to_vec(), None)
    }

    /// Return the HTTP status code for this response.
    pub fn status_code(&self) -> u16 {
        self.status
    }

    /// Return true if this is in incomplete response.
    pub fn is_partial(&self) -> bool {
        self.error.is_some()
    }

    /// Return the error from this response, if any.
    pub fn error(&self) -> Option<&RequestError> {
        self.error.as_ref()
    }

    /// Return the output from this response.
    ///
    /// Returns some output, even if the response indicates truncation or an error.
    pub fn output_unchecked(&self) -> &[u8] {
        &self.output
    }

    /// Return the output from this response, if it was successful and complete.
    pub fn output(&self) -> Result<&[u8], RequestFailedError> {
        self.check_ok()?;
        Ok(self.output_unchecked())
    }

    /// Return this the output from this response, as a string,
    /// if it was successful and complete and valid UTF-8.
    pub fn output_string(&self) -> Result<&str, RequestFailedError> {
        let output = self.output()?;
        let s = str::from_utf8(output).map_err(|_| RequestFailedError {
            // For RequestError::Utf8Encoding We need a `String::FromUtf8Error`
            // (which contains an owned copy of the bytes).
            error: String::from_utf8(output.to_owned())
                .expect_err("was bad, now good")
                .into(),
            source: self.source.clone(),
        })?;
        Ok(s)
    }

    /// Consume this DirResponse and return the output in it.
    ///
    /// Returns some output, even if the response indicates truncation or an error.
    pub fn into_output_unchecked(self) -> Vec<u8> {
        self.output
    }

    /// Consume this DirResponse and return the output, if it was successful and complete.
    pub fn into_output(self) -> Result<Vec<u8>, RequestFailedError> {
        self.check_ok()?;
        Ok(self.into_output_unchecked())
    }

    /// Consume this DirResponse and return the output, as a string,
    /// if it was successful and complete and valid UTF-8.
    pub fn into_output_string(self) -> Result<String, RequestFailedError> {
        self.check_ok()?;
        let s = String::from_utf8(self.output).map_err(|error| RequestFailedError {
            error: error.into(),
            source: self.source.clone(),
        })?;
        Ok(s)
    }

    /// Return the source information about this response.
    pub fn source(&self) -> Option<&SourceInfo> {
        self.source.as_ref()
    }

    /// Check if this request was successful and complete.
    fn check_ok(&self) -> Result<(), RequestFailedError> {
        let wrap_err = |error| {
            Err(RequestFailedError {
                error,
                source: self.source.clone(),
            })
        };
        if let Some(error) = &self.error {
            return wrap_err(error.clone());
        }
        assert!(!self.is_partial(), "partial but no error?");
        if self.status_code() != 200 {
            return wrap_err(RequestError::HttpStatus(self.status_code()));
        }
        Ok(())
    }
}

impl SourceInfo {
    /// Construct a new SourceInfo
    pub(crate) fn from_circuit(circuit: &ClientCirc) -> Self {
        SourceInfo {
            circuit: circuit.unique_id(),
            cache_id: circuit.first_hop().into(),
        }
    }

    /// Return the unique circuit identifier for the circuit on which
    /// we received this info.
    pub fn unique_circ_id(&self) -> &UniqId {
        &self.circuit
    }

    /// Return information about the peer from which we received this info.
    pub fn cache_id(&self) -> &OwnedChanTarget {
        self.cache_id.as_inner()
    }
}

#[cfg(test)]
mod test {
    // @@ begin test lint list maintained by maint/add_warning @@
    #![allow(clippy::bool_assert_comparison)]
    #![allow(clippy::clone_on_copy)]
    #![allow(clippy::dbg_macro)]
    #![allow(clippy::print_stderr)]
    #![allow(clippy::print_stdout)]
    #![allow(clippy::single_char_pattern)]
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::unchecked_duration_subtraction)]
    //! <!-- @@ end test lint list maintained by maint/add_warning @@ -->
    use super::*;

    #[test]
    fn errors() {
        let mut response = DirResponse::new(200, None, vec![b'Y'], None);

        assert_eq!(response.output().unwrap(), b"Y");
        assert_eq!(response.clone().into_output().unwrap(), b"Y");

        let expect_error = |response: &DirResponse, error: RequestError| {
            let error = RequestFailedError {
                error,
                source: None,
            };
            let error = format!("{:?}", error);

            assert_eq!(error, format!("{:?}", response.output().unwrap_err()));
            assert_eq!(
                error,
                format!("{:?}", response.clone().into_output().unwrap_err())
            );
        };

        let with_error = |response: &DirResponse| {
            let mut response = response.clone();
            response.error = Some(RequestError::DirTimeout);
            expect_error(&response, RequestError::DirTimeout);
        };

        with_error(&response);

        response.status = 404;
        expect_error(&response, RequestError::HttpStatus(404));

        with_error(&response);
    }
}
