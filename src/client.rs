use crate::client_error::ClientError;
use crate::response;
use crate::token_record::TokenRecord;

use std::collections::HashMap;
use std::time::Duration;
use typed_builder::TypedBuilder;

/// Default network timeout for API requests.
pub const DEFAULT_TIMEOUT: u64 = 30;
/// Default OAuth domain for client token.
pub const DEFAULT_OAUTH_DOMAIN: &str = "https://accounts.zoho.com";
/// Default API domain for requests.
pub const DEFAULT_API_DOMAIN: &str = "https://www.zohoapis.com";

/// Handles making requests to v2 of the Zoho CRM API.
///
/// You can either create a client with a preset access token, or fetch a new one later on.
/// This can be useful if you are keeping track of you access tokens in a database, for example. You will need an API client ID, secret, and refresh token.
///
/// You can read more information here:
/// [https://www.zoho.com/crm/developer/docs/api/oauth-overview.html](https://www.zoho.com/crm/developer/docs/api/oauth-overview.html)
///
/// ### Example
///
/// You should create a [`Client`](struct.Client) with the [`builder()`](struct.Client.html#method.builder) method.
///
/// ```
/// use zohoxide_crm::Client;
///
/// let client_id = "YOUR_CLIENT_ID";
/// let client_secret = "YOUR_CLIENT_SECRET";
/// let refresh_token = "YOUR_REFRESH_TOKEN";
///
/// let client = Client::builder()
///     .client_id(client_id)
///     .client_secret(client_secret)
///     .refresh_token(refresh_token)
///     .access_token(None) // optional
///     .oauth_domain(None) // optional
///     .api_domain(None) // optional
///     .sandbox(false) // optional
///     .timeout(30u64) // optional
///     .build();
///
/// ```
///
/// API methods will automatically fetch a new token if one has not been set. This token is then
/// saved internally to be used on all future requests.
#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(TypedBuilder)]
#[builder(doc, field_defaults(setter(into)))]
pub struct Client {
    client_id: String,
    client_secret: String,
    refresh_token: String,
    #[builder(default)]
    access_token: Option<String>,
    #[builder(default = Some(String::from(DEFAULT_OAUTH_DOMAIN)))]
    oauth_domain: Option<String>,
    #[builder(default = Some(String::from(DEFAULT_API_DOMAIN)))]
    api_domain: Option<String>,
    #[builder(default)]
    sandbox: bool,
    #[builder(default = DEFAULT_TIMEOUT)]
    timeout: u64,
}

impl Client {
    /// Get client id.
    pub fn id(&self) -> String {
        self.client_id.clone()
    }

    /// Get client secret.
    pub fn secret(&self) -> String {
        self.client_secret.clone()
    }

    /// Get client refresh_token.
    pub fn refresh_token(&self) -> String {
        self.refresh_token.clone()
    }

    /// Get the sandbox configuration.
    pub fn sandbox(&self) -> bool {
        self.sandbox
    }

    /// Get the timeout (in seconds) for API requests.
    pub fn timeout(&self) -> u64 {
        self.timeout
    }

    /// Get the access token.
    pub fn access_token(&self) -> Option<String> {
        self.access_token.clone()
    }

    /// Get the API domain URL.
    pub fn api_domain(&self) -> Option<String> {
        if self.sandbox() {
            Some(String::from("https://crmsandbox.zoho.com"))
        } else {
            self.api_domain.clone()
        }
    }

    /// Get the OAuth domain URL.
    pub fn oauth_domain(&self) -> Option<String> {
        self.oauth_domain.clone()
    }

