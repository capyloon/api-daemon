//! Messages sent over Tor channels
//!
//! A 'channel' is a direct connection between a tor client and a
//! relay, or between two relays.  Current channels all use TLS.
//!
//! This module implements the [ChanCell] type, which is the encoding for
//! data sent over a channel.  It also encodes and decodes various
//! channel messages, which are the types of data conveyed over a
//! channel.
pub mod codec;
pub mod msg;
use caret::caret_int;

/// The amount of data sent in a fixed-length cell.
///
/// Historically, this was set at 509 bytes so that cells would be
/// 512 bytes long once commands and circuit IDs were added.  But since
/// protocol version 4, circuit IDs are 2 bytes longer, so cells are
/// now 514 bytes.
pub const CELL_DATA_LEN: usize = 509;

/// A cell body considered as a raw array of bytes
pub type RawCellBody = [u8; CELL_DATA_LEN];

/// Channel-local identifier for a circuit.
///
/// A circuit ID can be 2 or 4 bytes long; since version 4 of the Tor
/// protocol, it's 4 bytes long.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct CircId(u32);

impl From<u32> for CircId {
    fn from(item: u32) -> Self {
        Self(item)
    }
}
impl From<CircId> for u32 {
    fn from(id: CircId) -> u32 {
        id.0
    }
}
impl std::fmt::Display for CircId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}
impl CircId {
    /// Return true if this is the zero CircId.
    ///
    /// A zero-valid circuit ID denotes a cell that is not related to
    /// any particular circuit, but which applies to the channel as a whole.
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

caret_int! {
    /// A ChanCmd is the type of a channel cell.  The value of the ChanCmd
    /// indicates the meaning of the cell, and (possibly) its length.
    pub struct ChanCmd(u8) {
        /// A fixed-length cell that will be dropped.
        PADDING = 0,
        /// Create a new circuit (obsolete format)
        CREATE = 1,
        /// Finish circuit-creation handshake (obsolete format)
        CREATED = 2,
        /// Relay cell, transmitted over a circuit.
        RELAY = 3,
        /// Destroy a circuit
        DESTROY = 4,
        /// Create a new circuit (no public-key)
        CREATE_FAST = 5,
        /// Finish a circuit-creation handshake (no public-key)
        CREATED_FAST = 6,
        // note gap in numbering: 7 is grouped with the variable-length cells
        /// Finish a channel handshake with time and address information
        NETINFO = 8,
        /// Relay cell, transmitted over a circuit.  Limited.
        RELAY_EARLY = 9,
        /// Create a new circuit (current format)
        CREATE2 = 10,
        /// Finish a circuit-creation handshake (current format)
        CREATED2 = 11,
        /// Adjust channel-padding settings
        PADDING_NEGOTIATE = 12,

        /// Variable-length cell, despite its number: negotiate versions
        VERSIONS = 7,
        /// Variable-length channel-padding cell
        VPADDING = 128,
        /// Provide additional certificates beyond those given in the TLS
        /// handshake
        CERTS = 129,
        /// Challenge material used in relay-to-relay handshake.
        AUTH_CHALLENGE = 130,
        /// Response material used in relay-to-relay handshake.
        AUTHENTICATE = 131,
        /// Indicates client permission to use relay.  Not currently used.
        AUTHORIZE = 132,
    }
}

/// Possible requirements on circuit IDs for a channel command.
enum CircIdReq {
    /// indicates a command that only takes a zero-valued circuit ID
    WantZero,
    /// indicates a command that only takes a nonzero-valued circuit ID
    WantNonZero,
    /// indicates a command that can take any circuit ID
    Any,
}

impl ChanCmd {
    /// Return true if this command is for a cell using the the
    /// variable-length format.
    pub fn is_var_cell(self) -> bool {
        // Version 1 of the channel protocol had no variable-length
        // cells, but that's obsolete.  In version 2, only the VERSIONS
        // cell was variable-length.  Since version 3, all cells having
        // a command value >= 128 are variable-length.
        self == ChanCmd::VERSIONS || self.0 >= 128_u8
    }
    /// Return what kind of circuit ID this command expects.
    fn allows_circid(self) -> CircIdReq {
        match self {
            ChanCmd::PADDING
            | ChanCmd::NETINFO
            | ChanCmd::PADDING_NEGOTIATE
            | ChanCmd::VERSIONS
            | ChanCmd::VPADDING
            | ChanCmd::CERTS
            | ChanCmd::AUTH_CHALLENGE
            | ChanCmd::AUTHENTICATE => CircIdReq::WantZero,
            ChanCmd::CREATE
            | ChanCmd::CREATED
            | ChanCmd::RELAY
            | ChanCmd::DESTROY
            | ChanCmd::CREATE_FAST
            | ChanCmd::CREATED_FAST
            | ChanCmd::RELAY_EARLY
            | ChanCmd::CREATE2
            | ChanCmd::CREATED2 => CircIdReq::WantNonZero,
            _ => CircIdReq::Any,
        }
    }
    /// Return true if this command is one that accepts the particular
    /// circuit ID `id`.
    pub fn accepts_circid_val(self, id: CircId) -> bool {
        match (self.allows_circid(), id.is_zero()) {
            (CircIdReq::WantNonZero, true) => false,
            (CircIdReq::WantZero, false) => false,
            (_, _) => true,
        }
    }
}

/// A decoded channel cell, to be sent or received on a channel.
#[derive(Debug)]
pub struct ChanCell {
    /// Circuit ID associated with this cell
    circid: CircId,
    /// Underlying message in this cell
    msg: msg::ChanMsg,
}

impl ChanCell {
    /// Construct a new channel cell.
    pub fn new(circid: CircId, msg: msg::ChanMsg) -> Self {
        ChanCell { circid, msg }
    }
    /// Return the circuit ID for this cell.
    pub fn circid(&self) -> CircId {
        self.circid
    }
    /// Return a reference to the underlying message of this cell.
    pub fn msg(&self) -> &msg::ChanMsg {
        &self.msg
    }
    /// Consume this cell and return its components.
    pub fn into_circid_and_msg(self) -> (CircId, msg::ChanMsg) {
        (self.circid, self.msg)
    }
}
