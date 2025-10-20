#![cfg(feature = "cli")]

use clap::{Parser, Subcommand};
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::fmt;
use std::fs::OpenOptions;

use zeromq::ReqSocket; // or DealerSocket, RouterSocket, etc.
use zeromq::ZmqMessage;
use zeromq::prelude::*; // traits

// Constants for paths and configuration
const DAEMON_SOCKET_PATH: &str = "ipc:///var/run/regmsgd.sock";
const LOG_FILE_PATH: &str = "/var/log/regmsg.log";

/// Custom error type for CLI operations
#[derive(Debug)]
enum CliError {
    SocketError(zeromq::ZmqError),
    Utf8Error(std::string::FromUtf8Error),
    IoError(std::io::Error),
    LogError(log::SetLoggerError),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::SocketError(e) => write!(f, "Socket error: {}", e),
            CliError::Utf8Error(e) => write!(f, "UTF-8 error: {}", e),
            CliError::IoError(e) => write!(f, "IO error: {}", e),
            CliError::LogError(e) => write!(f, "Log error: {}", e),
        }
    }
}

impl std::error::Error for CliError {}

impl From<zeromq::ZmqError> for CliError {
    fn from(error: zeromq::ZmqError) -> Self {
        CliError::SocketError(error)
    }
}

impl From<std::string::FromUtf8Error> for CliError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        CliError::Utf8Error(error)
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::IoError(error)
    }
}

impl From<log::SetLoggerError> for CliError {
    fn from(error: log::SetLoggerError) -> Self {
        CliError::LogError(error)
    }
}

/// Global CLI arguments
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Target screen identifier (optional)
    #[arg(short, long)]
    screen: Option<String>,

    /// Enable terminal logging
    #[arg(short, long)]
    log: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,

    /// Additional arguments passed to the daemon
    #[arg(last = true)]
    args: Vec<String>,
}

/// List of available subcommands
#[derive(Subcommand, Debug)]
#[command(rename_all = "camelCase")] // <--- all variants become camelCase
enum Commands {
    #[command(about = "Lists all available outputs (e.g., HDMI, VGA).")]
    ListModes,
    #[command(about = "List all available display outputs")]
    ListOutputs,
    #[command(about = "Displays the current display mode for the specified screen.")]
    CurrentMode,
    #[command(about = "Displays the current output (e.g., HDMI, VGA).")]
    CurrentOutput,
    #[command(about = "Displays the current resolution for the specified screen.")]
    CurrentResolution,
    #[command(about = "Displays the current screen rotation for the specified screen.")]
    CurrentRotation,
    #[command(about = "Displays the current refresh rate for the specified screen.")]
    CurrentRefresh,
    #[command(about = "Displays the current window system.")]
    CurrentBackend,
    #[command(about = "Sets the display mode for the specified screen.")]
    SetMode { mode: String },
    #[command(about = "Sets the output resolution and refresh rate (e.g., WxH@R or WxH).")]
    SetOutput { output: String },
    #[command(about = "Sets the screen rotation for the specified screen.")]
    SetRotation {
        #[arg(value_parser = ["0", "90", "180", "270"])]
        rotation: String,
    },
    #[command(about = "Takes a screenshot of the current screen.")]
    GetScreenshot,
    #[command(about = "Maps the touchscreen to the correct display.")]
    MapTouchScreen,
    #[command(
        about = "Sets the screen resolution to the maximum supported resolution (e.g., 1920x1080)."
    )]
    MinToMaxResolution,
}

/// Configure file and terminal logging
///
/// # Arguments
/// * `enable_terminal` - If true, enables terminal logging in addition to file logging
///
/// # Returns
/// * `Ok(())` - If logging was initialized successfully
/// * `Err(CliError)` - If an error occurred during initialization
fn init_logging(enable_terminal: bool) -> Result<(), CliError> {
    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = vec![create_file_logger()?];

    if enable_terminal {
        loggers.push(create_terminal_logger());
    }

    CombinedLogger::init(loggers).map_err(CliError::LogError)
}

