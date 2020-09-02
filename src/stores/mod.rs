

#[cfg(feature = "store_elastic")]
pub mod store_elastic;
#[cfg(feature = "store_elastic")]
pub use store_elastic::{ElasticStore, ElasticOptions};

