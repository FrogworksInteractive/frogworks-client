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
                _ => {}
            }
        },
        Some(("application", application_matches)) => {
            match application_matches.subcommand() {
                Some(("create", create_matches)) => {
                    handle(|| CreateApplication::handle_command(api_service, create_matches));
                },
                _ => {}
            }
        },
        _ => {},
    }
}