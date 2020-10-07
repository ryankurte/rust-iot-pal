//! IoT Protocol Abstraction Library

use std::path::Path;

use anyhow::Error;

pub mod clients;

pub mod stores;


/// General TLS Configuration options
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "structopt", derive(structopt::StructOpt))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TlsOptions {
    #[cfg_attr(feature = "structopt", structopt(long, env))]
    /// TLS Certiciate Authority (CA) file in PEM format
    pub tls_ca_file: Option<String>,

    #[cfg_attr(feature = "structopt", structopt(long, env))]
    /// TLS client certificate file in PEM format
    pub tls_cert_file: Option<String>,

    #[cfg_attr(feature = "structopt", structopt(long, env))]
    /// TLS client key file in PEM format
    pub tls_key_file: Option<String>,
}

impl Default for TlsOptions {
    fn default() -> Self {
        Self {
            tls_ca_file: None,
            tls_cert_file: None,
            tls_key_file: None,
        }
    }
}

impl TlsOptions {
    pub fn validate(&self) -> Result<(), anyhow::Error> {

        match &self.tls_ca_file {
            Some(f) if !Path::new(f).exists() => {
                return Err(Error::msg(format!("Could not access TLS CA file: {}", f)))
            }
            _ => (),
        }

        match &self.tls_cert_file {
            Some(f) if !Path::new(f).exists() => {
                return Err(Error::msg(format!("Could not access TLS cert file: {}", f)))
            }
            _ => (),
        }

        match &self.tls_key_file {
            Some(f) if !Path::new(f).exists() => {
                return Err(Error::msg(format!("Could not access TLS key file: {}", f)))
            }
            _ => (),
        }

        Ok(())
    }
}

/// General User (username / password) configuration options
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "structopt", derive(structopt::StructOpt))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserOptions {
    #[cfg_attr(feature = "structopt", structopt(long, env))]
    /// Username for connection
    pub username: Option<String>,

    #[cfg_attr(feature = "structopt", structopt(long, env))]
    /// Password for connection
    pub password: Option<String>,
}

impl Default for UserOptions {
    fn default() -> Self {
        Self {
            username: None,
            password: None,
        }
    }
}

