mod utils;

#[test]
fn client() {
    utils::setup();
    let client = utils::client();
    assert_eq!(client.id(), utils::TEST_CLIENT_ID);
    assert_eq!(client.secret(), utils::TEST_CLIENT_SECRET);
    assert_eq!(client.refresh_token(), utils::TEST_CLIENT_REFRESH_TOKEN);
    utils::teardown();
}
