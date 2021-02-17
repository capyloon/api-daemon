/// A XOR based "encryption" resource transformer and resource name provider.
use crate::common::{
    BoxedReader, ReaderTrait, ResourceId, ResourceNameProvider, ResourceStoreError,
    ResourceTransformer,
};
use crate::file_store::FileStore;
use async_std::io::{Read, Seek, SeekFrom};
use async_std::task::{Context, Poll};
use pin_project_lite::pin_project;
use std::pin::Pin;

struct XorNameProvider {
    xor: u8,
}

impl XorNameProvider {
    pub fn new(xor: u8) -> Self {
        Self { xor }
    }

    /// Transform a string in a xored + base64 version, safely usable as a file name.
    pub fn transform(&self, what: &str) -> String {
        let xored: Vec<u8> = what.chars().map(|c| (c as u8) ^ self.xor).collect();
        base64::encode_config(&xored, base64::BCRYPT)
    }
}

impl ResourceNameProvider for XorNameProvider {
    fn metadata_name(&self, id: &ResourceId) -> String {
        self.transform(&format!("{}.meta", id))
    }

    fn variant_name(&self, id: &ResourceId, variant: &str) -> String {
        self.transform(&format!("{}.{}.content", id, variant))
    }
}

pub struct XorTransformer {
    xor: u8,
}

impl XorTransformer {
    pub fn new(xor: u8) -> Self {
        Self { xor }
    }
}

fn xor_buffer(xor: u8, buffer: &mut [u8], max: usize) {
    let len = std::cmp::min(max, buffer.len());
    for item in buffer.iter_mut().take(len) {
        *item ^= xor;
    }
}

impl ResourceTransformer for XorTransformer {
    fn transform_to(&self, source: BoxedReader) -> BoxedReader {
        Box::new(XorReader::new(self.xor, source))
    }

    fn transform_from(&self, source: BoxedReader) -> BoxedReader {
        Box::new(XorReader::new(self.xor, source))
    }

    fn transform_array_to(&self, source: &[u8]) -> Vec<u8> {
        source.iter().map(|e| e ^ self.xor).collect()
    }

    fn transform_array_from(&self, source: &[u8]) -> Vec<u8> {
        source.iter().map(|e| e ^ self.xor).collect()
    }
}

pin_project! {
    /// Wrap a reader to perform XOR transformation.
    struct XorReader<R> {
        #[pin]
        inner: R,
        xor: u8,
    }
}

impl<R: ReaderTrait> XorReader<R> {
    fn new(xor: u8, inner: R) -> Self {
        Self { xor, inner }
    }

    fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut R> {
        self.project().inner
    }
}

impl<R: ReaderTrait> Seek for XorReader<R> {
    fn poll_seek(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        from: SeekFrom,
    ) -> Poll<Result<u64, std::io::Error>> {
        self.as_mut().get_pin_mut().poll_seek(cx, from)
    }
}

impl<R: ReaderTrait> Read for XorReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<async_std::io::Result<usize>> {
        let res = self.as_mut().get_pin_mut().poll_read(cx, buf);

        match res {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(nread)) => {
                xor_buffer(self.xor, buf, nread);
                Poll::Ready(Ok(nread))
            }
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
        }
    }
}

impl<R: ReaderTrait> ReaderTrait for XorReader<R> {}

pub async fn new_xor_store<P>(path: P, xor: u8) -> Result<FileStore, ResourceStoreError>
where
    P: AsRef<async_std::path::Path>,
{
    FileStore::new(
        path,
        Box::new(XorNameProvider::new(xor)),
        Box::new(XorTransformer::new(xor)),
    )
    .await
}

#[test]
fn xor_buffer_roundtrip() {
    let mut buf = [0u8; 2];
    xor_buffer(32, &mut buf, 2);
    assert_eq!(buf[0], 32);
    assert_eq!(buf[0], 32);

    xor_buffer(32, &mut buf, 2);
    assert_eq!(buf[0], 0);
    assert_eq!(buf[0], 0);
}
