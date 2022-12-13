//! # zohoxide-crm
//!
//! Library to help interact with v2 of the Zoho CRM API.
//!
//! You can read more information about the Zoho API here:
//! [https://www.zoho.com/crm/developer/docs/api/oauth-overview.html](https://www.zoho.com/crm/developer/docs/api/oauth-overview.html)
//!
//! To handle parsing response records, you will also need deserializable objects with `serde`:
//!
//! ```toml
//! [dependencies]
//! serde = { version = "1.0", features = ["derive"] }
//! ```
//!
//! ### Example
//!
//! ```no_run
//! use serde::Deserialize;
//! use zohoxide_crm::Client;
//!
//! let client_id = String::from("YOUR_CLIENT_ID");
//! let client_secret = String::from("YOUR_CLIENT_SECRET");
//! let refresh_token = String::from("YOUR_REFRESH_TOKEN");
//!
//! let mut client = Client::builder()
//!     .client_id(client_id)
//!     .client_secret(client_secret)
//!     .refresh_token(refresh_token)
//!     .access_token(None) // optional
//!     .api_domain(None) // optional
//!     .build();
//!
//! #[derive(Debug, Deserialize)]
//! struct Account {
//!     id: String,
//!     name: String,
//! }
//!
//! let account = client.get::<Account>("Accounts", "ZOHO_ID_HERE").unwrap();
//! ```

extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate serde_urlencoded;

mod client;
mod client_error;
pub mod response;
mod token_record;

pub use client::parse_params;
pub use client::Client;
pub use client::ClientBuilder;
pub use client_error::ClientError;
pub use token_record::TokenRecord;
