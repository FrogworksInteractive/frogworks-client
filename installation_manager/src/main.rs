use std::{io, process};
use std::path::{Path, PathBuf};
use clap::{Arg, Command, ArgMatches, ValueEnum, value_parser};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;

const SCHEME: &str = "frogworks";

#[derive(ValueEnum, Clone, Debug)]
enum Operation {
    Install,
    Uninstall
}

fn main() {
    let command: Command = Command::new("Frogworks Installation Manger")
        .version("0.1.0")
        .author("SlimyFrog123 <danieldcookjr@gmail.com>")
        .about("Handles the installation and uninstallation of Frogworks.")
        .arg(Arg::new("operation")
            .short('o')
            .long("operation")
            .value_parser(value_parser!(Operation))
            .required(true)
            .help("Specify the operation to perform"))
        .arg(Arg::new("installation-directory")
            .long("installation-directory")
            .value_parser(value_parser!(String))
            .help("Specify the installation directory (if installing)"));

    let matches: ArgMatches = command.get_matches();
    let operation = matches.get_one::<Operation>("operation").unwrap();
    let installation_directory =
        matches.get_one::<String>("installation-directory");

    match operation {
        Operation::Install => {
            if installation_directory.is_none() {
                eprintln!("Error: --installation-directory must be specified when installing.");
                process::exit(1);
            }

            install(installation_directory.unwrap());
        },
        Operation::Uninstall => uninstall().expect("Failed to uninstall."),
    }
}

fn error_out(details: &str) {
    eprintln!("Failed to install Frogworks.\n\nDetails: {}", details);
}

fn install(installation_directory: &str) {
    println!("Installing {}", installation_directory);

    // Generate the installation paths.
    let base_path: &Path = Path::new(installation_directory);
    let executable_path: &PathBuf = &base_path.join("frogworks.exe");
    let cli_path: &PathBuf = &base_path.join("cli.exe");
    let daemon_path: &PathBuf = &base_path.join("daemon.exe");

    // TODO: Copy over the files to the installation directory.

    // Create the registry keys.
    let registry_keys: io::Result<()> = create_registry_keys(
        executable_path.to_str().unwrap(),
        cli_path.to_str().unwrap(),
        daemon_path.to_str().unwrap(),
        base_path.to_str().unwrap()
    );

    if registry_keys.is_err() {
        error_out("Failed to create registry keys.");
        uninstall_registry_keys();

        process::exit(1);
    }

    // Register the URI scheme.
    let uri_scheme: io::Result<()> = register_uri_scheme(daemon_path.to_str().unwrap());

    if uri_scheme.is_err() {
        error_out("Failed to register URI scheme.");
        uninstall_registry_keys();

        process::exit(1);
    }

}

fn create_registry_keys(executable_path: &str, cli_path: &str, daemon_path: &str, installation_directory: &str) -> io::Result<()> {
    // Open or create the HKEY_CURRENT_USER\SOFTWARE\Frogworks subkey.
    let hkey_current_user: RegKey = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (frogworks_key, _) = hkey_current_user.create_subkey("Frogworks")?;

    // Set the main executable path.
    frogworks_key.set_value("MainExecutablePath", &executable_path)
        .expect("Failed to set the main executable path.");

    // Set the cli path.
    frogworks_key.set_value("CLIPath", &cli_path)
        .expect("Failed to set the cli path.");

    // Set the daemon path.
    frogworks_key.set_value("DaemonPath", &daemon_path)
        .expect("Failed to set the daemon path.");

    // Set the installation directory.
    frogworks_key.set_value("InstallationPath", &installation_directory)
        .expect("Failed to set installation path.");

    Ok(())
}

fn register_uri_scheme(daemon_path: &str) -> io::Result<()> {
    let hkey_current_user: RegKey = RegKey::predef(HKEY_LOCAL_MACHINE);

    // Create the scheme key under HKEY_CURRENT_USER\Software\Classes\<scheme>.
    let (key, _) =
        hkey_current_user.create_subkey(format!("Software\\Classes\\{}", SCHEME))?;

    // Set the default value to describe the protocol.
    key.set_value("", &format!("URL:{} Protocol", SCHEME))?;

    // Create and set the "URL Protocol" value (must be empty).
    key.set_value("URL Protocol", &"")?;

    // Create the command key to handle the execution.
    let (command_key, _) = key.create_subkey("shell\\open\\command")?;

    // Set the default value to point to the daemon executable with "%1" as an argument.
    command_key.set_value("", &format!(r#""{}" "%1""#, daemon_path))?;

    Ok(())
}

fn uninstall() -> io::Result<()> {
    // Get the installation path.
    let hkey_current_user: RegKey = RegKey::predef(HKEY_CURRENT_USER);
    let frogworks_key = hkey_current_user.open_subkey("Software\\Frogworks")?;

    // Get the installation directory.
    let installation_directory: String = frogworks_key.get_value("InstallationPath")?;

    // Take care of the registry keys.
    uninstall_registry_keys();

    // Remove the installation directory.

    Ok(())
}

fn uninstall_registry_keys() {
    remove_registry_keys().expect("Failed to remove registry keys.");
    unregister_uri_scheme().expect("Failed to unregister URI scheme.");
}

fn remove_registry_keys() -> io::Result<()> {
    let hkey_current_user: RegKey = RegKey::predef(HKEY_CURRENT_USER);
    hkey_current_user.delete_subkey_all("Software\\Frogworks")?;

    Ok(())
}

fn unregister_uri_scheme() -> io::Result<()> {
    let hkey_current_user: RegKey = RegKey::predef(HKEY_CURRENT_USER);

    // Delete the scheme key under HKEY_CURRENT_USER\Software\Classes\<scheme>
    hkey_current_user.delete_subkey_all(format!("Software\\Classes\\{}", SCHEME))?;

    Ok(())
}