    /// Get an abbreviated version of the access token. This is a (slightly) safer version
    /// of the access token should you need to print it out.
    ///
    /// ```
    /// # use zohoxide_crm::Client;
    /// let token = "1000.ad8f97a9sd7f9a7sdf7a89s7df87a9s8.a77fd8a97fa89sd7f89a7sdf97a89df3";
    /// # let client_id = "YOUR_CLIENT_ID";
    /// # let client_secret = "YOUR_CLIENT_SECRET";
    /// # let refresh_token = "YOUR_REFRESH_TOKEN";
    ///
    /// let mut client = Client::builder()
    ///  .access_token(Some(String::from(token)))
    ///  .client_id(client_id)
    ///  .client_secret(client_secret)
    ///  .refresh_token(refresh_token)
    ///  .build();
    ///
    /// assert_eq!("1000.ad8f..9df3", &client.abbreviated_access_token().unwrap());
    /// ```
    pub fn abbreviated_access_token(&self) -> Option<String> {
        match &self.access_token {
            Some(access_token) => {
                let prefix = &access_token[0..9];
                let suffix = &access_token.chars().rev().collect::<String>()[0..4]
                    .chars()
                    .rev()
                    .collect::<String>();
                let abbreviated_token = format!("{}..{}", prefix, suffix);

                Some(abbreviated_token)
            }
            None => None,
        }
    }
}

impl Client {
    /// Get the API base path, which changes depending on the current environment.
    ///
    /// This is primarily used to allow for HTTP test mocking of API calls.
    fn get_oauth_domain(&self) -> String {
        #[cfg(test)]
        return mockito::server_url();

        #[cfg(not(test))]
        match &self.oauth_domain {
            Some(oauth_domain) => oauth_domain.clone(),
            None => String::from("https://accounts.zoho.com"),
        }
    }

    /// Get a new access token from Zoho. Guarantees an access token when it returns
    /// an `Result::Ok`.
    ///
    /// The access token is saved to the [`Client`](struct.Client), so you don't
    /// need to retrieve the token and set it in different steps. But a copy
    /// of it is returned by this method.
    pub fn get_new_token(&mut self) -> Result<TokenRecord, ClientError> {
        let url = format!(
            "{}/oauth/v2/token?grant_type=refresh_token&client_id={}&client_secret={}&refresh_token={}",
            self.get_oauth_domain(),
            self.client_id,
            self.client_secret,
            self.refresh_token
        );

        let client = reqwest::Client::new();
        let mut response = client.post(url.as_str()).send()?;
        let raw_response = response.text()?;

        // TODO: refactor this with a more idiomatic pattern
        if let Ok(response) = serde_json::from_str::<response::AuthErrorResponse>(&raw_response) {
            return Err(ClientError::General(response.error));
        }

        let api_response: TokenRecord = serde_json::from_str(&raw_response)?;

        self.access_token = api_response.access_token.clone();
        self.api_domain = api_response.api_domain.clone();

        match &self.access_token {
            Some(_) => Ok(api_response),
            None => Err(ClientError::from("No token received")),
        }
    }

