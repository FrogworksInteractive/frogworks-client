use std::time::{Duration, Instant};
use clap::{value_parser, Arg, ArgMatches, Command};
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string_pretty, to_value, Value};
use serde_json::Value::Bool;
use core::ApiService;

const USER_AGENT_STRING: &str = "Frogworks CLI";
const APPLICATION_VERSION: &str = "0.1.0-dev";

trait CommandHandler {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value;
}

struct Ping {}

impl CommandHandler for Ping {
    fn handle_command(api_service: ApiService, _matches: &ArgMatches) -> Value {
        api_service.ping().unwrap()
    }
}

struct AuthenticateSession {}

impl CommandHandler for AuthenticateSession {
    fn handle_command(api_service: ApiService, _matches: &ArgMatches) -> Value {
        to_value(api_service.authenticate_session().unwrap()).unwrap()
    }
}

struct DeleteSession {}

impl CommandHandler for DeleteSession {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        let result = if let Some(session_id) = matches.get_one::<i32>("session-id") {
            api_service.delete_specific_session(session_id.to_owned())
        } else { 
            api_service.delete_session()
        };
        
        if result.is_ok() {
            Bool(true)
        } else { 
            Bool(false)
        }
    }
}

struct Login {}

impl CommandHandler for Login {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        let username: String = matches.get_one::<String>("username").unwrap().to_owned();
        let password: String = matches.get_one::<String>("password").unwrap().to_owned();
        
        // Logging in will get the session id.
        let session_id: String = api_service.login(username, password).unwrap();
        
        json!({"session_id": session_id})
    }
}

struct Register {}

impl CommandHandler for Register {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameters.
        let username: String = matches.get_one::<String>("username").unwrap().to_owned();
        let name: String = matches.get_one::<String>("name").unwrap().to_owned();
        let email_address: String = matches.get_one::<String>("email-address")
            .unwrap()
            .to_owned();
        let password: String = matches.get_one::<String>("password").unwrap().to_owned();
        let email_verification_code: i32 = matches.get_one::<i32>("email-verification-code")
            .unwrap()
            .to_owned();
        
        api_service.register(
            username,
            name,
            email_address,
            password,
            email_verification_code
        ).unwrap()
    }
}

struct RequestEmailVerification {}

impl CommandHandler for RequestEmailVerification {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameters.
        let email_address: String = matches.get_one::<String>("email-address")
            .unwrap()
            .to_owned();
        
        api_service.request_email_verification(email_address).unwrap();
        
        json!({"success": true})
    }
}

struct CheckEmailVerification {}

impl CommandHandler for CheckEmailVerification {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameters.
        let email_address: String = matches.get_one::<String>("email-address")
            .unwrap()
            .to_owned();
        let verification_code: i32 = matches.get_one::<i32>("verification-code")
            .unwrap()
            .to_owned();
        
        let verified = api_service.check_email_verification(
            email_address,
            verification_code
        ).unwrap();
        
        json!({"success": verified})
    }
}

struct GetUser {}

impl CommandHandler for GetUser {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameters.
        let identifier: String = matches.get_one::<String>("identifier")
            .unwrap()
            .to_owned();
        let identifier_type: String = matches.get_one::<String>("identifier-type")
            .unwrap()
            .to_owned();
        
        to_value(api_service.get_user(identifier, identifier_type).unwrap()).unwrap()
    }
}

struct CreateApplication {}

impl CommandHandler for CreateApplication {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameters.
        let name: String = matches.get_one::<String>("name")
            .unwrap()
            .to_owned();
        let package_name: String = matches.get_one::<String>("package-name")
            .unwrap()
            .to_owned();
        let application_type: String = matches.get_one::<String>("application-type")
            .unwrap()
            .to_owned();
        let description: String = matches.get_one::<String>("description")
            .unwrap()
            .to_owned();
        let release_date: String = matches.get_one::<String>("release-date")
            .unwrap()
            .to_owned();
        let early_access: bool = matches.get_one::<bool>("early-access")
            .unwrap()
            .to_owned();
        let supported_platforms_string: String = matches.get_one::<String>("supported-platforms")
            .unwrap()
            .to_owned();
        let genres_string: String = matches.get_one::<String>("genres")
            .unwrap()
            .to_owned();
        let tags_string: String = matches.get_one::<String>("tags")
            .unwrap()
            .to_owned();
        let base_price: f32 = matches.get_one::<f32>("base-price")
            .unwrap()
            .to_owned();
        
