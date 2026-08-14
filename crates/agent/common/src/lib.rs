//! # `cosplay-agent` - Common utilities shared between `cosplay-client` and `cosplay-server`.

mod agent_marker;
pub use agent_marker::{Agent, Client, Server};

mod new_id_factory;
pub use new_id_factory::NewObjectIdFactory;
