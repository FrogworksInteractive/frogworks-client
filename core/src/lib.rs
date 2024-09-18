use serde_json::from_str;
use crate::activity::Activity;
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

pub struct Parser {}

impl Parser {
    pub fn parse_activity(data: &str) -> Option<Activity> {
        from_str(data).ok()
    }

    pub fn parse_application(data: &str) -> Option<Application> {
        from_str(data).ok()
    }

    pub fn parse_application_key(data: &str) -> Option<ApplicationKey> {
        from_str(data).ok()
    }

    pub fn parse_application_session(data: &str) -> Option<ApplicationSession> {
        from_str(data).ok()
    }

    pub fn parse_application_version(data: &str) -> Option<ApplicationVersion> {
        from_str(data).ok()
    }

    pub fn parse_cloud_data(data: &str) -> Option<CloudData> {
        from_str(data).ok()
    }

    pub fn parse_deposit(data: &str) -> Option<Deposit> {
        from_str(data).ok()
    }

    pub fn parse_friend(data: &str) -> Option<Friend> {
        from_str(data).ok()
    }

    pub fn parse_friend_request(data: &str) -> Option<FriendRequest> {
        from_str(data).ok()
    }

    pub fn parse_iap(data: &str) -> Option<IAP> {
        from_str(data).ok()
    }

    pub fn parse_iap_record(data: &str) -> Option<IAPRecord> {
        from_str(data).ok()
    }

    pub fn parse_invite(data: &str) -> Option<Invite> {
        from_str(data).ok()
    }

    pub fn parse_photo(data: &str) -> Option<Photo> {
        from_str(data).ok()
    }

    pub fn parse_purchase(data: &str) -> Option<Purchase> {
        from_str(data).ok()
    }

    pub fn parse_sale(data: &str) -> Option<Sale> {
        from_str(data).ok()
    }

    pub fn parse_session(data: &str) -> Option<Session> {
        from_str(data).ok()
    }

    pub fn parse_transaction(data: &str) -> Option<Transaction> {
        from_str(data).ok()
    }

    pub fn parse_user(data: &str) -> Option<User> {
        from_str(data).ok()
    }
}
