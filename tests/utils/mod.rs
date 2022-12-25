use zohoxide_crm::Client;

pub const TEST_CLIENT_ID: &str = "id";
pub const TEST_CLIENT_SECRET: &str = "secret";
pub const TEST_CLIENT_REFRESH_TOKEN: &str = "refresh_token";

pub fn setup() {}
pub fn teardown() {}

/// Build a `Client`
pub fn client() -> Client {
    Client::builder()
        .client_id(TEST_CLIENT_ID)
        .client_secret(TEST_CLIENT_SECRET)
        .refresh_token(TEST_CLIENT_REFRESH_TOKEN)
        .build()
}
