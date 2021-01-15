use actix_web::{dev::MessageBody, error::PayloadError, Error, HttpResponse};
use bytes::{Bytes, BytesMut};
use futures::{
    future::ready,
    stream::{Stream, StreamExt},
};
pub use rand::random;
use rand::{self, distributions::Alphanumeric, Rng};
use std::{fs::File, io::Read};

/// Generates a random string with given size.
pub fn random_string(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .collect::<String>()
}

/// Reads file content into string result.
pub fn read_file(file: &str) -> Result<String, anyhow::Error> {
    let mut file = File::open(file)?;
    let mut content = String::new();
    let _ = file.read_to_string(&mut content);
    Ok(content)
}

/// Loads bytes from (Request) stream
pub async fn load_body<S>(stream: S) -> Result<BytesMut, PayloadError>
where
    S: Stream<Item = Result<Bytes, PayloadError>>,
{
    let body = stream
        .map(|res| match res {
            Ok(chunk) => chunk,
            _ => panic!(),
        })
        .fold(BytesMut::new(), move |mut body, chunk| {
            body.extend_from_slice(&chunk);
            ready(body)
        })
        .await;

    Ok(body)
}

/// Reads bytes from HttpResponse.
pub async fn read_body<B>(mut res: HttpResponse<B>) -> Result<Bytes, Error>
where
    B: MessageBody + Unpin,
{
    let mut body = res.take_body();
    let mut bytes = BytesMut::new();
    while let Some(item) = body.next().await {
        bytes.extend_from_slice(&item?);
    }
    Ok(bytes.freeze())
}

#[cfg(test)]
mod tests {

    use super::*;
    use actix_web::{
        test::{call_service, init_service, TestRequest},
        web, App, HttpResponse,
    };
    use bytes::Bytes;
    use futures::{future::ok, stream};

    #[test]
    fn random_string_has_given_size() {
        let size: u8 = random();
        let string = random_string(size as usize);

        assert_eq!(string.len(), size as usize);
    }

    #[test]
    fn random_strings_are_different() {
        let size: u8 = std::cmp::max(1, random());
        let first_string = random_string(size as usize);
        let second_string = random_string(size as usize);

        assert!(first_string != second_string);
    }

    #[test]
    fn read_file_returns_error_if_file_not_exists() {
        let not_exist = read_file("a");

        assert!(not_exist.is_err());
    }

    #[test]
    fn read_file_returns_content() {
        let content = read_file("tests/read_file_test");

        assert!(content.is_ok());
        assert_eq!(&content.unwrap(), "a1b2c3");
    }

    #[actix_rt::test]
    async fn read_request_body() {
        let content = &random_string(20);
        let content_bytes = Box::leak(content.clone().into_boxed_str()).as_bytes();
        let payload = Bytes::from(content_bytes);
        let stream = stream::once(ok(payload));

        assert_eq!(load_body(stream).await.unwrap(), Bytes::from(content_bytes));
    }

    #[actix_rt::test]
    async fn read_response_body() {
        let payload = random_string(20);
        let payload_bytes = Bytes::from(payload.clone());

        let mut app = init_service(
            App::new().default_service(web::to(move || HttpResponse::Ok().body(payload.clone()))),
        )
        .await;

        let req = TestRequest::default().to_request();
        let res = call_service(&mut app, req).await;

        assert_eq!(read_body(res.into()).await.unwrap(), payload_bytes);
    }
}