        // Parse each string list into a Vec<String>.
        let supported_platforms: Vec<String> = supported_platforms_string.split(",")
            .map(|s| s.to_owned())
            .collect();
        let genres: Vec<String> = genres_string.split(",")
            .map(|s| s.to_owned())
            .collect();
        let tags: Vec<String> = tags_string.split(",")
            .map(|s| s.to_owned())
            .collect();
        
        // Attempt to create the application.
        let response = api_service.create_application(
            name,
            package_name,
            application_type,
            description,
            release_date,
            early_access,
            supported_platforms,
            genres,
            tags,
            base_price
        ).unwrap();
        
        to_value(response).unwrap()
    }
}

struct GetApplication {}

impl CommandHandler for GetApplication {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let application_id: i32 = matches.get_one::<i32>("application-id")
            .unwrap()
            .to_owned();
        
        // Attempt to get the application.
        let application = api_service.get_application(application_id).unwrap();
        
        to_value(application).unwrap()
    }
}

struct GetApplicationVersions {}

impl CommandHandler for GetApplicationVersions {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameters.
        let application_id: i32 = matches.get_one::<i32>("application-id")
            .unwrap()
            .to_owned();
        let platform: String = matches.get_one::<String>("platform")
            .unwrap()
            .to_owned();
        
        // Attempt to get the application versions for the specified platform.
        let application_versions = api_service
            .get_application_versions(application_id, platform)
            .unwrap();
        
        to_value(application_versions).unwrap()
    }
}

struct GetSpecificApplicationVersion {}

impl CommandHandler for GetSpecificApplicationVersion {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let version_id: i32 = matches.get_one::<i32>("version-id")
            .unwrap()
            .to_owned();
        
        let application_version = api_service.get_application_version(version_id)
            .unwrap();
        
        to_value(application_version).unwrap()
    }
}

struct GetApplicationVersionFor {}

impl CommandHandler for GetApplicationVersionFor {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameters.
        let application_id: i32 = matches.get_one::<i32>("application-id")
            .unwrap()
            .to_owned();
        
        // TODO: Deal with this.
        // let platform: String = matches.get_one::<String>("platform")
        //     .unwrap()
        //     .to_owned();
        
        let application_version = 
            api_service.get_application_version(application_id)
                .unwrap();
        
        to_value(application_version).unwrap()
    }
}

struct GetFineTunedApplicationVersion {}

impl CommandHandler for GetFineTunedApplicationVersion {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameters.
        let application_id: i32 = matches.get_one::<i32>("application-id")
            .unwrap()
            .to_owned();
        
        let version_name: String = matches.get_one::<String>("version-name")
            .unwrap()
            .to_owned();
        
        let platform: String = matches.get_one::<String>("platform")
            .unwrap()
            .to_owned();
        
        let version = api_service.get_application_version_for(
            application_id,
            version_name,
            platform
        ).unwrap();
        
        to_value(version).unwrap()
    }
}

struct UpdateApplicationVersion {}

impl CommandHandler for UpdateApplicationVersion {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameters.
        let application_id: i32 = matches.get_one::<i32>("application-id")
            .unwrap()
            .to_owned();

        let version_name: String = matches.get_one::<String>("version-name")
            .unwrap()
            .to_owned();
        
        // Update the application version.
        let response = 
            api_service.update_application_version(application_id, version_name);
        
        json!({
            "success": response.is_ok()
        })
    }
}

struct CreateApplicationVersion {}

impl CommandHandler for CreateApplicationVersion {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameters.
        let application_id: i32 = matches.get_one::<i32>("application-id")
            .unwrap()
            .to_owned();
        let name: String = matches.get_one::<String>("name")
            .unwrap()
            .to_owned();
        let platform: String = matches.get_one::<String>("platform")
            .unwrap()
            .to_owned();
        let release_date: String = matches.get_one::<String>("release-date")
            .unwrap()
            .to_owned();
        let filename: String = matches.get_one::<String>("filename")
            .unwrap()
            .to_owned();
        let executable: String = matches.get_one::<String>("executable")
            .unwrap()
            .to_owned();
        let filepath: String = matches.get_one::<String>("file")
            .unwrap()
            .to_owned();
        
