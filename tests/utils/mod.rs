use mockito::{mock, Matcher, Mock};
use zohoxide_crm::{Client, ClientBuilder, DEFAULT_API_DOMAIN};

pub const TEST_CLIENT_ID: &str = "TEST_CLIENT_ID";
pub const TEST_CLIENT_SECRET: &str = "TEST_CLIENT_SECRET";
pub const TEST_REFRESH_TOKEN: &str = "TEST_REFRESH_TOKEN";
const TEST_ACCESS_TOKEN: &str = "TEST_ACCESS_TOKEN";

/// Mocked Zoho's OAuth success body response
pub fn oauth_body_success_response() -> String {
    format!(
        r#"{{"access_token":"{}","expires_in_sec":3600,"api_domain":"{}","token_type":"Bearer","expires_in":3600000}}"#,
        TEST_ACCESS_TOKEN, DEFAULT_API_DOMAIN
    )
}

/// Mocked Zoho's OAuth error body response
pub fn oauth_body_error_response(error_message: &str) -> String {
    format!(r#"{{"error":"{}"}}"#, error_message)
}

/// Setup a mocker to handle request
pub fn setup(method: &str, body: Option<&str>) -> Mock {
    let mut mocker = mock(method, Matcher::Any)
        .with_status(200)
        .with_header("Content-Type", "application/json;charset=UTF-8");

    if let Some(body) = body {
        mocker = mocker
            .with_header("Content-Length", &body.to_string().len().to_string())
            .with_body(body);
    }

    mocker.create()
}

/// Ensure mocker has been called once
pub fn teardown(mocker: Mock) {
    mocker.assert()
}

/// Get a `ClientBuilder` with required parameters already set
pub fn client() -> ClientBuilder<((String,), (String,), (String,), (), (), (), (), ())> {
    Client::builder()
        .client_id(TEST_CLIENT_ID)
        .client_secret(TEST_CLIENT_SECRET)
        .refresh_token(TEST_REFRESH_TOKEN)
}
