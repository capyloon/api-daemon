//! Implementation for encoding and decoding of ChanCells.

use super::CELL_DATA_LEN;
use crate::chancell::{msg, ChanCell, ChanCmd, CircId};
use crate::Error;
use arrayref::{array_mut_ref, array_ref};
use tor_bytes::{self, Reader, Writer};
use tor_error::internal;

use bytes::BytesMut;

/// This object can be used to encode and decode channel cells.
///
/// NOTE: only link protocol versions 3 and higher are supported.
/// VERSIONS cells are not supported via the encoder/decoder, since it
/// VERSIONS always uses a two-byte circuit-ID.
///
/// The implemented format is one of the following:
///
/// ```ignore
///     u32 circid;
///     u8 command;
///     u16 len;
///     u8 body[len];
/// ```
///
/// ```ignore
///     u32 circid;
///     u8 command;
///     u8 body[509];
/// ```
pub struct ChannelCodec {
    #[allow(dead_code)] // We don't support any link versions where this matters
    /// The link protocol version being used for this channel.
    ///
    /// (We don't currently support any versions of the link protocol
    /// where this version matters, but for some older versions, it would
    /// affect the length of the circuit ID.)
    link_version: u16,
}

impl ChannelCodec {
    /// Create a new ChannelCodec with a given link protocol version
    pub fn new(link_version: u16) -> Self {
        ChannelCodec { link_version }
    }

    /// Write the given cell into the provided BytesMut object.
    pub fn write_cell(&mut self, item: ChanCell, dst: &mut BytesMut) -> crate::Result<()> {
        let ChanCell { circid, msg } = item;
        let cmd = msg.cmd();
        dst.write_u32(circid.into());
        dst.write_u8(cmd.into());

        let pos = dst.len(); // always 5?

        // now write the cell body and handle the length.
        if cmd.is_var_cell() {
            dst.write_u16(0);
            msg.write_body_onto(dst)?;
            let len = dst.len() - pos - 2;
            if len > std::u16::MAX as usize {
                return Err(Error::Internal(internal!("ran out of space for varcell")));
            }
            // go back and set the length.
            *(array_mut_ref![&mut dst[pos..pos + 2], 0, 2]) = (len as u16).to_be_bytes();
        } else {
            msg.write_body_onto(dst)?;
            let len = dst.len() - pos;
            if len > CELL_DATA_LEN {
                return Err(Error::Internal(internal!("ran out of space for cell")));
            }
            // pad to end of fixed-length cell
            dst.write_zeros(CELL_DATA_LEN - len);
        }
        Ok(())
    }

    /// Try to decode a cell from the provided BytesMut object.
    ///
    /// On a definite decoding error, return Err(_).  On a cell that might
    /// just be truncated, return Ok(None).
    pub fn decode_cell(&mut self, src: &mut BytesMut) -> crate::Result<Option<ChanCell>> {
        /// Wrap `be` as an appropriate type.
        fn wrap_err(be: tor_bytes::Error) -> crate::Error {
            crate::Error::BytesErr {
                err: be,
                parsed: "channel cell",
            }
        }

        if src.len() < 7 {
            // Smallest possible command: varcell with len 0
            return Ok(None);
        }
        let cmd: ChanCmd = src[4].into();
        let varcell = cmd.is_var_cell();
        let cell_len: usize = if varcell {
            let msg_len = u16::from_be_bytes(*array_ref![&src[5..7], 0, 2]);
            msg_len as usize + 7
        } else {
            514
        };
        if src.len() < cell_len {
            return Ok(None);
        }

        let cell = src.split_to(cell_len).freeze();
        //trace!("{:?} cell body ({}) is {:?}", cmd, cell.len(), &cell[..]);
        let mut r = Reader::from_bytes(&cell);
        let circid: CircId = r.take_u32().map_err(wrap_err)?.into();
        r.advance(if varcell { 3 } else { 1 }).map_err(wrap_err)?;
        let msg = msg::ChanMsg::take(&mut r, cmd).map_err(wrap_err)?;

        if !cmd.accepts_circid_val(circid) {
            return Err(Error::ChanProto(format!(
                "Invalid circuit ID {} for cell command {}",
                circid, cmd
            )));
        }
        Ok(Some(ChanCell { circid, msg }))
    }
}