    /// Fetches a record from Zoho.
    ///
    /// Zoho returns a data array with this method, even though that array will always be of
    /// length-1. We return the data array, so you must treat the response accordingly.
    ///
    /// If an error occurred, and we are given a response code back, this method will return a
    /// [`ClientError::ApiError`](enum.ClientError.html#variant.ApiError) with the response code
    /// and message. Otherwise, a [`ClientError::General`](enum.ClientError.html#variant.General)
    /// error will be returned with the raw response text.
    ///
    /// ### Example
    ///
    /// ```no_run
    /// # use serde::Deserialize;
    /// # use std::collections::HashMap;
    /// use zohoxide_crm::Client;
    ///
    /// #[derive(Deserialize)]
    /// struct Account {
    ///     name: String,
    /// }
    ///
    /// # let client_id = "";
    /// # let client_secret = "";
    /// # let refresh_token = "";
    /// let mut client = Client::builder()
    /// .client_id(client_id)
    /// .client_secret(client_secret)
    /// .refresh_token(refresh_token)
    /// .build();
    ///
    /// let response = client.get::<Account>("Accounts", "ZOHO_ID_HERE").unwrap();
    ///
    /// let account = response.data.get(0).unwrap();
    /// assert_eq!(account.name, "Account name");
    /// ```
    pub fn get<T: serde::de::DeserializeOwned>(
        &mut self,
        module: &str,
        id: &str,
    ) -> Result<response::ApiGetResponse<T>, ClientError> {
        if self.access_token.is_none() {
            self.get_new_token()?;
        }

        // we are guaranteed a token when we reach this line
        let token = self.access_token.clone().unwrap();

        let timeout = Duration::from_secs(self.timeout);
        let client = reqwest::Client::builder().timeout(timeout).build()?;

        let url = format!("{}/crm/v2/{}/{}", self.api_domain().unwrap(), module, id);

        let mut response = client
            .get(url.as_str())
            .header("Authorization", format!("Zoho-oauthtoken {}", token))
            .send()?;
        let raw_response = response.text()?;

        if let Ok(response) = serde_json::from_str::<response::ApiErrorResponse>(&raw_response) {
            return Err(ClientError::ApiError(response));
        }

        match serde_json::from_str::<response::ApiGetResponse<T>>(&raw_response) {
            Ok(data) => Ok(data),
            Err(_) => {
                if !raw_response.is_empty() {
                    Err(ClientError::UnexpectedResponseType(raw_response))
                } else {
                    Err(ClientError::EmptyResponse)
                }
            }
        }
    }

    /// Fetches a page of records from Zoho.
    ///
    /// Zoho API function documentation:
    /// [https://www.zoho.com/crm/developer/docs/api/get-records.html](https://www.zoho.com/crm/developer/docs/api/get-records.html)
    ///
    /// ### Example
    ///
    /// ```no_run
    /// # use serde::Deserialize;
    /// # use std::collections::HashMap;
    /// use zohoxide_crm::Client;
    ///
    /// #[derive(Deserialize)]
    /// struct Account {
    ///     name: String,
    /// }
    ///
    /// # let client_id = "";
    /// # let client_secret = "";
    /// # let refresh_token = "";
    /// let mut client = Client::builder()
    /// .client_id(client_id)
    /// .client_secret(client_secret)
    /// .refresh_token(refresh_token)
    /// .build();
    ///
    /// let accounts = client.get_many::<Account>("Accounts", None).unwrap();
    /// ```
    ///
    /// ### Example with parameters
    ///
    /// ```no_run
    /// # use serde::Deserialize;
    /// # use std::collections::HashMap;
    /// use zohoxide_crm::{parse_params, Client};
    ///
    /// #[derive(Deserialize)]
    /// struct Account {
    ///     name: String,
    /// }
    ///
    /// # let client_id = "";
    /// # let client_secret = "";
    /// # let refresh_token = "";
    ///
    /// # let mut client = Client::builder()
    /// .client_id(client_id)
    /// .client_secret(client_secret)
    /// .refresh_token(refresh_token)
    /// .build();
    ///
    /// let mut params: HashMap<&str, &str> = HashMap::new();
    /// params.insert("cvid", "YOUR_VIEW_ID_HERE");
    /// params.insert("page", "2");
    /// params.insert("per_page", "50");
    ///
    /// let params = parse_params(params).unwrap();
    /// let accounts = client.get_many::<Account>("Accounts", Some(params)).unwrap();
    /// ```
    pub fn get_many<T: serde::de::DeserializeOwned>(
        &mut self,
        module: &str,
        params: Option<String>,
    ) -> Result<response::ApiGetManyResponse<T>, ClientError> {
        if self.access_token.is_none() {
            self.get_new_token()?;
        }

        // we are guaranteed a token when we reach this line
        let token = self.access_token().unwrap();
        let api_domain = self.api_domain().unwrap();

        let timeout = Duration::from_secs(self.timeout);
        let client = reqwest::Client::builder().timeout(timeout).build()?;

        let mut url = format!("{}/crm/v2/{}", api_domain, module);

        if params.is_some() {
            url = url + &format!("?{}", params.unwrap());
        }

        let mut response = client
            .get(url.as_str())
            .header("Authorization", String::from("Zoho-oauthtoken ") + &token)
            .send()?;
        let raw_response = response.text()?;

        if let Ok(response) = serde_json::from_str::<response::ApiErrorResponse>(&raw_response) {
            return Err(ClientError::ApiError(response));
        }

        match serde_json::from_str::<response::ApiGetManyResponse<T>>(&raw_response) {
            Ok(data) => Ok(data),
            Err(_) => {
                if !raw_response.is_empty() {
                    Err(ClientError::UnexpectedResponseType(raw_response))
                } else {
                    Err(ClientError::EmptyResponse)
                }
            }
        }
    }

