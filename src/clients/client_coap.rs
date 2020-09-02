use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::{Stream, StreamExt};
use async_trait::async_trait;
use anyhow::Error;

use coap::client::{CoAPClientAsync, CoAPObserverAsync, RequestOptions};

use super::{ClientBase, ClientPub, ClientSub};
use crate::TlsOptions;

/// Generic futures-based CoAP client abstraction
pub struct CoapClient {
    client: CoAPClientAsync<tokio::net::UdpSocket>,
    subs: Vec<CoAPObserverAsync>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "structopt", derive(structopt::StructOpt))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CoapOptions {
    #[cfg_attr(feature = "structopt", structopt(long))]
    /// URL for CoAP server
    pub coap_url: String,

    #[cfg_attr(feature = "structopt", structopt(flatten))]
    pub tls_opts: TlsOptions,
}

impl Into<CoapOptions> for &str {
    fn into(self) -> CoapOptions {
        CoapOptions {
            coap_url: self.to_string(),
            tls_opts: TlsOptions::default(),
        }
    }
}


impl CoapClient {
    /// Create a new client using the provided driver
    pub async fn new<O: Into<CoapOptions>>(&self, opts: O) -> Result<CoapClient, Error> {
        let o = opts.into();

        // TODO: parse out URI opts for underlying driver
        let client = CoAPClientAsync::new_udp(o.coap_url).await?;

        Ok(CoapClient{client, subs: vec![]})
    }

    /// Fetch inner object for raw use
    pub fn inner<'a>(&'a mut self) -> &'a mut CoAPClientAsync<tokio::net::UdpSocket> {
        &mut self.client
    }
}


#[async_trait]
impl ClientBase for CoapClient {

    /// Disconnect from client
    async fn disconnect(&mut self) -> Result<(), Error> {
        // Remove observations
        for s in self.subs.drain(..) {
            self.client.unobserve(s).await?;
        }

        Ok(())
    }
}


#[async_trait]
impl ClientSub for CoapClient {

    /// Subscribe to a topic
    async fn subscribe(&mut self, topic: &str) -> Result<(), Error> {
        let observer = self.client.observe(topic, &RequestOptions::default()).await?;
        self.subs.push(observer);

        Ok(())
    }

    /// Unsubscribe from a topic
    async fn unsubscribe(&mut self, topic: &str) -> Result<(), Error> {
        let observer = self.client.observe(topic, &RequestOptions::default()).await?;
        self.subs.push(observer);

        Ok(())
    }
}

/// Stream implementation for CoapSub
impl Stream for CoapClient {
    type Item = Vec<u8>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        for s in &mut self.subs {
            let m = match s.poll_next_unpin(cx) {
                Poll::Ready(Some(m)) => m,
                Poll::Ready(_) => return Poll::Ready(None),
                Poll::Pending => continue,
            };

            return Poll::Ready(Some(m.message.payload))
        }

        Poll::Pending
    }
}

#[async_trait]
impl ClientPub for CoapClient {
    /// Publish data to a topic
    async fn publish(&mut self, topic: &str, data: &[u8]) -> Result<(), Error> {
        self.client.put(topic, data, &RequestOptions::default()).await?;
        Ok(())
    }
}
