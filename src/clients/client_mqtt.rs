use std::pin::Pin;
use std::task::{Context, Poll};

use log::{debug};
use futures::stream::{Stream, StreamExt};

use async_trait::async_trait;
use anyhow::Error;

use paho_mqtt::{AsyncClient, Message};

use super::{ClientBase, ClientPub, ClientSub};
use crate::TlsOptions;


/// Generic futures-based MQTT client abstraction
pub struct MqttClient {
    client: AsyncClient,
    rx: Box<dyn Stream<Item = Option<Message>> + Unpin + Send>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "structopt", derive(structopt::StructOpt))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MqttOptions {
    #[cfg_attr(feature = "structopt", structopt(long))]
    /// URL for MQTT server for base broker connection (prefixed by ssl:// or tcp://)
    pub mqtt_url: String,

    #[cfg_attr(feature = "structopt", structopt(long))]
    /// Client ID for MQTT connection
    pub mqtt_id: Option<String>,

    #[cfg_attr(feature = "structopt", structopt(flatten))]
    pub tls_opts: TlsOptions,
}

impl Into<MqttOptions> for &str {
    fn into(self) -> MqttOptions {
        MqttOptions {
            mqtt_url: self.to_string(),
            mqtt_id: None,
            tls_opts: Default::default(),
        }
    }
}


impl MqttClient {
    /// Create a new client using the provided options
    pub async fn new<O: Into<MqttOptions>>(&self, opts: O) -> Result<MqttClient, Error> {
        let o = opts.into();

        debug!("MQTT client connect opts: {:?}", o);

        // Create client with URI and ID
        let mut client_opts = paho_mqtt::CreateOptionsBuilder::new()
            .server_uri(o.mqtt_url)
            .persistence(paho_mqtt::PersistenceType::None);

        if let Some(id) = o.mqtt_id {
            client_opts = client_opts.client_id(id);
        }
            
        let mut client = AsyncClient::new(client_opts.finalize())?;

        // Setup TLS
        let mut tls_options = None;

        // Set TLS CA file if provided
        if let Some(ca_file) = &o.tls_opts.tls_ca_file {
            let mut tls_opts = paho_mqtt::SslOptionsBuilder::new();
            tls_opts.trust_store(ca_file)?;
            tls_options = Some(tls_opts);
        }
        
        // Set TLS certificate / key files if provided
        match (&mut tls_options, &o.tls_opts.tls_cert_file, &o.tls_opts.tls_key_file) {
            (Some(tls_opts), Some(cert_file), Some(key_file)) => {
                tls_opts.key_store(cert_file)?;
                tls_opts.private_key(key_file)?;
            },
            (None, Some(cert_file), Some(key_file)) => {
                let mut tls_opts = paho_mqtt::SslOptionsBuilder::new();
                tls_opts.key_store(cert_file)?;
                tls_opts.private_key(key_file)?;
                tls_options = Some(tls_opts);
            },
            (_, Some(_), None) | (_, None, Some(_)) => {
                return Err(Error::msg("TLS requires both tls-cert and tls-key arguments"))
            },
            _ => (),
        }

        // Setup connection options and connect
        let mut connect_options = paho_mqtt::ConnectOptionsBuilder::new();
        connect_options.clean_session(true);
        
        if let Some(tls_opts) = tls_options {
            connect_options.ssl_options(tls_opts.finalize());
        }

        // Connect!
        client.connect(connect_options.finalize()).await?;

        // Build incoming stream
        let rx = Box::new(client.get_stream(10));

        Ok(MqttClient{client, rx})
    }

    /// Fetch inner object for raw use
    pub fn inner<'a>(&'a mut self) -> &'a mut AsyncClient {
        &mut self.client
    }
}

#[async_trait]
impl ClientBase for MqttClient {

    #[cfg(disabled)]
    async fn connected(&mut self) -> Result<bool, Error> {
        Ok(self.client.is_connected())
    }

    async fn disconnect(&mut self) -> Result<(), Error> {
        self.client.disconnect(None).await?;
        Ok(())
    }
}

#[async_trait]
impl ClientSub for MqttClient {
    /// Subscribe to a topic
    async fn subscribe(&mut self, topic: &str) -> Result<(), Error> {
        self.client.subscribe(topic, 0).await?;
        Ok(())
    }

    /// Unsubscribe from a topic
    async fn unsubscribe(&mut self, topic: &str) -> Result<(), Error> {
        self.client.unsubscribe(topic).await?;
        Ok(())
    }
}

/// Impl stream for ClientSub
impl Stream for MqttClient {
    type Item = (String, Vec<u8>);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let m = match self.rx.poll_next_unpin(cx) {
            Poll::Ready(Some(Some(m))) => m,
            Poll::Ready(_) => return Poll::Ready(None),
            Poll::Pending => return Poll::Pending,
        };

        Poll::Ready(Some( (m.topic().to_string(), m.payload().to_vec()) ))
    }
}


#[async_trait]
impl ClientPub for MqttClient {
    /// Publish data to a topic
    async fn publish(&mut self, topic: &str, data: &[u8]) -> Result<(), Error> {
        let m = paho_mqtt::Message::new(topic, data, 0);
        self.client.publish(m).await?;
        Ok(())
    }
}