        let response = api_service.create_application_version(
            application_id,
            name,
            platform,
            release_date,
            filename,
            executable,
            filepath
        );
        
        json!({
            "success": response.is_ok()
        })
    }
}

struct CreateSale {}

impl CommandHandler for CreateSale {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameters. 
        let application_id: i32 = matches.get_one::<i32>("application-id")
            .unwrap()
            .to_owned();
        let title: String = matches.get_one::<String>("title")
            .unwrap()
            .to_owned();
        let description: String = matches.get_one::<String>("description")
            .unwrap()
            .to_owned();
        let price: f32 = matches.get_one::<f32>("price")
            .unwrap()
            .to_owned();
        let start_date: String = matches.get_one::<String>("start-date")
            .unwrap()
            .to_owned();
        let end_date: String = matches.get_one::<String>("end-date")
            .unwrap()
            .to_owned();
        
        let response = api_service.create_sale(
            application_id,
            title,
            description,
            price,
            start_date,
            end_date
        );
        
        json!({
            "success": response.is_ok()
        })
    }
}

struct GetActiveSale {}

impl CommandHandler for GetActiveSale {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let application_id: i32 = matches.get_one::<i32>("application-id")
            .unwrap()
            .to_owned();
        
        let active_sale = api_service.get_active_sale(application_id).unwrap();
        
        to_value(active_sale).unwrap()
    }
}

struct GetAllSales {}

impl CommandHandler for GetAllSales {
    fn handle_command(api_service: ApiService, _matches: &ArgMatches) -> Value {
        let sales = api_service.get_all_sales().unwrap();
        
        to_value(sales).unwrap()
    }
}

struct DeleteSale {}

impl CommandHandler for DeleteSale {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let sale_id: i32 = matches.get_one::<i32>("sale-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.delete_sale(sale_id);
        
        json!({
            "success": response.is_ok()
        })
    }
}

struct GetUserTransactions {}

impl CommandHandler for GetUserTransactions {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let user_id: i32 = matches.get_one::<i32>("user-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.get_user_transactions(user_id).unwrap();
        
        to_value(response).unwrap()
    }
}

struct GetTransaction {}

impl CommandHandler for GetTransaction {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let transaction_id: i32 = matches.get_one::<i32>("transaction-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.get_transaction(transaction_id).unwrap();
        
        to_value(response).unwrap()
    }
}

struct GetPurchase {}

impl CommandHandler for GetPurchase {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let purchase_id: i32 = matches.get_one::<i32>("purchase-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.get_purchase(purchase_id).unwrap();
        
        to_value(response).unwrap()
    }
}

struct GetDeposit {}

impl CommandHandler for GetDeposit {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let deposit_id: i32 = matches.get_one::<i32>("deposit-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.get_deposit(deposit_id).unwrap();
        
        to_value(response).unwrap()
    }
}

struct GetApplicationKey {}

impl CommandHandler for GetApplicationKey {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let key: String = matches.get_one::<String>("key")
            .unwrap()
            .to_owned();
        
        let response = api_service.get_application_key(key).unwrap();
        
        to_value(response).unwrap()
    }
}

struct GetUserApplicationKeys {}

impl CommandHandler for GetUserApplicationKeys {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let user_id: i32 = matches.get_one::<i32>("user-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.get_user_application_keys(user_id).unwrap();
        
        to_value(response).unwrap()
    }
}

struct PurchaseApplication {}

impl CommandHandler for PurchaseApplication {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let application_id: i32 = matches.get_one::<i32>("application-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.purchase_application(application_id);
        
        json!({
            "success": response.is_ok()
        })
    }
}

struct PurchaseIap {}

impl CommandHandler for PurchaseIap {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let iap_id: i32 = matches.get_one::<i32>("iap-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.purchase_iap(iap_id);
        
        json!({
            "success": response.is_ok()
        })
    }
}

