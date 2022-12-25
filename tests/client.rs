mod utils;

use zohoxide_crm::{DEFAULT_API_DOMAIN, DEFAULT_OAUTH_DOMAIN, DEFAULT_TIMEOUT};

#[test]
fn client() {
    utils::setup();
    let client = utils::client();
    assert_eq!(client.id(), utils::TEST_CLIENT_ID);
    assert_eq!(client.secret(), utils::TEST_CLIENT_SECRET);
    assert_eq!(client.refresh_token(), utils::TEST_CLIENT_REFRESH_TOKEN);
    assert!(client.access_token().is_none());
    assert_eq!(
        client
            .api_domain()
            .expect("Client should return an API  default value"),
        DEFAULT_API_DOMAIN
    );
    assert_eq!(
        client
            .oauth_domain()
            .expect("Client should return an OAuth domain default value"),
        DEFAULT_OAUTH_DOMAIN
    );
    assert_eq!(client.timeout(), DEFAULT_TIMEOUT);
    assert_eq!(client.sandbox(), bool::default());
    utils::teardown();
}
