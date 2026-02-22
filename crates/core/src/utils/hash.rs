use digest::Digest;
use std::io::{self, Read};

pub trait HashFsExt {
    fn update<R: Read>(&mut self, reader: R) -> io::Result<()>;
}

impl<D: Digest> HashFsExt for D {
    fn update<R: Read>(&mut self, mut reader: R) -> io::Result<()> {
        let mut buffer = [0u8; 64 * 1024]; // 64KB buffer
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            self.update(&buffer[..bytes_read]);
        }
        Ok(())
    }
}

#[cfg(feature = "io-utils")]
pub trait HashAsyncFsExt {
    fn update<R: tokio::io::AsyncRead + Send + Unpin>(
        &mut self,
        reader: R,
    ) -> impl Future<Output = io::Result<()>> + Send;
}

#[cfg(feature = "io-utils")]
impl<D: Digest + Send> HashAsyncFsExt for D {
    async fn update<R: tokio::io::AsyncRead + Send + Unpin>(
        &mut self,
        mut reader: R,
    ) -> io::Result<()> {
        use tokio::io::AsyncReadExt;

        let mut buffer = [0u8; 64 * 1024]; // 64KB buffer
        loop {
            let bytes_read = reader.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            self.update(&buffer[..bytes_read]);
        }
        Ok(())
    }
}