struct GetIapRecords {}

impl CommandHandler for GetIapRecords {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameters.
        let user_id: i32 = matches.get_one::<i32>("user-id")
            .unwrap()
            .to_owned();
        let application_id: i32 = matches.get_one::<i32>("application-id")
            .unwrap()
            .to_owned();
        let only_unacknowledged: bool = matches.contains_id("only-unacknowledged");
        
        let response = api_service.get_iap_records(
            user_id,
            application_id,
            only_unacknowledged
        ).unwrap();
        
        to_value(response).unwrap()
    }
}

struct GetSession {}

impl CommandHandler for GetSession {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let session_id: String = matches.get_one::<String>("session-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.get_session(session_id).unwrap();
        
        to_value(response).unwrap()
    }
}

struct SendFriendRequest {}

impl CommandHandler for SendFriendRequest {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let user_id: i32 = matches.get_one::<i32>("user-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.send_friend_request(user_id);
        
        json!({
            "success": response.is_ok()
        })
    }
}

struct DeleteFriendRequest {}

impl CommandHandler for DeleteFriendRequest {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let request_id: i32 = matches.get_one::<i32>("request-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.delete_friend_request(request_id);
        
        json!({
            "success": response.is_ok()
        })
    }
}

struct GetIncomingFriendRequests {}

impl CommandHandler for GetIncomingFriendRequests {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let user_id: i32 = matches.get_one::<i32>("user-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.get_incoming_friend_requests(user_id).unwrap();
        
        to_value(response).unwrap()
    }
}

struct GetOutgoingFriendRequests {}

impl CommandHandler for GetOutgoingFriendRequests {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let user_id: i32 = matches.get_one::<i32>("user-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.get_outgoing_friend_requests(user_id).unwrap();
        
        to_value(response).unwrap()
    }
}

struct AcceptFriendRequest {}

impl CommandHandler for AcceptFriendRequest {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        // Get the parameter.
        let request_id: i32 = matches.get_one::<i32>("request-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.accept_friend_request(request_id);
        
        json!({
            "success": response.is_ok()
        })
    }
}

struct GetFriends {}

impl CommandHandler for GetFriends {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        let user_id: i32 = matches.get_one::<i32>("user-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.get_friends(user_id).unwrap();
        
        to_value(response).unwrap()
    }
}

struct RemoveFriend {}

