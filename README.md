# zohoxide-crm

Library (based on https://github.com/rideron89/zoho-crm) to help interact with v2 of the Zoho CRM API.

## Description & Examples

You can either create a client with a preset access token, or fetch a new one later on. This can be useful if you are keeping track of you access tokens in a database, for example. You will need an API client ID, secret, and refresh token.

You can read more information here:
https://www.zoho.com/crm/developer/docs/api/oauth-overview.html

To handle parsing response records, you will also need deserializable objects with `serde`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
```

## Example

```rust
use serde::Deserialize;
use zohoxide_crm::Client;

let client_id = "YOUR_CLIENT_ID";
let client_secret = "YOUR_CLIENT_SECRET";
let refresh_token = "YOUR_REFRESH_TOKEN";

let mut client = Client::builder()
    .client_id(client_id)
    .client_secret(client_secret)
    .refresh_token(refresh_token)
    .access_token(None) // optional
    .oauth_domain(Some(String::from("https://accounts.zoho.com"))) // optional
    .api_domain(Some(String::from("https://zohoapis.com"))) // optional
    .sandbox(false) // optional
    .timeout(30u64) // optional
    .build();

#[derive(Debug, Deserialize)]
struct Account {
    id: String,
    name: String,
}

let account = client.get::<Account>("Accounts", "ZOHO_ID_HERE").unwrap();
```