    /// Insert multiple records in Zoho.
    ///
    /// Zoho API function documentation:
    /// [https://www.zoho.com/crm/developer/docs/api/insert-records.html](https://www.zoho.com/crm/developer/docs/api/insert-records.html)
    ///
    /// It is important to note that this method *may* mask errors with a successful response.
    /// That is because record specific errors will be shown alongside the record in the response.
    /// We do not want to assume this is an *unsuccessful* response, and so it is up to you to
    /// handle them.
    ///
    /// The `params` argument accepts any serializable data type.
    ///
    /// ```no_run
    /// # use std::collections::HashMap;
    /// # use zohoxide_crm::Client;
    /// # let client_id = "";
    /// # let client_secret = "";
    /// # let refresh_token = "";
    /// # let mut zoho_client = Client::builder()
    /// .client_id(client_id)
    /// .client_secret(client_secret)
    /// .refresh_token(refresh_token)
    /// .build();
    ///
    /// let mut record: HashMap<&str, &str> = HashMap::new();
    /// record.insert("name", "sample");
    ///
    /// let response = zoho_client.insert("Accounts", vec![record]).unwrap();
    ///
    /// for record in response.data {
    ///     match record.code.as_str() {
    ///         "SUCCESS" => println!("Record was successful"),
    ///         _ => println!("Record was NOT successful"),
    ///     }
    /// }
    /// ```
    pub fn insert<T>(
        &mut self,
        module: &str,
        data: Vec<T>,
    ) -> Result<response::ApiSuccessResponse, ClientError>
    where
        T: serde::ser::Serialize,
    {
        if self.access_token.is_none() {
            self.get_new_token()?;
        }

        // we are guaranteed a token when we reach this line
        let token = self.access_token().unwrap();
        let api_domain = self.api_domain().unwrap();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.timeout))
            .build()?;

        let url = format!("{}/crm/v2/{}", api_domain, module);

        // Zoho requires incoming data to be sent via a `data` field
        let mut params: HashMap<&str, Vec<T>> = HashMap::new();
        params.insert("data", data);

        let mut response = client
            .post(url.as_str())
            .header("Authorization", String::from("Zoho-oauthtoken ") + &token)
            .json(&params)
            .send()?;
        let raw_response = response.text()?;

        if let Ok(response) = serde_json::from_str::<response::ApiErrorResponse>(&raw_response) {
            return Err(ClientError::ApiError(response));
        }

        match serde_json::from_str::<response::ApiSuccessResponse>(&raw_response) {
            Ok(response) => Ok(response),
            Err(_) => {
                if !raw_response.is_empty() {
                    Err(ClientError::UnexpectedResponseType(raw_response))
                } else {
                    Err(ClientError::EmptyResponse)
                }
            }
        }
    }

    /// Updates multiple records in Zoho.
    ///
    /// Zoho API function documentation:
    /// [https://www.zoho.com/crm/developer/docs/api/update-records.html](https://www.zoho.com/crm/developer/docs/api/update-records.html)
    ///
    /// It is important to note that this method *may* mask errors with a successful response.
    /// That is because record specific errors will be shown alongside the record in the response.
    /// We do not want to assume this is an *unsuccessful* response, and so it is up to you to
    /// handle them.
    ///
    /// The `params` argument accepts any serializable data type.
    ///
    /// ```no_run
    /// # use std::collections::HashMap;
    /// # use zohoxide_crm::Client;
    /// # let client_id = "";
    /// # let client_secret = "";
    /// # let refresh_token = "";
    /// # let mut zoho_client = Client::builder()
    /// .client_id(client_id)
    /// .client_secret(client_secret)
    /// .refresh_token(refresh_token)
    /// .build();
    ///
    /// let mut record: HashMap<&str, &str> = HashMap::new();
    /// record.insert("id", "ZOHO_RECORD_ID_HERE");
    /// record.insert("name", "sample");
    ///
    /// let response = zoho_client.update_many("Accounts", vec![record]).unwrap();
    ///
    /// for record in response.data {
    ///     match record.code.as_str() {
    ///         "SUCCESS" => println!("Record was successful"),
    ///         _ => println!("Record was NOT successful"),
    ///     }
    /// }
    /// ```
    pub fn update_many<T>(
        &mut self,
        module: &str,
        data: Vec<T>,
    ) -> Result<response::ApiSuccessResponse, ClientError>
    where
        T: serde::ser::Serialize,
    {
        if self.access_token.is_none() {
            self.get_new_token()?;
        }

        // we are guaranteed a token when we reach this line
        let token = self.access_token().unwrap();
        let api_domain = self.api_domain().unwrap();

        let timeout = Duration::from_secs(self.timeout);
        let client = reqwest::Client::builder().timeout(timeout).build()?;

        let url = format!("{}/crm/v2/{}", api_domain, module);

        // Zoho requires incoming data to be sent via a `data` field
        let mut params: HashMap<&str, Vec<T>> = HashMap::new();
        params.insert("data", data);

        let mut response = client
            .put(url.as_str())
            .header("Authorization", String::from("Zoho-oauthtoken ") + &token)
            .json(&params)
            .send()?;
        let raw_response = response.text()?;

        if let Ok(response) = serde_json::from_str::<response::ApiErrorResponse>(&raw_response) {
            return Err(ClientError::ApiError(response));
        }

        match serde_json::from_str::<response::ApiSuccessResponse>(&raw_response) {
            Ok(response) => Ok(response),
            Err(_) => {
                if !raw_response.is_empty() {
                    Err(ClientError::UnexpectedResponseType(raw_response))
                } else {
                    Err(ClientError::EmptyResponse)
                }
            }
        }
    }
}

