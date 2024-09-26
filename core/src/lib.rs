use std::borrow::Cow;
use std::ffi::OsString;
use std::fmt::format;
use std::io;
use std::str::FromStr;
use gethostname::gethostname;
use reqwest::blocking::{Client, Response};
use reqwest::blocking::multipart::Form;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use url::Url;
use crate::activity::Activity;
use crate::api_error::APIError;
use crate::application::Application;
use crate::application_key::ApplicationKey;
use crate::application_session::ApplicationSession;
use crate::application_version::ApplicationVersion;
use crate::cloud_data::CloudData;
use crate::deposit::Deposit;
use crate::friend::Friend;
use crate::friend_request::FriendRequest;
use crate::iap::IAP;
use crate::iap_record::IAPRecord;
use crate::invite::Invite;
use crate::photo::Photo;
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

pub type APIResult<T> = Result<T, APIError>;

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

pub struct APIService {
    base_url: Url,
    server_port: u16,
    session_id: Option<String>,
    user_agent_string: Option<String>,
    version: String,
    client: Client
}

impl APIService {
    pub fn new(base_url: &'static str) -> Self {
        Self {
            base_url: Url::from_str(base_url).unwrap(),
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

    pub fn with_authentication(mut self, session_id: &'static str) -> Self {
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

    fn get_platform(&self) -> &'static str {
        if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "unknown"
        }
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
    pub fn ping(&self) -> APIResult<Value> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/ping");

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .send()?;

        if response.status() != StatusCode::OK {
            return Err(APIError::UnhandledStatusCode(response.status()))
        }

        Ok(from_str(response.text()?.as_str())?)
    }

    /// Requests a verification code be sent to a specified email address.
    ///
    /// # Arguments
    /// * `email_address` The email address to send the verification code to
    pub fn request_email_verification(&self, email_address: &'static str) -> APIResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url = self.get_url_for("/api/email-verification/request");

        let form: Form = Form::new()
            .text("email_address", email_address);

        let response: Response = self.client
            .post(url.as_str())
            .headers(headers)
            .multipart(form)
            .send()?;

        if response.status() == StatusCode::OK {
            return Ok(())
        } else if response.status() == StatusCode::BAD_REQUEST {
            return Err(APIError::BadRequest(response.text()?))
        }

        Err(APIError::UnhandledStatusCode(response.status()))
    }

    /// Checks a verification code against the one in the database for a specific email address (if
    /// any).
    ///
    /// # Arguments
    /// * `email_address` - The user's email address
    /// * `verification_code` - The email verification code
    pub fn check_email_verification(&self, email_address: &'static str,
                                    verification_code: i32) -> APIResult<bool> {
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

        if response.status() == StatusCode::OK {
            let verification_response: EmailVerificationCheckResponse =
                from_str(response.text()?.as_str())?;

            return Ok(verification_response.email_verified)
        } else if response.status() == StatusCode::BAD_REQUEST {
            return Err(APIError::BadRequest(response.text()?))
        }

        Err(APIError::UnhandledStatusCode(response.status()))
    }

    /// Attempts to create a new user account.
    ///
    /// # Arguments
    /// * `username` - The user's preferred username
    /// * `name` - The user's name
    /// * `email_address` - The user's email address
    /// * `password` - The user's password
    /// * `email_verification_code` - The verification code sent to the user's email address
    pub fn register(&self, username: &'static str, name: &'static str, email_address: &'static str,
                    password: &'static str, email_verification_code: i32) -> APIResult<Value> {
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
    pub fn login(&self, username: &'static str, password: &'static str) -> APIResult<String> {
        // Get the device details for the session (hostname, mac address, platform).
        let hostname: OsString = gethostname();
        let hostname_cow: Cow<str> = hostname.to_string_lossy();
        let hostname_string: String = hostname_cow.into_owned();
        let mac_address: String = self.get_mac_address().expect("Failed to get mac address.")
            .expect("Failed to get mac address.");
        let platform: &str = self.get_platform();

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
                let response: LoginResponse = from_str(response.text()?.as_str())?;

                Ok(response.session_id)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Attempt to fetch a user by their Frogworks ID.
    ///
    /// # Arguments
    /// * `identifier` The user's Frogworks ID
    pub fn get_user(&self, identifier: &'static str,
                    identifier_type: &'static str) -> APIResult<User> {
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
            StatusCode::UNAUTHORIZED => {
                Err(APIError::Unauthorized(response.text()?))
            },
            StatusCode::FORBIDDEN => {
                Err(APIError::Forbidden(response.text()?))
            },
            StatusCode::BAD_REQUEST => {
                Err(APIError::BadRequest(response.text()?))
            },
            StatusCode::OK => {
                // The request went okay; parse the result.
                let user: User = from_str(response.text()?.as_str())?;

                Ok(user)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status())),
        }
    }

    /// Attempt to authenticate the current session (must have a valid session id).
    pub fn authenticate_session(&self) -> APIResult<SessionAuthenticationResponse> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/session/authenticate");

        let response: Response = self.client
            .get(url.as_str())
            .headers(headers)
            .send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => {
                Err(APIError::Unauthorized(response.text()?))
            },
            StatusCode::OK => {
                // The request is okay; parse the response.
                let response: SessionAuthenticationResponse =
                    from_str(response.text()?.as_str())?;

                Ok(response)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Attempt to delete the current session (not to be confused with `delete_specific_session`).
    pub fn delete_session(&self) -> APIResult<()> {
        let headers: HeaderMap = self.get_headers();
        let url: Url = self.get_url_for("/api/session/delete");

        let response: Response = self.client
            .delete(url.as_str())
            .headers(headers)
            .send()?;

        match response.status() {
            StatusCode::BAD_REQUEST => {
                Err(APIError::BadRequest(response.text()?))
            },
            StatusCode::UNAUTHORIZED => {
                Err(APIError::Unauthorized(response.text()?))
            }
            StatusCode::OK => {
                Ok(())
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    pub fn delete_specific_session(&self, session_id: i32) -> APIResult<()> {
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
            StatusCode::UNAUTHORIZED => {
                Err(APIError::Unauthorized(response.text()?))
            },
            StatusCode::FORBIDDEN => {
                Err(APIError::Forbidden(response.text()?))
            },
            StatusCode::BAD_REQUEST => {
                Err(APIError::BadRequest(response.text()?))
            },
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
    pub fn create_application(&self, name: &'static str, package_name: &'static str,
                              application_type: &'static str, description: &'static str,
                              release_date: &'static str, early_access: bool,
                              supported_platforms: Vec<&'static str>, genres: Vec<&'static str>,
                              tags: Vec<&'static str>,
                              base_price: f32) -> APIResult<ApplicationCreationResponse> {
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
            StatusCode::UNAUTHORIZED => {
                Err(APIError::Unauthorized(response.text()?))
            },
            StatusCode::FORBIDDEN => {
                Err(APIError::Unauthorized(response.text()?))
            },
            StatusCode::BAD_REQUEST => {
                Err(APIError::BadRequest(response.text()?))
            },
            StatusCode::CREATED => {
                // Parse the response.
                let creation_response: ApplicationCreationResponse =
                    from_str(response.text()?.as_str())?;

                Ok(creation_response)
            },
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Get an application by its unique id.
    ///
    /// # Arguments
    /// * `application_id` - The application's id
    pub fn get_application(&self, application_id: i32) -> APIResult<Application> {
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
            StatusCode::UNAUTHORIZED => {
                Err(APIError::Unauthorized(response.text()?))
            },
            StatusCode::FORBIDDEN => {
                Err(APIError::Unauthorized(response.text()?))
            },
            StatusCode::BAD_REQUEST => {
                Err(APIError::BadRequest(response.text()?))
            },
            StatusCode::OK => {
                // Parse the response.
                let application: Application = from_str(response.text()?.as_str())?;

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
                                    platform: &'static str) -> APIResult<Vec<ApplicationVersion>> {
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
            StatusCode::UNAUTHORIZED => {
                Err(APIError::Unauthorized(response.text()?))
            },
            StatusCode::FORBIDDEN => {
                Err(APIError::Unauthorized(response.text()?))
            },
            StatusCode::BAD_REQUEST => {
                Err(APIError::BadRequest(response.text()?))
            },
            StatusCode::OK => {
                // Parse the response.
                let api_response: GetApplicationVersionsResponse =
                    from_str(response.text()?.as_str())?;

                Ok(api_response.versions)
            }
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }

    /// Get a specific application version by its unique id.
    ///
    /// # Arguments
    /// * `version_id` - The version's id
    pub fn get_application_version(&self, version_id: i32) -> APIResult<ApplicationVersion> {
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
            StatusCode::UNAUTHORIZED => {
                Err(APIError::Unauthorized(response.text()?))
            },
            StatusCode::FORBIDDEN => {
                Err(APIError::Unauthorized(response.text()?))
            },
            StatusCode::BAD_REQUEST => {
                Err(APIError::BadRequest(response.text()?))
            },
            StatusCode::OK => {
                // Parse the response.
                let application_version: ApplicationVersion = from_str(response.text()?.as_str())?;

                Ok(application_version)
            }
            _ => Err(APIError::UnhandledStatusCode(response.status()))
        }
    }
}
