

use async_trait::async_trait;

#[cfg(feature = "store_elastic")]
pub mod store_elastic;
#[cfg(feature = "store_elastic")]
pub use store_elastic::{ElasticStore, ElasticOptions};

#[async_trait]
pub trait Store {

}
