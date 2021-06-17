use bincode::{self, Options};
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use log::debug;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::BufWriter;
use std::io::{Read, Write};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] ::std::io::Error),
    #[error("Bincode Error: {0}")]
    Bincode(#[from] bincode::Error),
}

type Result<T> = ::std::result::Result<T, Error>;

// We use a simple framing with a u32 size followed by the data,
pub struct Frame;

impl Frame {
    /// Tries to read a Frame from an io:Read implementation.
    pub fn read_from<T: Read>(source: &mut T) -> Result<Vec<u8>> {
        // Read the frame length as a unsigned 32 bits network order integer.
        let size = source.read_u32::<NativeEndian>()? as usize;
        debug!("read_from size={}", size);

        let mut data = Vec::with_capacity(size);
        unsafe {
            data.set_len(size);
        }

        source.read_exact(&mut data)?;
        Ok(data)
    }

    /// Tries to write a Frame to an io:Write implementation.
    pub fn write_to<T: Write>(data: &[u8], dest: &mut T) -> Result<()> {
        debug!("write_to size={}", data.len());
        dest.write_u32::<NativeEndian>(data.len() as u32)?;
        dest.write_all(data)?;

        dest.flush()?;
        Ok(())
    }

    /// Tries to write a struct that can be serialized with bincode.
    pub fn serialize_to<T: Write, S: Serialize>(obj: &S, dest: &mut T) -> Result<()> {
        let size = crate::get_bincode().serialized_size(obj)?;
        debug!("serialize_to size={}", size);
        dest.write_u32::<NativeEndian>(size as u32)?;
        let buffer = BufWriter::new(dest);
        crate::get_bincode()
            .serialize_into(buffer, obj)
            .map_err(|e| e.into())
    }

    /// Tries to read a struct that can be deserialized with bincode.
    /// In that case we ignore the framing length because the bincode
    /// deserializer doesn't need it.
    pub fn deserialize_from<T: Read, D: DeserializeOwned>(source: &mut T) -> Result<D> {
        let size = source.read_u32::<NativeEndian>()?;
        debug!("deserialize_from size={}", size);
        crate::get_bincode()
            .deserialize_from(source)
            .map_err(|e| e.into())
    }
}
