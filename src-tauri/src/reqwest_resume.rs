//! Wrapper that uses the `Range` HTTP header to resume get requests.
//!
//!
//! # Example
//!
//! ```
//! use async_compression::futures::bufread::GzipDecoder;
//! use futures::{io::BufReader, AsyncBufReadExt, StreamExt, TryStreamExt};
//! use std::io;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let url = "http://commoncrawl.s3.amazonaws.com/crawl-data/CC-MAIN-2018-30/warc.paths.gz";
//! let body = reqwest_resume::get(url.parse().unwrap()).await.unwrap();
//! // Content-Encoding isn't set, so decode manually
//! let body = body
//!     .bytes_stream()
//!     .map_err(|e| io::Error::new(io::ErrorKind::Other, e));
//! let body = BufReader::new(body.into_async_read());
//! let mut body = GzipDecoder::new(body); // Content-Encoding isn't set, so decode manually
//! body.multiple_members(true);
//!
//! let mut lines = BufReader::new(body).lines();
//! while let Some(line) = lines.next().await {
//!     println!("{}", line.unwrap());
//! }
//! # }
//! ```

use bytes::Bytes;
use futures::{ready, FutureExt, Stream, TryFutureExt};
use reqwest::header::ACCEPT_RANGES;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::sleep;

/// Extension to [`reqwest::Client`] that provides a method to convert it
pub trait ClientExt {
    /// Convert a [`reqwest::Client`] into a [`reqwest_resume::Client`](Client)
    fn resumable(self) -> Client;
}
impl ClientExt for reqwest::Client {
    fn resumable(self) -> Client {
        Client(self)
    }
}

/// A `Client` to make Requests with.
///
/// See [`reqwest::Client`].
#[derive(Debug)]
pub struct Client(reqwest::Client);
impl Client {
    /// Constructs a new `Client`.
    ///
    /// See [`reqwest::Client::new()`].
    pub fn new() -> Self {
        Self(reqwest::Client::new())
    }
    /// Convenience method to make a `GET` request to a URL.
    ///
    /// See [`reqwest::Client::get()`].
    pub fn get(&self, url: reqwest::Url) -> RequestBuilder {
        // <U: reqwest::IntoUrl>
        RequestBuilder(self.0.clone(), reqwest::Method::GET, url)
    }
}

/// A builder to construct the properties of a Request.
///
/// See [`reqwest::RequestBuilder`].
#[derive(Debug)]
pub struct RequestBuilder(reqwest::Client, reqwest::Method, reqwest::Url);
impl RequestBuilder {
    /// Constructs the Request and sends it the target URL, returning a Response.
    ///
    /// See [`reqwest::RequestBuilder::send()`].
    pub fn send(&mut self) -> impl Future<Output = reqwest::Result<Response>> + Send {
        let (client, method, url) = (self.0.clone(), self.1.clone(), self.2.clone());
        async move {
            let response = loop {
                let builder = client.request(method.clone(), url.clone());
                match builder.send().await {
                    Err(err) if !err.is_builder() && !err.is_redirect() && !err.is_status() => {
                        sleep(Duration::from_secs(1)).await
                    }
                    x => break x?,
                }
            };
            let accept_byte_ranges = if let Some(ranges) = response.headers().get(ACCEPT_RANGES) {
                ranges
                    .to_str()
                    .unwrap_or("")
                    .split(',')
                    .any(|u| u.trim() == "bytes")
            } else {
                false
            };

            Ok(Response {
                client,
                method,
                url,
                response,
                accept_byte_ranges,
                pos: 0,
            })
        }
    }
}

/// A Response to a submitted Request.
///
/// See [`reqwest::Response`].
#[derive(Debug)]
pub struct Response {
    client: reqwest::Client,
    method: reqwest::Method,
    url: reqwest::Url,
    response: reqwest::Response,
    accept_byte_ranges: bool,
    pos: u64,
}
impl Response {
    /// Convert the response into a `Stream` of `Bytes` from the body.
    ///
    /// See [`reqwest::Response::bytes_stream()`].
    pub fn bytes_stream(self) -> impl Stream<Item = reqwest::Result<Bytes>> + Send {
        Decoder {
            client: self.client,
            method: self.method,
            url: self.url,
            decoder: Box::pin(self.response.bytes_stream()),
            accept_byte_ranges: self.accept_byte_ranges,
            pos: self.pos,
        }
    }
}

struct Decoder {
    client: reqwest::Client,
    method: reqwest::Method,
    url: reqwest::Url,
    decoder: Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send + Unpin>>,
    accept_byte_ranges: bool,
    pos: u64,
}
impl Stream for Decoder {
    type Item = reqwest::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        loop {
            match ready!(self.decoder.as_mut().poll_next(cx)) {
                Some(Err(err)) => {
                    if !self.accept_byte_ranges {
                        // TODO: we could try, for those servers that don't output Accept-Ranges but work anyway
                        break Poll::Ready(Some(Err(err)));
                    }
                    let builder = self.client.request(self.method.clone(), self.url.clone());
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(
                        reqwest::header::RANGE,
                        reqwest::header::HeaderValue::from_str(&format!("bytes={}-", self.pos))
                            .unwrap(),
                    );
                    let builder = builder.headers(headers.into());
                    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Range_requests
                    // https://github.com/sdroege/gst-plugin-rs/blob/dcb36832329fde0113a41b80ebdb5efd28ead68d/gst-plugin-http/src/httpsrc.rs
                    self.decoder = Box::pin(
                        sleep(Duration::from_secs(1))
                            .then(|()| builder.send())
                            .map_ok(reqwest::Response::bytes_stream)
                            .try_flatten_stream(),
                    );
                }
                Some(Ok(n)) => {
                    self.pos += n.len() as u64;
                    break Poll::Ready(Some(Ok(n)));
                }
                None => break Poll::Ready(None),
            }
        }
    }
}

/// Shortcut method to quickly make a GET request.
///
/// See [`reqwest::get`].
pub fn get(url: reqwest::Url) -> impl Future<Output = reqwest::Result<Response>> + Send {
    // <T: IntoUrl>
    Client::new().get(url).send()
}
