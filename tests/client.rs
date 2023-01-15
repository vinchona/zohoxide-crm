mod utils;

use crate::utils::TEST_ACCESS_TOKEN;
use zohoxide_crm::{DEFAULT_API_DOMAIN, DEFAULT_OAUTH_DOMAIN, DEFAULT_TIMEOUT};

#[test]
/// Tests client with required values
fn client_required_and_default_values() {
    let client = utils::client().build();

    assert_eq!(client.id(), utils::TEST_CLIENT_ID);
    assert_eq!(client.secret(), utils::TEST_CLIENT_SECRET);
    assert_eq!(client.refresh_token(), utils::TEST_REFRESH_TOKEN);
    assert!(client.access_token().is_none());
    assert_eq!(
        client
            .api_domain()
            .expect("Client should return the default API domain value."),
        DEFAULT_API_DOMAIN
    );
    assert_eq!(
        client
            .oauth_domain()
            .expect("Client should return the default OAuth domain value."),
        DEFAULT_OAUTH_DOMAIN
    );
    assert_eq!(client.timeout(), DEFAULT_TIMEOUT);
    assert_eq!(client.sandbox(), bool::default());
}

#[test]
/// Tests client optional values but sandbox
fn client_optional_values() {
    let optional_access_token = Some(String::from("access_token"));
    let optional_api_domain = Some(String::from("api_domain"));
    let optional_oauth_domain = Some(String::from("oauth_domain"));
    let optional_timeout: u64 = 0;

    let client = utils::client()
        .access_token(optional_access_token.clone())
        .api_domain(optional_api_domain.clone())
        .oauth_domain(optional_oauth_domain.clone())
        .timeout(optional_timeout)
        .build();

    assert_eq!(client.id(), utils::TEST_CLIENT_ID);
    assert_eq!(client.secret(), utils::TEST_CLIENT_SECRET);
    assert_eq!(client.refresh_token(), utils::TEST_REFRESH_TOKEN);
    assert_eq!(client.access_token(), optional_access_token);
    assert_eq!(client.api_domain(), optional_api_domain);
    assert_eq!(client.oauth_domain(), optional_oauth_domain);
    assert_eq!(client.timeout(), optional_timeout);
    assert_eq!(client.sandbox(), bool::default());
}

#[test]
/// Tests client sandbox optional value
fn client_sandbox_changes_api() {
    let optional_sandbox = true;
    let client = utils::client().sandbox(optional_sandbox).build();

    assert_eq!(client.id(), utils::TEST_CLIENT_ID);
    assert_eq!(client.secret(), utils::TEST_CLIENT_SECRET);
    assert_eq!(client.refresh_token(), utils::TEST_REFRESH_TOKEN);
    assert!(client.access_token().is_none());
    assert_ne!(
        client
            .api_domain()
            .expect("Client should NOT return the default API domain value."),
        DEFAULT_API_DOMAIN
    );
    assert_eq!(
        client
            .oauth_domain()
            .expect("Client should return the default OAuth domain value."),
        DEFAULT_OAUTH_DOMAIN
    );
    assert_eq!(client.timeout(), DEFAULT_TIMEOUT);
    assert_eq!(client.sandbox(), optional_sandbox);
}

#[test]
/// Tests that the `abbreviated_access_token()` method works without an access token.
fn empty_abbreviated_token() {
    assert!(utils::client().build().abbreviated_access_token().is_none());
}

#[test]
/// Tests that the `abbreviated_access_token()` method works with an access token.
fn valid_abbreviated_token() {
    let access_token = String::from("12345678901234567890");
    let client = utils::client().access_token(access_token).build();
    assert!(client.abbreviated_access_token().unwrap().len() < client.access_token().unwrap().len())
}

#[test]
/// Tests that a valid [`access_token`](struct.Client.html#method.access_token) and [`api_domai`](struct.Client.html#method.api_domain) is set when [`new_token`](struct.Client.html#method.new_token) succeeds.
fn new_token_success() {
    let setup = utils::setup(
        "POST",
        Some(&utils::oauth_body_success_response(TEST_ACCESS_TOKEN)),
    );
    let mut client = utils::client()
        .oauth_domain(mockito::server_url())
        .api_domain(None)
        .access_token(None)
        .build();

    assert!(client.access_token().is_none());
    assert!(client.api_domain().is_none());

    match client.new_token() {
        Ok(token_record) => {
            assert_eq!(token_record.access_token.unwrap(), "TEST_ACCESS_TOKEN");
            assert!(client.api_domain().is_some());
        }
        Err(error) => panic!("Bad: {:#?}", error),
    }

    utils::teardown(setup);
}

#[test]
/// Tests that [`new_token`](struct.Client.html#method.new_token) return [`ClientError::General`](enum.ClientError.html#variant.General) and set None to [`access_token`](struct.Client.html#method.access_token) and [`api_domain`](struct.Client.html#method.api_domain).
fn new_token_error_with_invalid_token() {
    let error_message = "invalid_token";
    let setup = utils::setup(
        "POST",
        Some(&utils::oauth_body_error_response(error_message)),
    );
    let mut client = utils::client()
        .oauth_domain(mockito::server_url())
        .api_domain(None)
        .access_token(None)
        .build();

    assert!(client.access_token().is_none());
    assert!(client.api_domain().is_none());

    match client.new_token() {
        Ok(_) => panic!("Error was not thrown"),
        Err(error) => {
            assert_eq!(error_message.to_string(), error.to_string());
        }
    }

    assert!(client.api_domain().is_none());
    assert!(client.access_token().is_none());
    utils::teardown(setup);
}