/// Creates a file logger
///
/// # Returns
/// * `Ok(Box<dyn simplelog::SharedLogger>)` - A file logger ready to be used
/// * `Err(CliError)` - If an error occurred while opening the file
fn create_file_logger() -> Result<Box<dyn simplelog::SharedLogger>, CliError> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE_PATH)?;

    Ok(WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        file,
    ))
}

/// Creates a terminal logger
///
/// # Returns
/// * `Box<dyn simplelog::SharedLogger>` - A terminal logger ready to be used
fn create_terminal_logger() -> Box<dyn simplelog::SharedLogger> {
    TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
}

/// Main entry point of the CLI application
///
/// Parses command line arguments, initializes logging,
/// connects to the daemon and executes the requested command.
///
/// # Returns
/// * `Ok(())` - If the application executed successfully
/// * `Err(CliError)` - If an error occurred during execution
#[async_std::main]
async fn main() -> Result<(), CliError> {
    let cli = Cli::parse();

    // Init logging
    init_logging(cli.log)?;

    // Connect to daemon via ZeroMQ
    //let ctx = zmq::Context::new();
    let mut socket = ReqSocket::new();
    let _ = socket.connect(DAEMON_SOCKET_PATH).await;

    // Execute the command
    if let Err(e) = handle_command(&cli, socket).await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    Ok(())
}

/// Execute the selected subcommand by sending a request to the daemon
///
/// # Arguments
/// * `cli` - The parsed command line arguments
/// * `socket` - The ZeroMQ socket to communicate with the daemon
///
/// # Returns
/// * `Ok(())` - If the command was executed successfully
/// * `Err(CliError)` - If an error occurred during execution
async fn handle_command(cli: &Cli, mut socket: zeromq::ReqSocket) -> Result<(), CliError> {
    // Build the complete command
    let cmd = build_command_string(cli);

    // Send the command to the daemon
    let _ = socket.send(ZmqMessage::from(cmd)).await;

    // Receive and display
    let reply = socket.recv().await?;

    // Get the first frame as a UTF-8 string
    let reply_str = match reply.get(0) {
        Some(frame) => String::from_utf8(frame.to_vec())?,
        None => String::new(),
    };

    println!("{}", reply_str); // prints the raw string

    Ok(())
}

/// Build the complete command string to send to the daemon
///
/// # Arguments
/// * `cli` - The parsed command line arguments
///
/// # Returns
/// * `String` - The complete formatted command to send to the daemon
fn build_command_string(cli: &Cli) -> String {
    let mut cmd = match &cli.command {
        Commands::ListModes => "listModes".to_string(),
        Commands::ListOutputs => "listOutputs".to_string(),
        Commands::CurrentMode => "currentMode".to_string(),
        Commands::CurrentOutput => "currentOutput".to_string(),
        Commands::CurrentResolution => "currentResolution".to_string(),
        Commands::CurrentRotation => "currentRotation".to_string(),
        Commands::CurrentRefresh => "currentRefresh".to_string(),
        Commands::CurrentBackend => "currentBackend".to_string(),
        Commands::SetMode { mode } => format!("setMode {}", mode),
        Commands::SetOutput { output } => format!("setOutput {}", output),
        Commands::SetRotation { rotation } => format!("setRotation {}", rotation),
        Commands::GetScreenshot => "getScreenshot".to_string(),
        Commands::MapTouchScreen => "mapTouchScreen".to_string(),
        Commands::MinToMaxResolution => "minToMaxResolution".to_string(),
    };

    // Add --screen if specified
    if let Some(screen) = &cli.screen {
        cmd.push_str(&format!(" --screen {}", screen));
    }

    // Add additional arguments
    if !cli.args.is_empty() {
        cmd.push(' ');
        cmd.push_str(&cli.args.join(" "));
    }

    cmd
}
