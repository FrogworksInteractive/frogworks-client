use std::borrow::Cow;
use std::ffi::OsString;
use std::fs::{read_to_string, File, OpenOptions};
use std::io::{Error, Write};
use std::path::PathBuf;
use std::str::FromStr;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use bytes::Bytes;
use gethostname::gethostname;
use reqwest::blocking::{Client, Response};
use reqwest::blocking::multipart::Form;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, to_string_pretty, Value};
use url::Url;
use crate::api_error::APIError;
use crate::application::Application;
use crate::application_key::ApplicationKey;
use crate::application_version::ApplicationVersion;
use crate::cloud_data::CloudData;
use crate::deposit::Deposit;
use crate::friend::Friend;
use crate::friend_request::FriendRequest;
use crate::iap::IAP;
use crate::iap_record::IAPRecord;
use crate::invite::Invite;
use crate::purchase::Purchase;
use crate::sale::Sale;
use crate::session::Session;
use crate::transaction::Transaction;
use crate::user::User;

pub mod activity;
pub mod application;
pub mod application_key;
pub mod application_session;
pub mod application_version;
pub mod cloud_data;
pub mod deposit;
pub mod friend;
pub mod friend_request;
pub mod iap;
pub mod iap_record;
pub mod invite;
pub mod photo;
pub mod purchase;
pub mod sale;
pub mod session;
pub mod transaction;
pub mod user;
pub mod api_error;