impl CommandHandler for RemoveFriend {
    fn handle_command(api_service: ApiService, matches: &ArgMatches) -> Value {
        let user_id: i32 = matches.get_one::<i32>("user-id")
            .unwrap()
            .to_owned();
        
        let response = api_service.remove_friend(user_id);
        
        json!({
            "success": response.is_ok()
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonResponse<T> {
    time: f64,
    response: T
}

fn timed_response<T, F>(request_logic: F) -> Value
where 
    F: FnOnce() -> T,
    T: Serialize,
{
    let start: Instant = Instant::now();
    let response: T = request_logic();
    let duration: Duration = start.elapsed();
    
    let json_response = JsonResponse {
        time: duration.as_secs_f64(),
        response
    };
    
    to_value(&json_response).unwrap()
}

fn handle<T, F>(request_logic: F)
where
    F: FnOnce() -> T,
    T: Serialize
{
    let value: Value = timed_response(request_logic);
    
    println!("{}", to_string_pretty(&value).unwrap());
}

fn main() {
    // Debug session ids:
    //  - SlimyFrog123: b5eadd7911364cb98e162acc163a73c1
    //  - DragonMinecart303: d210bd70f62040afa7a78b16d003e89b
    let command: Command = Command::new(USER_AGENT_STRING)
        .author("SlimyFrog123")
        .version(APPLICATION_VERSION)
        .about("CLI interface for the Frogworks backend.")
        .subcommand_required(true)
        .arg(
            Arg::new("session-id")
                .help("The Frogworks session id. Required for anything other than pinging, registering, and logging in.")
                .long("session-id")
                .value_parser(value_parser!(String))
        )
        .subcommand(
            Command::new("server")
                .long_flag("server")
                .subcommand_required(true)
                .subcommand(
                    Command::new("ping")
                        .long_flag("ping")
                )
        )
        .subcommand(
            Command::new("account")
                .long_flag("account")
                .subcommand_required(true)
                .subcommand(
                    Command::new("login")
                        .long_flag("login")
                        .arg(
                            Arg::new("username")
                                .long("username")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("password")
                                .long("password")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                )
                .subcommand(
                    Command::new("register")
                        .long_flag("register")
                        .arg(
                            Arg::new("username")
                                .long("username")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("name")
                                .long("name")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("email-address")
                                .long("email-address")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("password")
                                .long("password")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("email-verification-code")
                                .long("email-verification-code")
                                .value_parser(value_parser!(i32))
                                .required(true)
                        )
                )
        )
        .subcommand(
            Command::new("email")
                .long_flag("email")
                .subcommand_required(true)
                .subcommand(
                    Command::new("verification")
                        .long_flag("verification")
                        .subcommand_required(true)
                        .subcommand(
                            Command::new("request")
                                .long_flag("request")
                                .arg(
                                    Arg::new("email-address")
                                        .long("email-address")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("check")
                                .long_flag("check")
                                .arg(
                                    Arg::new("email-address")
                                        .long("email-address")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("verification-code")
                                        .long("verification-code")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                )
        )
        .subcommand(
            Command::new("session")
                .long_flag("session")
                .subcommand_required(true)
                .subcommand(
                    Command::new("authenticate")
                        .long_flag("authenticate")
                )
                .subcommand(
                    Command::new("delete")
                        .long_flag("delete")
                        .arg(
                            Arg::new("session-id")
                                .long("session-id")
                                .value_parser(value_parser!(i32))
                        )
                )
                .subcommand(
                    Command::new("get")
                        .long_flag("get")
                        .arg(
                            Arg::new("session-id")
                                .long("session-id")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                )
        )
        .subcommand(
            Command::new("user")
                .long_flag("user")
                .subcommand_required(true)
                .subcommand(
                    Command::new("get")
                        .long_flag("get")
                        .arg(
                            Arg::new("identifier")
                                .long("identifier")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("identifier-type")
                                .long("identifier-type")
                                .value_parser(value_parser!(String))
                                .default_value("identifier")
                        )
                )
                .subcommand(
                    Command::new("properties")
                        .long_flag("properties")
                        .subcommand_required(true)
                        .subcommand(
                            Command::new("get")
                                .long_flag("get")
                                .subcommand_required(true)
                                .subcommand(
                                    Command::new("iap-records")
                                        .long_flag("iap-records")
                                        .arg(
                                            Arg::new("user-id")
                                                .long("user-id")
                                                .value_parser(value_parser!(i32))
                                                .required(true)
                                        )
                                        .arg(
                                            Arg::new("application-id")
                                                .long("application-id")
                                                .value_parser(value_parser!(i32))
                                                .required(true)
                                        )
                                        .arg(
                                            Arg::new("only-unacknowledged")
                                                .long("only-unacknowledged")
                                                .required(false)
                                                .num_args(0)
                                        )
                                )
                        )
                )
        )
        .subcommand(
            Command::new("application")
                .long_flag("application")
                .subcommand_required(true)
                .subcommand(
                    Command::new("create")
                        .long_flag("create")
                        .arg(
                            Arg::new("name")
                                .long("name")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("package-name")
                                .long("package-name")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("application-type")
                                .long("application-type")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("description")
                                .long("description")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("release-date")
                                .long("release-date")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("early-access")
                                .long("early-access")
                                .value_parser(value_parser!(bool))
                                .required(true)
                        )
                        .arg(
                            Arg::new("supported-platforms")
                                .long("supported-platforms")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("genres")
                                .long("genres")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("tags")
                                .long("tags")
                                .value_parser(value_parser!(String))
                                .required(true)
                        )
                        .arg(
                            Arg::new("base-price")
                                .long("base-price")
                                .value_parser(value_parser!(f32))
                                .required(true)
                        )
                )
                .subcommand(
                    Command::new("get")
                        .long_flag("get")
                        .arg(
                            Arg::new("application-id")
                                .long("application-id")
                                .value_parser(value_parser!(i32))
                                .required(true)
                        )
                )
                .subcommand(
                    Command::new("version")
                        .long_flag("version")
                        .subcommand_required(true)
                        .subcommand(
                            Command::new("get-for")
                                .long_flag("get-for")
                                .arg(
                                    Arg::new("application-id")
                                        .long("application-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("platform")
                                        .long("platform")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("get")
                                .long_flag("get")
                                .arg(
                                    Arg::new("version-id")
                                        .long("version-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("get-fine-tuned")
                                .long_flag("get-fine-tuned")
                                .arg(
                                    Arg::new("application-id")
                                        .long("application-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("version-name")
                                        .long("version-name")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("platform")
                                        .long("platform")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("get-list")
                                .long_flag("get-list")
                                .arg(
                                    Arg::new("application-id")
                                        .long("application-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("update")
                                .long_flag("update")
                                .arg(
                                    Arg::new("application-id")
                                        .long("application-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("version-name")
                                        .long("version-name")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("create")
                                .long_flag("create")
                                .arg(
                                    Arg::new("application-id")
                                        .long("application-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("name")
                                        .long("name")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("platform")
                                        .long("platform")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("release-date")
                                        .long("release-date")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("filename")
                                        .long("filename")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("executable")
                                        .long("executable")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("file")
                                        .long("file")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                        )
                )
                .subcommand(
                    Command::new("sale")
                        .long_flag("sale")
                        .subcommand_required(true)
                        .subcommand(
                            Command::new("create")
                                .long_flag("create")
                                .arg(
                                    Arg::new("application-id")
                                        .long("application-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("title")
                                        .long("title")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("description")
                                        .long("description")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("price")
                                        .long("price")
                                        .value_parser(value_parser!(f32))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("start-date")
                                        .long("start-date")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                                .arg(
                                    Arg::new("end-date")
                                        .long("end-date")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("get-active")
                                .long_flag("get-active")
                                .arg(
                                    Arg::new("application-id")
                                        .long("application-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("get-all")
                                .long_flag("get-all")
                        )
                        .subcommand(
                            Command::new("delete")
                                .long_flag("delete")
                                .arg(
                                    Arg::new("sale-id")
                                        .long("sale-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                )
                .subcommand(
                    Command::new("key")
                        .long_flag("key")
                        .subcommand_required(true)
                        .subcommand(
                            Command::new("get")
                                .long_flag("get")
                                .arg(
                                    Arg::new("key")
                                        .long("key")
                                        .value_parser(value_parser!(String))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("get-list-for")
                                .long_flag("get-list-for")
                                .arg(
                                    Arg::new("user-id")
                                        .long("user-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                )
        )
        .subcommand(
            Command::new("payment")
                .long_flag("payment")
                .subcommand_required(true)
                .subcommand(
                    Command::new("get")
                        .long_flag("get")
                        .subcommand_required(true)
                        .subcommand(
                            Command::new("user-transactions")
                                .arg(
                                    Arg::new("user-id")
                                        .long("user-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("transaction")
                                .arg(
                                    Arg::new("transaction-id")
                                        .long("transaction-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("purchase")
                                .arg(
                                    Arg::new("purchase-id")
                                        .long("purchase-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("deposit")
                                .arg(
                                    Arg::new("deposit-id")
                                        .long("deposit-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                )
                .subcommand(
                    Command::new("buy")
                        .long_flag("buy")
                        .subcommand_required(true)
                        .subcommand(
                            Command::new("application")
                                .long_flag("application")
                                .arg(
                                    Arg::new("application-id")
                                        .long("application-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("iap")
                                .long_flag("iap")
                                .arg(
                                    Arg::new("iap-id")
                                        .long("iap-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                )
        )
        .subcommand(
            Command::new("friend")
                .long_flag("friend")
                .subcommand_required(true)
                .subcommand(
                    Command::new("request")
                        .long_flag("request")
                        .subcommand_required(true)
                        .subcommand(
                            Command::new("send")
                                .arg(
                                    Arg::new("user-id")
                                        .long("user-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("delete")
                                .arg(
                                    Arg::new("request-id")
                                        .long("request-id")
                                        .value_parser(value_parser!(i32))
                                        .required(true)
                                )
                        )
                        .subcommand(
                            Command::new("get")
                                .long_flag("get")
                                .subcommand_required(true)
                                .subcommand(
                                    Command::new("incoming")
                                        .long_flag("incoming")
                                        .arg(
                                            Arg::new("user-id")
                                                .long("user-id")
                                                .value_parser(value_parser!(i32))
                                                .required(true)
                                        )
                                )
                                .subcommand(
                                    Command::new("outgoing")
                                        .long_flag("outgoing")
                                        .arg(
                                            Arg::new("user-id")
                                                .long("user-id")
                                                .value_parser(value_parser!(i32))
                                                .required(true)
                                        )
                                )
                        )
                )
                .subcommand(
                    Command::new("get-list")
                        .long_flag("get-list")
                        .arg(
                            Arg::new("user-id")
                                .long("user-id")
                                .value_parser(value_parser!(i32))
                                .required(true)
                        )
                )
                .subcommand(
                    Command::new("remove")
                        .long_flag("remove")
                        .arg(
                            Arg::new("user-id")
                                .long("user-id")
                                .value_parser(value_parser!(i32))
                                .required(true)
                        )
                )
        );
    
    let matches: ArgMatches = command.get_matches();
    
    let mut api_service: ApiService = ApiService::new("http://192.168.1.16/".to_string());
    
    if let Some(session_id) = matches.get_one::<String>("session-id") { 
        api_service = api_service.with_authentication(session_id.to_owned());
    }
    
    match matches.subcommand() {
        Some(("server", server_matches)) => {
            match server_matches.subcommand() {
                Some(("ping", matches)) => {
                    handle(|| Ping::handle_command(api_service, &matches));
                },
                _ => {}
            }
        },
        Some(("account", account_matches)) => {
            match account_matches.subcommand() {
                Some(("login", login_matches)) => {
                    handle(|| Login::handle_command(api_service, &login_matches));
                },
                Some(("register", register_matches)) => {
                    handle(|| Register::handle_command(api_service, &register_matches));
                },
                _ => {}
            }
        },
        Some(("session", session_matches)) => {
            match session_matches.subcommand() { 
                Some(("authenticate", session_matches)) => {
                    handle(|| AuthenticateSession::handle_command(api_service, session_matches));
                },
                Some(("delete", session_matches)) => {
                    handle(|| DeleteSession::handle_command(api_service, session_matches));
                },
                Some(("get", session_matches)) => {
                    handle(|| GetSession::handle_command(api_service, session_matches))
                },
                _ => {}
            }
        },
        Some(("email", email_matches)) => {
            match email_matches.subcommand() {
                Some(("verification", verification_matches)) => {
                    match verification_matches.subcommand() {
                        Some(("request", verification_matches)) => {
                            handle(|| RequestEmailVerification::handle_command(
                                api_service, verification_matches));
                        },
                        Some(("check", verification_matches)) => {
                            handle(|| CheckEmailVerification::handle_command(
                                api_service, verification_matches));
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        },
        Some(("user", user_matches)) => {
            match user_matches.subcommand() { 
                Some(("get", get_matches)) => {
                    handle(|| GetUser::handle_command(api_service, get_matches));
                },
                Some(("properties", properties_matches)) => {
                    match properties_matches.subcommand() {
                        Some(("get", get_matches)) => {
                            match get_matches.subcommand() {
                                Some(("iap-records", matches)) => {
                                    handle(|| GetIapRecords::handle_command(api_service,
                                                                            matches));
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        },
        Some(("application", application_matches)) => {
            match application_matches.subcommand() {
                Some(("create", create_matches)) => {
                    handle(|| CreateApplication::handle_command(api_service, create_matches));
                },
                Some(("get", get_matches)) => {
                    handle(|| GetApplication::handle_command(api_service, get_matches));
                },
                Some(("version", version_matches)) => {
                    match version_matches.subcommand() { 
                        Some(("get-for", get_matches)) => {
                            handle(|| GetApplicationVersionFor::handle_command(api_service,
                                                                               get_matches));
                        },
                        Some(("get", get_matches)) => {
                            handle(|| GetSpecificApplicationVersion::handle_command(api_service,
                                                                                    get_matches));
                        }
                        Some(("get-fine-tuned", get_matches)) => {
                            handle(|| GetFineTunedApplicationVersion::handle_command(api_service,
                                                                                     get_matches));
                        },
                        Some(("get-list", get_matches)) => {
                            handle(|| GetApplicationVersions::handle_command(api_service, 
                                                                             get_matches));
                        },
                        Some(("update", update_matches)) => {
                            handle(|| UpdateApplicationVersion::handle_command(api_service,
                                                                               update_matches));
                        },
                        Some(("create", create_matches)) => {
                            handle(|| CreateApplicationVersion::handle_command(api_service, 
                                                                               create_matches));
                        },
                        _ => {}
                    }
                },
                Some(("sale", sale_matches)) => {
                    match sale_matches.subcommand() {
                        Some(("create", create_matches)) => {
                            handle(|| CreateSale::handle_command(api_service, create_matches));
                        },
                        Some(("get-active", matches)) => {
                            handle(|| GetActiveSale::handle_command(api_service, matches));
                        },
                        Some(("get-all", matches)) => {
                            handle(|| GetAllSales::handle_command(api_service, matches));
                        },
                        Some(("delete", matches)) => {
                            handle(|| DeleteSale::handle_command(api_service, matches))
                        }
                        _ => {}
                    }
                },
                Some(("key", key_matches)) => {
                    match key_matches.subcommand() {
                        Some(("get", get_matches)) => {
                            handle(|| GetApplicationKey::handle_command(api_service, get_matches));
                        },
                        Some(("get-list-for", matches)) => {
                            handle(|| GetUserApplicationKeys::handle_command(api_service, matches));
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        },
        Some(("payment", payment_matches)) => {
            match payment_matches.subcommand() {
                Some(("get", get_matches)) => {
                    match get_matches.subcommand() { 
                        Some(("user-transactions", matches)) => {
                            handle(|| GetUserTransactions::handle_command(api_service, matches));
                        },
                        Some(("transaction", matches)) => {
                            handle(|| GetTransaction::handle_command(api_service, matches));
                        },
                        Some(("purchase", matches)) => {
                            handle(|| GetPurchase::handle_command(api_service, matches));
                        },
                        Some(("deposit", matches)) => {
                            handle(|| GetDeposit::handle_command(api_service, matches));
                        },
                        _ => {}
                    }
                },
                Some(("buy", buy_matches)) => {
                    match buy_matches.subcommand() {
                        Some(("application", matches)) => {
                            handle(|| PurchaseApplication::handle_command(api_service, matches));
                        },
                        Some(("iap", matches)) => {
                            handle(|| PurchaseApplication::handle_command(api_service, matches));
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        },
        Some(("friend", friend_matches)) => {
            match friend_matches.subcommand() { 
                Some(("request", request_matches)) => {
                    match request_matches.subcommand() {
                        Some(("send", matches)) => {
                            handle(|| SendFriendRequest::handle_command(api_service, matches));
                        },
                        Some(("delete", matches)) => {
                            handle(|| DeleteFriendRequest::handle_command(api_service, matches));
                        },
                        Some(("get", get_matches)) => {
                            match get_matches.subcommand() {
                                Some(("incoming", incoming_matches)) => {
                                    handle(|| GetIncomingFriendRequests::handle_command(
                                        api_service, incoming_matches));
                                },
                                Some(("outgoing", outgoing_matching)) => {
                                    handle(|| GetOutgoingFriendRequests::handle_command(
                                        api_service, outgoing_matching));
                                },
                                _ => {}
                            }
                        },
                        Some(("accept", accept_matches)) => {
                            handle(|| AcceptFriendRequest::handle_command(api_service, 
                                                                          accept_matches));
                        },
                        _ => {}
                    }
                },
                Some(("get-list", matches)) => {
                    handle(|| GetFriends::handle_command(api_service, matches));
                },
                Some(("remove", matches)) => {
                    handle(|| RemoveFriend::handle_command(api_service, matches));
                },
                _ => {}
            }
        },
        _ => {},
    }
}