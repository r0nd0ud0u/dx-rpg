pub mod event;
pub mod event_inventory;
pub mod msg_from_client;
// Not public outside of this module
#[cfg(feature = "server")]
mod common_event;

pub const NO_CLIENT_ID: i64 = -1;
pub const STARTING_CLIENT_ID: i64 = 1;