pub type ApiResult<T> = Result<T, APIError>;

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailVerificationCheckResponse {
    email_verified: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    session_id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionAuthenticationResponse {
    authenticated: bool,
    user_id: Option<i32>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApplicationCreationResponse {
    details: String,
    application_id: i32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetApplicationVersionsResponse {
    versions: Vec<ApplicationVersion>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAllSalesResponse {
    sales: Vec<Sale>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUserTransactionsResponse {
    transactions: Vec<Transaction>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUserApplicationKeysResponse {
    application_keys: Vec<ApplicationKey>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetIAPRecordsResponse {
    iap_records: Vec<IAPRecord>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetFriendRequestsResponse {
    friend_requests: Vec<FriendRequest>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetFriendsResponse {
    friends: Vec<Friend>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetInvitesResponse {
    invites: Vec<Invite>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetIAPsResponse {
    iaps: Vec<IAP>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUserSessionsResponse {
    sessions: Vec<Session>
}

pub struct ApiService {
    base_url: Url,
    server_port: u16,
    session_id: Option<String>,
    user_agent_string: Option<String>,
    version: String,
    client: Client
}

impl ApiService {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: Url::from_str(base_url.as_str()).unwrap(),
            server_port: 80,
            session_id: None,
            user_agent_string: None,
            version: String::from("1.0"),
            client: Client::new()
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.server_port = port;
        self
    }

    pub fn with_authentication(mut self, session_id: String) -> Self {
        self.session_id = Some(String::from(session_id));
        self
    }

    pub fn with_user_agent(mut self, user_agent_string: String) -> Self {
        self.user_agent_string = Some(user_agent_string);
        self
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    fn get_headers(&self) -> HeaderMap {
        let mut headers: HeaderMap = HeaderMap::new();

        if let Some(user_agent_string) = &self.user_agent_string {
            headers.insert("User-Agent",
                           HeaderValue::from_str(format!("{} v{}",
                                                         user_agent_string,
                                                         self.version).as_str()).unwrap());
        }


        if let Some(session_id) = &self.session_id {
            headers.insert("Session-Id", HeaderValue::from_str(session_id).unwrap());
        }

        headers
    }

    fn get_url_for(&self, path: &str) -> Url {
        self.base_url.join(path).unwrap()
    }

    fn get_platform(&self) -> String {
        String::from(if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "unknown"
        })
    }

    fn get_mac_address(&self) -> Result<Option<String>, mac_address::MacAddressError> {
        match mac_address::get_mac_address() {
            Ok(Some(mac_address)) => {
                Ok(Some(format!("{}", mac_address)))
            },
            Ok(None) => Ok(None),
            Err(err) => Err(err)
        }
    }

    pub fn authenticated(&self) -> bool {
        self.session_id.is_some()
    }

    /// Pings the server (used for connectivity testing).
    pub fn ping(&self) -> ApiResult<Value> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/ping");

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .send()?;

        if response.status() != StatusCode::OK {
            return Err(APIError::UnhandledStatusCode(response.status()))
        }

        Ok(from_str(&response.text()?)?)
    }

    /// Requests a verification code be sent to a specified email address.
    ///
    /// # Arguments
    /// * `email_address` The email address to send the verification code to
    pub fn request_email_verification(&self, email_address: String) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url = self.get_url_for("/api/email-verification/request");

        let form: Form = Form::new()
            .text("email_address", email_address);

        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::OK => Ok(()),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Checks a verification code against the one in the database for a specific email address (if
    /// any).
    ///
    /// # Arguments
    /// * `email_address` - The user's email address
    /// * `verification_code` - The email verification code
    pub fn check_email_verification(&self, email_address: String,
                                    verification_code: i32) -> ApiResult<bool> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/email-verification/check");

        let form: Form = Form::new()
            .text("email_address", email_address)
            .text("verification_code", verification_code.to_string());

        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let verification_response: EmailVerificationCheckResponse =
                    from_str(&response.text()?)?;

                Ok(verification_response.email_verified)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Attempts to create a new user account.
    ///
    /// # Arguments
    /// * `username` - The user's preferred username
    /// * `name` - The user's name
    /// * `email_address` - The user's email address
    /// * `password` - The user's password
    /// * `email_verification_code` - The verification code sent to the user's email address
    pub fn register(&self, username: String, name: String, email_address: String,
                    password: String, email_verification_code: i32) -> ApiResult<Value> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/register");

        let form: Form = Form::new()
            .text("username", username)
            .text("name", name)
            .text("email_address", email_address)
            .text("password", password)
            .text("email_verification_code", email_verification_code.to_string());

        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::BAD_REQUEST => {
                Err(APIError::BadRequest(response.text()?))
            },
            StatusCode::CREATED => {
                Ok(from_str::<Value>(response.text()?.as_str())?)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Attempt to log in; creating a new session.
    /// <br>
    /// This collects the following device data:
    ///  - Hostname
    ///  - MAC address
    ///  - Platform (windows, linux, macos, unknown)
    ///
    /// # Arguments
    ///
    /// * `username` - The user's username.
    /// * `password` - The user's password.
    pub fn login(&self, username: String, password: String) -> ApiResult<String> {
        // Get the device details for the session (hostname, mac address, platform).
        let hostname: OsString = gethostname();
        let hostname_cow: Cow<str> = hostname.to_string_lossy();
        let hostname_string: String = hostname_cow.into_owned();
        let mac_address: String = self.get_mac_address().expect("Failed to get mac address.")
            .expect("Failed to get mac address.");
        let platform: String = self.get_platform();

        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/login");

        let form: Form = Form::new()
            .text("username", username)
            .text("password", password)
            .text("hostname", hostname_string)
            .text("mac_address", mac_address)
            .text("platform", platform);

        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::BAD_REQUEST => {
                Err(APIError::BadRequest(response.text()?))
            },
            StatusCode::OK => {
                // Login went okay; parse the response.
                let response: LoginResponse = from_str(&response.text()?)?;

                Ok(response.session_id)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Attempt to fetch a user by their Frogworks ID.
    ///
    /// # Arguments
    /// * `identifier` The user's Frogworks ID
    pub fn get_user(&self, identifier: String,
                    identifier_type: String) -> ApiResult<User> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/get");

        let form: Form = Form::new()
            .text("identifier", identifier)
            .text("identifier_type", identifier_type);

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                // The request went okay; parse the result.
                let user: User = from_str(&response.text()?)?;

                Ok(user)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status())),
        }
    }

    /// Attempt to authenticate the current session (must have a valid session id).
    pub fn authenticate_session(&self) -> ApiResult<SessionAuthenticationResponse> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/session/authenticate");

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                // The request is okay; parse the response.
                let response: SessionAuthenticationResponse =
                    from_str(&response.text()?)?;

                Ok(response)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Attempt to delete the current session (not to be confused with `delete_specific_session`).
    pub fn delete_session(&self) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/session/delete");

        let response: Response = self.client
            .delete(url.as_str())
            .headers(headers)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                Ok(())
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn delete_specific_session(&self, session_id: i32) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/session/delete-specific");

        let form: Form = Form::new()
            .text("session_id", session_id.to_string());

        let response: Response = self.client
            .delete(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                Ok(())
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Attempt to create an application.
    ///
    /// # Arguments
    /// * `name` - The application's name
    /// * `package_name` - The application's package name
    /// * `application_type` - The application's type (game, application)
    /// * `description` - The application's description
    /// * `release_date` - The application's release date
    /// * `early_access` - Whether the application is in early access or not
    /// * `supported_platforms` - The list of supported platforms (windows, linux, macos)
    /// * `genres` - The list of the application's genres
    /// * `tags` - The list of the application's tags
    /// * `base_price` - The base price of the application
    pub fn create_application(&self, name: String, package_name: String,
                              application_type: String, description: String,
                              release_date: String, early_access: bool,
                              supported_platforms: Vec<String>, genres: Vec<String>,
                              tags: Vec<String>,
                              base_price: f32) -> ApiResult<ApplicationCreationResponse> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/application/create");

        let form: Form = Form::new()
            .text("name", name)
            .text("package_name", package_name)
            .text("type", application_type)
            .text("description", description)
            .text("release_date", release_date)
            .text("early_access", early_access.to_string())
            .text("supported_platforms", supported_platforms.join(","))
            .text("genres", genres.join(","))
            .text("tags", tags.join(","))
            .text("base_price", base_price.to_string());

        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::CREATED => {
                // Parse the response.
                let creation_response: ApplicationCreationResponse =
                    from_str(&response.text()?)?;

                Ok(creation_response)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Get an application by its unique id.
    ///
    /// # Arguments
    /// * `application_id` - The application's id
    pub fn get_application(&self, application_id: i32) -> ApiResult<Application> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/application/get");

        let form: Form = Form::new()
            .text("application_id", application_id.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                // Parse the response.
                let application: Application = from_str(&response.text()?)?;

                Ok(application)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Get all the versions for a specific application and platform.
    ///
    /// # Arguments
    /// * `application_id` - The application's id
    /// * `platform` - The target platform
    pub fn get_application_versions(&self, application_id: i32,
                                    platform: String) -> ApiResult<Vec<ApplicationVersion>> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/application/versions");

        let form: Form = Form::new()
            .text("application_id", application_id.to_string())
            .text("platform", platform);

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                // Parse the response.
                let api_response: GetApplicationVersionsResponse =
                    from_str(&response.text()?)?;

                Ok(api_response.versions)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Get a specific application version by its unique id.
    ///
    /// # Arguments
    /// * `version_id` - The version's id
    pub fn get_application_version(&self, version_id: i32) -> ApiResult<ApplicationVersion> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/application/versions/get-specific");

        let form: Form = Form::new()
            .text("version_id", version_id.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                // Parse the response.
                let application_version: ApplicationVersion = from_str(&response.text()?)?;

                Ok(application_version)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Get a specific application version by its platform and version name.
    ///
    /// # Arguments
    /// * `application_id` - The application's id
    /// * `platform` - The target platform
    /// * `version_name` - The target version name (e.g. "1.0")
    pub fn get_application_version_for(
            &self, application_id: i32,
            version_name: String, platform: String) -> ApiResult<ApplicationVersion> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/application/versions/get/fine-tuned");

        let form: Form = Form::new()
            .text("application_id", application_id.to_string())
            .text("version_name", version_name)
            .text("platform", platform);

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                // Parse the response.
                let application_version: ApplicationVersion = from_str(&response.text()?)?;

                Ok(application_version)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Attempt to download a specific application version.
    ///
    /// # Arguments
    /// * `version_id` - The id of the version you are trying to download
    /// * `download_folder` - The folder to download the file to
    pub fn download_application_version(&self, version_id: i32,
                                        download_folder: String) -> ApiResult<()> {
        // Get the version.
        let version: ApplicationVersion = self.get_application_version(version_id.clone())?;

        // Send the version download request.
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/application/versions/download");

        let form: Form = Form::new()
            .text("version_id", version_id.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                // The server is okay with the file download; parse the response.
                // Calculate the download filepath.
                let mut filepath: PathBuf = PathBuf::from(download_folder);
                filepath.push(version.filename);

                // Create the file.
                let mut file: File = File::create(filepath)?;

                // Get the response bytes.
                let file_contents: Bytes = response.bytes()?;

                // Write the file contents.
                file.write_all(&file_contents).expect("Failed to write file contents.");

                Ok(())
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Attempt to update the specified application's latest version.
    pub fn update_application_version(&self, application_id: i32,
                                      version_name: String) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/application/update-version");

        let form: Form = Form::new()
            .text("application_id", application_id.to_string())
            .text("version", version_name.to_string());

        let response: Response = self.client
            .put(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                Ok(())
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn create_application_version(&self, application_id: i32, name: String,
                                      platform: String, release_date: String,
                                      filename: String, executable: String,
                                      filepath: String) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/version/create");

        let form: Form = Form::new()
            .text("application_id", application_id.to_string())
            .text("name", name)
            .text("platform", platform)
            .text("release_date", release_date)
            .text("filename", filename)
            .text("executable", executable)
            .file("file", filepath)?;

        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                Ok(())
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn create_sale(&self, application_id: i32, title: String, description: String,
                       price: f32, start_date: String,
                       end_date: String) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/sales/create");

        let form: Form = Form::new()
            .text("application_id", application_id.to_string())
            .text("title", title)
            .text("description", description)
            .text("price", price.to_string())
            .text("start_date", start_date)
            .text("end_date", end_date);

        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                Ok(())
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn get_active_sale(&self, application_id: i32) -> ApiResult<Sale> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/sales/get");

        let form: Form = Form::new()
            .text("application_id", application_id.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let sale: Sale = from_str(&response.text()?)?;

                Ok(sale)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn get_all_sales(&self) -> ApiResult<Vec<Sale>> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/sales/get-all");

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                // Parse the response.
                let sales_response: GetAllSalesResponse = from_str(&response.text()?)?;

                Ok(sales_response.sales)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn delete_sale(&self, sale_id: i32) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/sales/delete");

        let form: Form = Form::new()
            .text("sale_id", sale_id.to_string());

        let response: Response = self.client
            .delete(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn get_user_transactions(&self, user_id: i32) -> ApiResult<Vec<Transaction>> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/get-transactions");

        let form: Form = Form::new()
            .text("user_id", user_id.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                // Parse the response.
                let transactions_response: GetUserTransactionsResponse =
                    from_str(&response.text()?)?;

                Ok(transactions_response.transactions)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    pub fn get_transaction(&self, transaction_id: i32) -> ApiResult<Transaction> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/get-transaction");

        let form: Form = Form::new()
            .text("transaction_id", transaction_id.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let transaction: Transaction = from_str(&response.text()?)?;

                Ok(transaction)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn get_purchase(&self, purchase_id: i32) -> ApiResult<Purchase> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/get-purchase");

        let form: Form = Form::new()
            .text("purchase_id", purchase_id.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let purchase: Purchase = from_str(&response.text()?)?;

                Ok(purchase)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn get_deposit(&self, deposit_id: i32) -> ApiResult<Deposit> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/get-deposit");

        let form: Form = Form::new()
            .text("deposit_id", deposit_id.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let deposit: Deposit = from_str(&response.text()?)?;

                Ok(deposit)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn get_application_key(&self, key: String) -> ApiResult<ApplicationKey> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/get-application-key");

        let form: Form = Form::new()
            .text("key", key.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let application_key: ApplicationKey = from_str(&response.text()?)?;

                Ok(application_key)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn get_user_application_keys(&self, user_id: i32) -> ApiResult<Vec<ApplicationKey>> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/get-application-keys");

        let form: Form = Form::new()
            .text("user_id", user_id.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::BadRequest(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let keys_response: GetUserApplicationKeysResponse = from_str(&response.text()?)?;

                Ok(keys_response.application_keys)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn purchase_application(&self, application_id: i32) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/purchase/application");

        let form: Form = Form::new()
            .text("application_id", application_id.to_string());

        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn purchase_iap(&self, iap_id: i32) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/purchase/iap");

        let form: Form = Form::new()
            .text("iap_id", iap_id.to_string());

        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn get_iap_records(&self, user_id: i32, application_id: i32, 
                           only_unacknowledged: bool) -> ApiResult<Vec<IAPRecord>> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/get-iap-records");

        let mut form: Form = Form::new()
            .text("user_id", user_id.to_string())
            .text("application_id", application_id.to_string());

        if only_unacknowledged {
            form = form.text("only_unacknowledged", "true");
        }
        
        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let records_response: GetIAPRecordsResponse = from_str(&response.text()?)?;

                Ok(records_response.iap_records)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn get_session(&self, session_id: String) -> ApiResult<Session> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/session/get");

        let form: Form = Form::new()
            .text("session_id", session_id.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let session_response: Session = from_str(&response.text()?)?;

                Ok(session_response)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn send_friend_request(&self, user_id: i32) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/friend/send-request");

        let form: Form = Form::new()
            .text("user_id", user_id.to_string());

        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn delete_friend_request(&self, request_id: i32) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/friend/delete-request");

        let form: Form = Form::new()
            .text("request_id", request_id.to_string());

        let response: Response = self.client
            .delete(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn get_incoming_friend_requests(&self, user_id: i32) -> ApiResult<Vec<FriendRequest>> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/friend/get-requests/incoming");

        let form: Form = Form::new()
            .text("user_id", user_id.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let friend_requests: GetFriendRequestsResponse = from_str(&response.text()?)?;

                Ok(friend_requests.friend_requests)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn get_outgoing_friend_requests(&self, user_id: i32) -> ApiResult<Vec<FriendRequest>> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/friend/get-requests/outgoing");

        let form: Form = Form::new()
            .text("user_id", user_id.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let friend_requests: GetFriendRequestsResponse = from_str(&response.text()?)?;

                Ok(friend_requests.friend_requests)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn accept_friend_request(&self, request_id: i32) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/friend/accept-request");

        let form: Form = Form::new()
            .text("request_id", request_id.to_string());

        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn get_friends(&self, user_id: i32) -> ApiResult<Vec<Friend>> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/get-friends");

        let form: Form = Form::new()
            .text("user_id", user_id.to_string());

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                // Parse the response.
                let friends_response: GetFriendsResponse = from_str(&response.text()?)?;

                Ok(friends_response.friends)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn remove_friend(&self, user_id: i32) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/friend/remove");

        let form: Form = Form::new()
            .text("user_id", user_id.to_string());

        let response: Response = self.client
            .delete(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn send_invite(&self, user_id: i32, application_id: i32, 
                       details: String) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/send-invite");
        
        let form: Form = Form::new()
            .text("user_id", user_id.to_string())
            .text("application_id", application_id.to_string())
            .text("details", details.to_string());
        
        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn get_invites(&self, user_id: i32) -> ApiResult<Vec<Invite>> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/get-invites");
        
        let form: Form = Form::new()
            .text("user_id", user_id.to_string());
        
        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() { 
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let invites_response: GetInvitesResponse = from_str(&response.text()?)?;
                
                Ok(invites_response.invites)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn get_invite(&self, invite_id: i32) -> ApiResult<Invite> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/get-invite");
        
        let form: Form = Form::new()
            .text("invite_id", invite_id.to_string());
        
        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let invite: Invite = from_str(&response.text()?)?;
                
                Ok(invite)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn delete_invite(&self, invite_id: i32) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/delete-invite");
        
        let form: Form = Form::new()
            .text("invite_id", invite_id.to_string());
        
        let response: Response = self.client
            .delete(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn create_photo(&self, subfolder: String, filepath: String) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/photo/create");
        
        let form: Form = Form::new()
            .text("subfolder", subfolder.to_string())
            .file("photo", filepath)?;
        
        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn get_photo(&self, id: i32) -> ApiResult<Value> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/photo/get");
        
        let form: Form = Form::new()
            .text("id", id.to_string());
        
        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match &response.status() {
            &StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            &StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            &StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            &StatusCode::OK => {
                // Get the photo's bytes.
                let response_bytes: Bytes = response.bytes()?;

                // Encode the bytes into base 64.
                let base64: String = BASE64_STANDARD.encode(response_bytes);
                
                Ok(json!({
                    "bytes": base64
                }))
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn create_iap(&self, application_id: i32, title: String, description: String, 
                      price: f32, data: String) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/iap/create");
        
        let form: Form = Form::new()
            .text("application_id", application_id.to_string())
            .text("title", title.to_string())
            .text("description", description.to_string())
            .text("price", price.to_string())
            .text("data", data.to_string());
        
        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn get_iap(&self, id: i32) -> ApiResult<IAP> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/iap/get");
        
        let form: Form = Form::new()
            .text("id", id.to_string());
        
        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let iap: IAP = from_str(&response.text()?)?;
                
                Ok(iap)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn get_iaps(&self, application_id: i32) -> ApiResult<Vec<IAP>> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/application/get-iaps");
        
        let form: Form = Form::new()
            .text("application_id", application_id.to_string());
        
        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() { 
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let iaps_response: GetIAPsResponse = from_str(&response.text()?)?;
                
                Ok(iaps_response.iaps)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn upload_cloud_data(&self, user_id: i32, application_id: i32, 
                             cloud_data: String) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/cloud-data/upload");
        
        let form: Form = Form::new()
            .text("user_id", user_id.to_string())
            .text("application_id", application_id.to_string())
            .text("data", cloud_data);
        
        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::CREATED => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn get_cloud_data(&self, user_id: i32, application_id: i32) -> ApiResult<CloudData> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/cloud-data/get");
        
        let form: Form = Form::new()
            .text("user_id", user_id.to_string())
            .text("application_id", application_id.to_string());
        
        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() { 
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let cloud_data: CloudData = from_str(&response.text()?)?;
                
                Ok(cloud_data)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn delete_cloud_data(&self, user_id: i32, application_id: i32) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/cloud-data/delete");
        
        let form: Form = Form::new()
            .text("user_id", user_id.to_string())
            .text("application_id", application_id.to_string());
        
        let response: Response = self.client
            .delete(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn delete_application_cloud_data(&self, application_id: i32) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/application/delete-cloud-data");
        
        let form: Form = Form::new()
            .text("application_id", application_id.to_string());
        
        let response: Response = self.client
            .delete(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn update_profile_photo(&self, user_id: i32, photo_id: i32) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/update-profile-photo");
        
        let form: Form = Form::new()
            .text("user_id", user_id.to_string())
            .text("photo_id", photo_id.to_string());
        
        let response: Response = self.client
            .put(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() { 
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn get_user_sessions(&self, user_id: i32) -> ApiResult<Vec<Session>> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/user/get-sessions");
        
        let form: Form = Form::new()
            .text("user_id", user_id.to_string());
        
        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let sessions_response: GetUserSessionsResponse = from_str(&response.text()?)?;
                
                Ok(sessions_response.sessions)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn get_iap_record(&self, id: i32) -> ApiResult<IAPRecord> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/iap-record/get");
        
        let form: Form = Form::new()
            .text("id", id.to_string());
        
        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => {
                let iap_record: IAPRecord = from_str(&response.text()?)?;
                
                Ok(iap_record)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
    
    pub fn acknowledge_iap_record(&self, id: i32) -> ApiResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/iap-record/acknowledge");
        
        let form: Form = Form::new()
            .text("id", id.to_string());
        
        let response: Response = self.client
            .put(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;
        
        match response.status() { 
            StatusCode::UNAUTHORIZED => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::FORBIDDEN => Err(APIError::Unauthorized(response.text()?)),
            StatusCode::BAD_REQUEST => Err(APIError::BadRequest(response.text()?)),
            StatusCode::OK => Ok(()),
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        } 
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CliConfig {
    server_url: String,
    server_port: Option<i32>
}

pub struct CliTools {
    config_filename: String
}

impl CliTools {
    pub fn new(config_filename: String) -> Self {
        Self { config_filename }
    }
    
    fn get_config_filepath(&self) -> PathBuf {
        // Get the current working directory.
        let cwd: PathBuf = std::env::current_dir().unwrap();
        
        // Combine the paths to get the full filepath.
        cwd.join(&self.config_filename)
    }
    
    pub fn get_config(&self) -> Result<CliConfig, Error> {
        // Read the config file.
        let data: String = read_to_string(self.get_config_filepath())?;
        
        // Load the config.
        let config: CliConfig = from_str(&data)?;
        
        Ok(config)
    }
    
    pub fn write_config(&self, config: CliConfig) -> Result<(), Error> {
        // Serialize the config data.
        let config_data: String = to_string_pretty(&config)?;
        
        // Write the data to the config file.
        let mut file: File = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.get_config_filepath())?;
        
        file.write_all(config_data.as_bytes())?;
        
        Ok(())
    }
}