/// Utility function to help a parameter list into a URL-encoded string.
///
/// This should be passed into any method that supports URL-encoded parameters, such as
/// [`get_many`](struct.Client.html#method.get_many).
///
/// ### Example
///
/// ```no_run
/// # use serde::Deserialize;
/// # use std::collections::HashMap;
/// # use zohoxide_crm::{parse_params, Client};
/// # #[derive(Deserialize)]
/// # struct Record {
/// #     id: String,
/// # }
/// let mut client = Client::builder()
/// .client_id("")
/// .client_secret("")
/// .refresh_token("")
/// .build();
///
/// let mut params: HashMap<&str, &str> = HashMap::new();
/// params.insert("page", "2");
///
/// let params = parse_params(params).unwrap();
/// assert_eq!("page=2", &params);
///
/// client.get_many::<Record>("Accounts", Some(params)).unwrap();
/// ```
#[allow(dead_code)]
pub fn parse_params(
    params: impl serde::ser::Serialize,
) -> Result<String, serde_urlencoded::ser::Error> {
    serde_urlencoded::to_string(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    use mockito::{mock, Matcher, Mock};
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Debug, Deserialize)]
    struct ResponseRecord {
        id: String,
    }

    /// Get a `Client` with an access token.
    fn get_client(
        access_token: Option<String>,
        oauth_domain: Option<String>,
        api_domain: Option<String>,
    ) -> Client {
        let id = "id";
        let secret = "secret";
        let refresh_token = "refresh_token";

        Client::builder()
            .access_token(access_token)
            .oauth_domain(oauth_domain)
            .api_domain(api_domain)
            .client_id(id)
            .client_secret(secret)
            .refresh_token(refresh_token)
            .build()
    }

    /// Get an HTTP mocker.
    fn get_mocker<T: Into<Matcher>>(method: &str, url: T, body: Option<&str>) -> Mock {
        let mut mocker = mock(method, url)
            .with_status(200)
            .with_header("Content-Type", "application/json;charset=UTF-8");

        if let Some(body) = body {
            mocker = mocker
                .with_header("Content-Length", &body.to_string().len().to_string())
                .with_body(body);
        }

        mocker = mocker.create();

        mocker
    }

    #[test]
    /// Tests that an error is return after calling the `Client` `get_new_token()` method with an
    /// invalid refresh token.
    fn get_new_token_invalid_token() {
        let error_message = "invalid_token";
        let body = format!(r#"{{"error":"{}"}}"#, error_message);
        let mocker = get_mocker("POST", Matcher::Any, Some(&body));
        let mut client = get_client(None, None, None);

        match client.get_new_token() {
            Ok(_) => panic!("Error was not thrown"),
            Err(error) => {
                assert_eq!(error_message.to_string(), error.to_string());
            }
        }

        mocker.assert();
    }

    #[test]
    /// Tests that a `TokenRecord` with a valid access token is returned from the `Client`
    /// `get_new_token()` method.
    fn return_new_token_success() {
        let access_token = "9999.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let api_domain = "https://www.zohoapis.com";
        let body = format!(
            r#"{{"access_token":"{}","expires_in_sec":3600,"api_domain":"{}","token_type":"Bearer","expires_in":3600000}}"#,
            access_token, api_domain
        );
        let mocker = get_mocker("POST", Matcher::Any, Some(&body));
        let mut client = get_client(None, None, None);

        let token = client.get_new_token().unwrap();

        mocker.assert();
        assert_eq!(token.access_token, Some(String::from(access_token)));
    }

    #[test]
    /// Tests that a `TokenRecord` with a valid API domain is returned from the `Client`
    /// `get_new_token()` method.
    fn return_api_domain_success() {
        let access_token = "9999.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let api_domain = "https://www.zohoapis.com";
        let body = format!(
            r#"{{"access_token":"{}","expires_in_sec":3600,"api_domain":"{}","token_type":"Bearer","expires_in":3600000}}"#,
            access_token, api_domain
        );
        let mocker = get_mocker("POST", Matcher::Any, Some(&body));
        let mut client = get_client(None, None, None);

        let token = client.get_new_token().unwrap();

        mocker.assert();
        assert_eq!(token.api_domain, Some(String::from(api_domain)));
    }

    #[test]
    /// Tests that fetching a record via the `get()` method works.
    fn get_success() {
        let access_token = "9999.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let api_domain = mockito::server_url();
        let record_id = "40000000123456789";
        let body = format!(
            r#"{{"data":[{{"id":"{}"}}],"info":{{"more_records":true,"per_page":1,"count":1,"page":1}}}}"#,
            record_id
        );
        let mocker = get_mocker("GET", Matcher::Any, Some(&body));
        let mut client = get_client(Some(String::from(access_token)), None, Some(api_domain));

        let response = client.get::<ResponseRecord>("Accounts", record_id).unwrap();

        mocker.assert();
        assert_eq!(response.data.get(0).unwrap().id, record_id);
    }

    #[test]
    /// Tests that an error code returned via the `get()` method returns an error.
    fn get_regular_error() {
        let access_token = "9999.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let api_domain = mockito::server_url();
        let error_code = "INVALID_URL_PATTERN";
        let body = format!(
            r#"{{"code":"{}","details":{{}},"message":"Please check if the URL trying to access is a correct one","status":"error"}}"#,
            error_code
        );
        let mocker = get_mocker("GET", Matcher::Any, Some(&body));
        let mut client = get_client(Some(String::from(access_token)), None, Some(api_domain));

        match client.get::<ResponseRecord>("INVALID_MODULE", "00000") {
            Ok(_) => panic!("Response did not return an error"),
            Err(err) => match err {
                ClientError::ApiError(error) => assert_eq!(error.code, error_code),
                _ => panic!("Wrong error type"),
            },
        }

        mocker.assert();
    }

    #[test]
    /// Tests that a plain error message returned via the `get()` method returns an error.
    fn get_text_error() {
        let access_token = "9999.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let api_domain = mockito::server_url();
        let error_code = "invalid_client";
        let body = error_code.to_string();
        let mocker = get_mocker("GET", Matcher::Any, Some(&body));
        let mut client = get_client(Some(String::from(access_token)), None, Some(api_domain));

        match client.get::<ResponseRecord>("INVALID_MODULE", "00000") {
            Ok(_) => panic!("Response did not return an error"),
            Err(err) => {
                assert_eq!(err.to_string(), error_code.to_string());
            }
        }

        mocker.assert();
    }

    #[test]
    /// Tests that inserting a record via the `insert()` method works.
    fn insert_many_success() {
        let access_token = "9999.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let api_domain = mockito::server_url();
        let record_id = "40000000123456789";
        let body = format!(
            r#"{{
            "data": [
                {{
                    "code": "SUCCESS",
                    "details": {{
                        "Modified_Time": "2019-05-02T11:17:33+05:30",
                        "Modified_By": {{
                            "name": "Patricia Boyle",
                            "id": "554023000000235011"
                        }},
                        "Created_Time": "2019-05-02T11:17:33+05:30",
                        "id": "{}",
                        "Created_By": {{
                            "name": "Patricia Boyle",
                            "id": "554023000000235011"
                        }}
                    }},
                    "message": "record added",
                    "status": "success"
                }}
            ]
        }}"#,
            record_id
        );
        let mocker = get_mocker("POST", Matcher::Any, Some(&body));
        let mut client = get_client(Some(String::from(access_token)), None, Some(api_domain));

        let mut record: HashMap<&str, &str> = HashMap::new();
        record.insert("name", "New Record Name");

        let response = client.insert("Accounts", vec![record]).unwrap();
        let response = response.data.get(0).unwrap();

        let details = match &response.details {
            response::ResponseDataItemDetails::Error(_) => {
                panic!("Experienced an unexpected error");
            }
            response::ResponseDataItemDetails::Success(details) => details,
        };

        mocker.assert();
        assert_eq!(details.id, record_id);
    }

    #[test]
    /// Tests that an error code returned via the `insert()` method returns an error.
    fn insert_regular_error() {
        let access_token = "9999.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let api_domain = mockito::server_url();
        let error_code = "INVALID_MODULE";
        let body = format!(
            r#"{{
            "code": "{}",
            "details": {{}},
            "message": "Please check if the URL trying to access is a correct one",
            "status": "error"
        }}"#,
            error_code
        );
        let mocker = get_mocker("POST", Matcher::Any, Some(&body));
        let mut client = get_client(Some(String::from(access_token)), None, Some(api_domain));

        let mut record: HashMap<&str, &str> = HashMap::new();
        record.insert("name", "New Record Name");

        match client.insert("INVALID_MODULE", vec![record]) {
            Ok(_) => panic!("Response did not return an error"),
            Err(err) => match err {
                ClientError::ApiError(error) => assert_eq!(error.code, error_code),
                _ => panic!("Wrong error type"),
            },
        }

        mocker.assert();
    }

    #[test]
    /// Tests that a plain error message returned via the `insert()` method returns an error.
    fn insert_many_text_error() {
        let access_token = "9999.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let api_domain = mockito::server_url();
        let error_code = "invalid_client";
        let body = error_code.to_string();
        let mocker = get_mocker("POST", Matcher::Any, Some(&body));
        let mut client = get_client(Some(String::from(access_token)), None, Some(api_domain));

        let mut record: HashMap<&str, &str> = HashMap::new();
        record.insert("name", "New Record Name");

        match client.insert("INVALID_MODULE", vec![record]) {
            Ok(_) => panic!("Response did not return an error"),
            Err(err) => {
                assert_eq!(err.to_string(), error_code.to_string());
            }
        }

        mocker.assert();
    }

    #[test]
    /// Tests that updating a record via the `update_many()` method works.
    fn update_many_success() {
        let access_token = "9999.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let api_domain = mockito::server_url();
        let record_id = "40000000123456789";
        let body = format!(
            r#"{{
            "data": [
                {{
                    "code": "SUCCESS",
                    "details": {{
                      "Modified_Time": "2019-05-02T11:17:33+05:30",
                      "Modified_By": {{
                        "name": "Patricia Boyle",
                        "id": "554023000000235011"
                      }},
                      "Created_Time": "2019-05-02T11:17:33+05:30",
                      "id": "{}",
                      "Created_By": {{
                        "name": "Patricia Boyle",
                        "id": "554023000000235011"
                      }}
                    }},
                    "message": "record updated",
                    "status": "success"
                }}
            ]
        }}"#,
            record_id
        );
        let mocker = get_mocker("PUT", Matcher::Any, Some(&body));
        let mut client = get_client(Some(String::from(access_token)), None, Some(api_domain));

        let mut record: HashMap<&str, &str> = HashMap::new();
        record.insert("name", "New Record Name");

        let response = client.update_many("Accounts", vec![record]).unwrap();
        let response = response.data.get(0).unwrap();

        let details = match &response.details {
            response::ResponseDataItemDetails::Error(_) => {
                panic!("Experienced an unexpected error");
            }
            response::ResponseDataItemDetails::Success(details) => details,
        };

        mocker.assert();
        assert_eq!(details.id, record_id);
    }

    #[test]
    /// Tests that an error code returned via the `update_many()` method returns an error.
    fn update_regular_error() {
        let access_token = "9999.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let api_domain = mockito::server_url();
        let error_code = "INVALID_MODULE";
        let body = format!(
            r#"{{
            "code": "{}",
            "details": {{}},
            "message": "Please check if the URL trying to access is a correct one",
            "status": "error"
        }}"#,
            error_code
        );
        let mocker = get_mocker("PUT", Matcher::Any, Some(&body));
        let mut client = get_client(Some(String::from(access_token)), None, Some(api_domain));

        let mut record: HashMap<&str, &str> = HashMap::new();
        record.insert("name", "New Record Name");

        match client.update_many("INVALID_MODULE", vec![record]) {
            Ok(_) => panic!("Response did not return an error"),
            Err(err) => match err {
                ClientError::ApiError(error) => assert_eq!(error.code, error_code),
                _ => panic!("Wrong error type"),
            },
        }

        mocker.assert();
    }

    #[test]
    /// Tests that a plain error message returned via the `update_many()` method returns an error.
    fn update_many_text_error() {
        let access_token = "9999.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let api_domain = mockito::server_url();
        let error_code = "invalid_client";
        let body = error_code.to_string();
        let mocker = get_mocker("PUT", Matcher::Any, Some(&body));
        let mut client = get_client(Some(String::from(access_token)), None, Some(api_domain));

        let mut record: HashMap<&str, &str> = HashMap::new();
        record.insert("name", "New Record Name");

        match client.update_many("INVALID_MODULE", vec![record]) {
            Ok(_) => panic!("Response did not return an error"),
            Err(err) => {
                assert_eq!(err.to_string(), error_code.to_string());
            }
        }

        mocker.assert();
    }

    #[test]
    fn test_parse_params() {
        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("cvid", "00000");
        params.insert("page", "2");

        let converted = parse_params(params).unwrap();

        match converted.as_str() {
            "page=2&cvid=00000" => (),
            "cvid=00000&page=2" => (),
            _ => {
                panic!("Params did not convert properly");
            }
        }
    }
}
