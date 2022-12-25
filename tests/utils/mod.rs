use zohoxide_crm::Client;

pub const TEST_CLIENT_ID: &str = "TEST_CLIENT_ID";
pub const TEST_CLIENT_SECRET: &str = "TEST_CLIENT_SECRET";
pub const TEST_REFRESH_TOKEN: &str = "TEST_REFRESH_TOKEN";
pub fn setup() {}
pub fn teardown() {}

pub fn client() -> Client {
    Client::builder()
        .client_id(TEST_CLIENT_ID)
        .client_secret(TEST_CLIENT_SECRET)
        .refresh_token(TEST_REFRESH_TOKEN)
        .build()
}
