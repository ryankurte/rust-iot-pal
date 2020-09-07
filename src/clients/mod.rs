
use futures::stream::Stream;
use async_trait::async_trait;

pub use anyhow::Result;


#[cfg(feature = "client_mqtt")]
pub mod client_mqtt;
#[cfg(feature = "client_mqtt")]
pub use client_mqtt::{MqttClient, MqttOptions};

#[cfg(feature = "client_coap")]
pub mod client_coap;
#[cfg(feature = "client_coap")]
pub use client_coap::{CoapClient, CoapOptions};


/// Abstract client base trait, provides connect / status / disconnect
#[async_trait]
pub trait ClientBase: Sized + Send {

    /// Disconnect a client
    async fn disconnect(&mut self) -> Result<()>;
}

/// Abstract client publish trait, allows writing data
#[async_trait]
pub trait ClientPub {
    /// Publish data to a topic / resource / endpoint
    async fn publish(&mut self, topic: &str, data: &[u8]) -> Result<()>;
}

/// Abstract client subscribe trait, allows subscription and streaming of data
#[async_trait]
pub trait ClientSub: Stream<Item = (String, Vec<u8>)> {
    /// Subscribe to a topic / resource / endpoint
    async fn subscribe(&mut self, topic: &str) -> Result<()>;

    /// Unsubscribe from a topic / resource / endpoint
    async fn unsubscribe(&mut self, topic: &str) -> Result<()>;
}


