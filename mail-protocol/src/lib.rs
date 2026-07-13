pub mod backend;
pub mod error;
pub mod imap;
pub mod smtp;

pub use backend::*;
pub use error::MailError;
