mod utils;

use zohoxide_crm::{Client, DEFAULT_API_DOMAIN, DEFAULT_OAUTH_DOMAIN, DEFAULT_TIMEOUT};

#[test]
/// Tests client with required values
fn client_required_and_default_values() {
    utils::setup();
    let required_client_id = "client_id";
    let required_client_secret = "client_secret";
    let required_refresh_token = "refresh_token";
    let client = Client::builder()
        .client_id(required_client_id)
        .client_secret(required_client_secret)
        .refresh_token(required_refresh_token)
        .build();

    assert_eq!(client.id(), required_client_id);
    assert_eq!(client.secret(), required_client_secret);
    assert_eq!(client.refresh_token(), required_refresh_token);
    assert!(client.access_token().is_none());
    assert_eq!(
        client
            .api_domain()
            .expect("Client should return an API domain default value."),
        DEFAULT_API_DOMAIN
    );
    assert_eq!(
        client
            .oauth_domain()
            .expect("Client should return an OAuth domain default value."),
        DEFAULT_OAUTH_DOMAIN
    );
    assert_eq!(client.timeout(), DEFAULT_TIMEOUT);
    assert_eq!(client.sandbox(), bool::default());
    utils::teardown();
}

#[test]
/// Tests client optional values but sandbox
fn client_optional_values() {
    utils::setup();
    let required_client_id = "client_id";
    let required_client_secret = "client_secret";
    let required_refresh_token = "refresh_token";
    let optional_access_token = Some(String::from("access_token"));
    let optional_api_domain = Some(String::from("api_domain"));
    let optional_oauth_domain = Some(String::from("oauth_domain"));
    let optional_timeout: u64 = 0;

    let client = Client::builder()
        .client_id(required_client_id)
        .client_secret(required_client_secret)
        .refresh_token(required_refresh_token)
        .access_token(optional_access_token.clone())
        .api_domain(optional_api_domain.clone())
        .oauth_domain(optional_oauth_domain.clone())
        .timeout(optional_timeout)
        .build();

    assert_eq!(client.id(), required_client_id);
    assert_eq!(client.secret(), required_client_secret);
    assert_eq!(client.refresh_token(), required_refresh_token);
    assert_eq!(client.access_token(), optional_access_token);
    assert_eq!(client.api_domain(), optional_api_domain);
    assert_eq!(client.oauth_domain(), optional_oauth_domain);
    assert_eq!(client.timeout(), optional_timeout);
    assert_eq!(client.sandbox(), bool::default());

    utils::teardown();
}

#[test]
/// Tests client sandbox optional value
fn client_sandbox_changes_api() {
    utils::setup();
    let required_client_id = "client_id";
    let required_client_secret = "client_secret";
    let required_refresh_token = "refresh_token";
    let optional_sandbox = true;

    let client = Client::builder()
        .client_id(required_client_id)
        .client_secret(required_client_secret)
        .refresh_token(required_refresh_token)
        .sandbox(optional_sandbox)
        .build();

    assert_eq!(client.id(), required_client_id);
    assert_eq!(client.secret(), required_client_secret);
    assert_eq!(client.refresh_token(), required_refresh_token);
    assert!(client.access_token().is_none());
    assert_ne!(
        client
            .api_domain()
            .expect("Client should return an API domain value which is not the default one."),
        DEFAULT_API_DOMAIN
    );
    assert_eq!(
        client
            .oauth_domain()
            .expect("Client should return an OAuth domain default value."),
        DEFAULT_OAUTH_DOMAIN
    );
    assert_eq!(client.timeout(), DEFAULT_TIMEOUT);
    assert_eq!(client.sandbox(), optional_sandbox);
    utils::teardown();
}
