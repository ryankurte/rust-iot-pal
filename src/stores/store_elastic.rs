
use std::fs;

use log::{debug};
use anyhow::Error;
use futures::compat::{Future01CompatExt};

use elastic::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{json};

use reqwest::{Certificate, Identity};
use reqwest::r#async::ClientBuilder as HttpClientBuilder;
use reqwest::header::{AUTHORIZATION, HeaderValue};

use crate::{TlsOptions, UserOptions};

/// Generic futures-based ElasticSearch client abstraction
pub struct ElasticStore {
    client: AsyncClient,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "structopt", derive(structopt::StructOpt))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ElasticOptions {
    #[cfg_attr(feature = "structopt", structopt(long))]
    /// URL for ElasticSearch server
    pub es_url: String,

    #[cfg_attr(feature = "structopt", structopt(flatten))]
    pub tls_opts: TlsOptions,

    #[cfg_attr(feature = "structopt", structopt(flatten))]
    pub user_opts: UserOptions,
}

impl From<&str> for ElasticOptions {
    fn from(url: &str) -> Self {
        Self {
            es_url: url.to_string(),
            tls_opts: Default::default(),
            user_opts: Default::default(),
        }
    }
}

impl From<(&str, UserOptions)> for ElasticOptions {
    fn from(o: (&str, UserOptions)) -> Self {
        Self {
            es_url: o.0.to_string(),
            tls_opts: Default::default(),
            user_opts: o.1,
        }
    }
}

impl From<(&str, TlsOptions)> for ElasticOptions {
    fn from(o: (&str, TlsOptions)) -> Self {
        Self {
            es_url: o.0.to_string(),
            tls_opts: o.1,
            user_opts: Default::default(),
        }
    }
}

impl From<(&str, UserOptions, TlsOptions)> for ElasticOptions {
    fn from(o: (&str, UserOptions, TlsOptions)) -> Self {
        Self {
            es_url: o.0.to_string(),
            tls_opts: o.2,
            user_opts: o.1,
        }
    }
}

impl ElasticStore {
    /// Create a new ElasticStore with the provided options
    pub fn new<O: Into<ElasticOptions>>(opts: O) -> Result<Self, Error> {
        let o = opts.into();

        // Setup HTTP client options
        let mut http_client_builder = HttpClientBuilder::new();

        // Load CA if provided
        if let Some(f) = &o.tls_opts.tls_ca_file {
            debug!("loading TLS CA certificate: {}", f);

            let ca = fs::read_to_string(f)?;
            let ca = Certificate::from_pem(ca.as_bytes())?;

            http_client_builder = http_client_builder.add_root_certificate(ca);
        }

        // Load client certificate and keys if provided
        match (&o.tls_opts.tls_cert_file, &o.tls_opts.tls_key_file) {
            (Some(c), Some(k)) => {
                debug!("Loading TLS client cert / key: {} {}", c, k);

                // Read files
                let mut cert = fs::read(c)?;
                let mut key = fs::read(k)?;
                key.append(&mut cert);

                let client = Identity::from_pem(&key)?;

                http_client_builder = http_client_builder.identity(client);
            },
            (Some(_), None) | (None, Some(_)) => {
                return Err(Error::msg("TLS requires both tls-cert and tls-key arguments"))
            },
            _ => (),
        }

        let http_client = http_client_builder.build().unwrap();

        // Setup Elastic client options
        let mut client_builder = AsyncClient::builder()
            .static_node(o.es_url)
            .http_client(http_client);

        // Load username / password if provided for HTTP basic auth
        match (&o.user_opts.username, &o.user_opts.password) {
            (Some(username), Some(password)) => {
                // Generate HTTP basic auth header
                let v = format!("Basic {}", base64::encode(&format!("{}:{}", username, password)));
                let auth = HeaderValue::from_str(&v).unwrap();

                client_builder = client_builder.params_fluent(move |p| p.header(AUTHORIZATION, auth.clone()));
            },
            (Some(_), None) | (None, Some(_)) => {
                return Err(Error::msg("User auth requires both username and password arguments"))
            },
            _ => (),
        }

        // Build client
        let client = client_builder.build().unwrap();
           
        Ok(Self {
            client,
        })
    }

    /// Fetch inner client for direct use
    pub fn inner<'a>(&'a mut self) -> &'a mut AsyncClient {
        &mut self.client
    }


    /// Store a record in the database
    pub async fn store<R: DocumentType + Serialize + Send + 'static>(&mut self, record: R) -> Result<(), Error> {
        self.client.document().index(record).send().compat().await.unwrap();

        Ok(())
    }


    /// Search for records matching the provided JSON query
    pub async fn search<Q: Serialize + Send, R: DocumentType + DeserializeOwned + Send + 'static>(&mut self, query: Q) -> Result<Vec<R>, Error> {
        // Encode query
        let q = serde_json::to_string(&query)?;

        // Issue request
        let resp = self.client.search::<R>().body(q).send().compat().await.unwrap();

        // Parse out response
        let docs: Vec<_> = resp.into_documents().collect();

        Ok(docs)
    }

    /// Create an index for the provided document on the specified index
    pub async fn map<T: DocumentType>(&mut self, index: &str) -> Result<(), Error> {
        let doc = T::index_mapping();
        let mapping = serde_json::to_string(&doc).unwrap();

        let i = index.to_string();
        let body = json!({
            "mappings": {
                &i: mapping,
            }
        });

        self.client.index(i.clone()).create().send().compat().await.unwrap();

        let req = elastic::endpoints::IndicesPutMappingRequest::for_index(i.clone(), body);
        self.client.request(req).send().compat().await.unwrap();

        Ok(())
    }
}
