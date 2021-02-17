/// A wrapper type around a Vec<u8> providing Read and Seek implementations.
use async_std::io::{Read, Seek, SeekFrom};
use async_std::task::{Context, Poll};
use futures::io::{Error as FutError, ErrorKind};
use std::pin::Pin;

pub struct Array {
    pos: u64,
    data: Vec<u8>,
}

impl Array {
    pub fn new(data: Vec<u8>) -> Self {
        Self { pos: 0, data }
    }
}

impl Seek for Array {
    fn poll_seek(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        pos: SeekFrom,
    ) -> Poll<Result<u64, FutError>> {
        let (base_pos, offset) = match pos {
            SeekFrom::Start(n) => {
                self.get_mut().pos = n;
                return Poll::Ready(Ok(n));
            }
            SeekFrom::End(n) => (self.data.len() as u64, n),
            SeekFrom::Current(n) => (self.pos, n),
        };
        let new_pos = if offset >= 0 {
            base_pos.checked_add(offset as u64)
        } else {
            base_pos.checked_sub((offset.wrapping_neg()) as u64)
        };
        match new_pos {
            Some(n) => {
                self.get_mut().pos = n;
                Poll::Ready(Ok(n))
            }
            None => Poll::Ready(Err(FutError::from(ErrorKind::InvalidInput))),
        }
    }
}

impl Read for Array {
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        result: &mut [u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let len = result.len();

        let me = self.get_mut();
        let upos = me.pos as usize;
        let max: usize = me.data.len() - upos;
        let to_read = std::cmp::min(len, max);

        if to_read == 1 {
            result[0] = me.data[upos];
        } else if to_read != 0 {
            let end = upos + to_read;
            result[..to_read].copy_from_slice(&me.data.as_slice()[upos..end]);
        }

        me.pos += to_read as u64;
        Poll::Ready(Ok(to_read))
    }
}

impl crate::common::ReaderTrait for Array {}

#[async_std::test]
async fn array_read() {
    use async_std::io::ReadExt;

    let content = b"Hello World!".to_vec();
    let mut array = Array::new(content);

    let mut result = String::new();
    let len = array.read_to_string(&mut result).await.unwrap();

    assert_eq!(len, 12);
    assert_eq!(&result, "Hello World!");

    // Standard buffer is 32 bytes, test a longer array.
    let content = b"Hello World 1! Hello World 2! Hello World 3! Hello World 4!".to_vec();
    let mut array = Array::new(content);

    let mut result = String::new();
    let len = array.read_to_string(&mut result).await.unwrap();

    assert_eq!(len, 59);
    assert_eq!(
        &result,
        "Hello World 1! Hello World 2! Hello World 3! Hello World 4!"
    );

    // Special case for reading a single character, using a length of 33.
    let content = b"H".to_vec();
    let mut array = Array::new(content);

    let mut result = String::new();
    let len = array.read_to_string(&mut result).await.unwrap();

    assert_eq!(len, 1);
    assert_eq!(&result, "H");
}
