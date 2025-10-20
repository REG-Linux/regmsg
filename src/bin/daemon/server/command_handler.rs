use std::fmt;

/// Result type for command execution
pub type CommandResult = Result<String, CommandError>;

/// Error type for command handling
#[derive(Debug)]
pub enum CommandError {
    /// Error when command has invalid arguments
    InvalidArguments(String),
    /// Error when command is unknown
    UnknownCommand(String),
    /// Error when command execution fails
    ExecutionError(Box<dyn std::error::Error>),
    /// Error when command is empty
    EmptyCommand,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            CommandError::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
            CommandError::ExecutionError(err) => write!(f, "Execution error: {}", err),
            CommandError::EmptyCommand => write!(f, "Empty command"),
        }
    }
}

impl std::error::Error for CommandError {}

impl From<std::string::FromUtf8Error> for CommandError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        CommandError::ExecutionError(Box::new(error))
    }
}

impl From<Box<dyn std::error::Error>> for CommandError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        CommandError::ExecutionError(error)
    }
}

/// Function type aliases for screen operations
pub type ListModesFn = fn(Option<&str>) -> Result<String, Box<dyn std::error::Error>>;
/// Function type for listing outputs
pub type ListOutputsFn = fn() -> Result<String, Box<dyn std::error::Error>>;
/// Function type for getting current mode
pub type CurrentModeFn = fn(Option<&str>) -> Result<String, Box<dyn std::error::Error>>;
/// Function type for getting current output
pub type CurrentOutputFn = fn() -> Result<String, Box<dyn std::error::Error>>;
/// Function type for getting current resolution
pub type CurrentResolutionFn = fn(Option<&str>) -> Result<String, Box<dyn std::error::Error>>;
/// Function type for getting current rotation
pub type CurrentRotationFn = fn(Option<&str>) -> Result<String, Box<dyn std::error::Error>>;
/// Function type for getting current refresh rate
pub type CurrentRefreshFn = fn(Option<&str>) -> Result<String, Box<dyn std::error::Error>>;
/// Function type for getting current backend
pub type CurrentBackendFn = fn() -> Result<String, Box<dyn std::error::Error>>;
/// Function type for setting mode
pub type SetModeFn = fn(Option<&str>, &str) -> Result<(), Box<dyn std::error::Error>>;
/// Function type for setting output
pub type SetOutputFn = fn(&str) -> Result<(), Box<dyn std::error::Error>>;
/// Function type for setting rotation
pub type SetRotationFn = fn(Option<&str>, &str) -> Result<(), Box<dyn std::error::Error>>;
/// Function type for taking screenshot
pub type GetScreenshotFn = fn() -> Result<(), Box<dyn std::error::Error>>;
/// Function type for mapping touchscreen
pub type MapTouchScreenFn = fn() -> Result<(), Box<dyn std::error::Error>>;
/// Function type for setting resolution to maximum
pub type MinToMaxResolutionFn = fn(Option<&str>) -> Result<(), Box<dyn std::error::Error>>;

/// Handle a command string and return the result
/// 
/// # Arguments
/// * `cmdline` - The command line string to parse and execute
/// * `list_modes` - Function to list display modes
/// * `list_outputs` - Function to list display outputs
/// * `current_mode` - Function to get current display mode
/// * `current_output` - Function to get current display output
/// * `current_resolution` - Function to get current resolution
/// * `current_rotation` - Function to get current rotation
/// * `current_refresh` - Function to get current refresh rate
/// * `current_backend` - Function to get current backend
/// * `set_mode` - Function to set display mode
/// * `set_output` - Function to set display output
/// * `set_rotation` - Function to set rotation
/// * `get_screenshot` - Function to take a screenshot
/// * `map_touch_screen` - Function to map touchscreen
/// * `min_to_max_resolution` - Function to set resolution to maximum
/// 
/// # Returns
/// * `CommandResult` - The result of executing the command
pub fn handle_command(
    cmdline: &str,
    list_modes: ListModesFn,
    list_outputs: ListOutputsFn,
    current_mode: CurrentModeFn,
    current_output: CurrentOutputFn,
    current_resolution: CurrentResolutionFn,
    current_rotation: CurrentRotationFn,
    current_refresh: CurrentRefreshFn,
    current_backend: CurrentBackendFn,
    set_mode: SetModeFn,
    set_output: SetOutputFn,
    set_rotation: SetRotationFn,
    get_screenshot: GetScreenshotFn,
    map_touch_screen: MapTouchScreenFn,
    min_to_max_resolution: MinToMaxResolutionFn,
) -> CommandResult {
    // Split command and arguments
    let parts: Vec<&str> = cmdline.split_whitespace().collect();
    let (cmd, args) = parts.split_first().unwrap_or((&"", &[]));

    match *cmd {
        // Functions that return Result<String, Box<dyn Error>>
        "listModes" => execute_command_with_result(|| list_modes(None)),
        "listOutputs" => execute_command_with_result(|| list_outputs()),
        "currentMode" => execute_command_with_result(|| current_mode(None)),
        "currentOutput" => execute_command_with_result(|| current_output()),
        "currentResolution" => execute_command_with_result(|| current_resolution(None)),
        "currentRotation" => execute_command_with_result(|| current_rotation(None)),
        "currentRefresh" => execute_command_with_result(|| current_refresh(None)),
        "currentBackend" => execute_command_with_result(|| current_backend()),

        // Functions that return Result<(), Box<dyn Error>>
        "getScreenshot" => {
            execute_command_with_unit_result(|| get_screenshot(), "Screenshot taken")
        }
        "mapTouchScreen" => {
            execute_command_with_unit_result(|| map_touch_screen(), "Touchscreen mapped")
        }
        "minTomaxResolution" => execute_command_with_unit_result(
            || min_to_max_resolution(None),
            "Resolution set to max",
        ),

        // Commands with arguments
        "setMode" if args.len() == 1 => execute_command_with_arg_result(
            |arg| set_mode(None, arg),
            args[0],
            "Mode set to {}",
            "setMode",
        ),
        "setOutput" if args.len() == 1 => execute_command_with_arg_result(
            |arg| set_output(arg),
            args[0],
            "Output set to {}",
            "setOutput",
        ),
        "setRotation" if args.len() == 1 => execute_command_with_arg_result(
            |arg| set_rotation(None, arg),
            args[0],
            "Rotation set to {}",
            "setRotation",
        ),

        // Empty command
        "" => Err(CommandError::EmptyCommand),

        // Unknown command
        _ => Err(CommandError::UnknownCommand(cmdline.to_string())),
    }
}

/// Execute a command that returns Result<String, Box<dyn std::error::Error>>
/// 
/// # Arguments
/// * `f` - The function to execute
/// 
/// # Returns
/// * `CommandResult` - The result of executing the function
fn execute_command_with_result<F>(f: F) -> CommandResult
where
    F: FnOnce() -> Result<String, Box<dyn std::error::Error>>,
{
    match f() {
        Ok(result) => Ok(result),
        Err(err) => Err(CommandError::ExecutionError(err)),
    }
}

/// Execute a command that returns Result<(), Box<dyn std::error::Error>> and returns a success message
/// 
/// # Arguments
/// * `f` - The function to execute
/// * `success_msg` - The success message to return if execution is successful
/// 
/// # Returns
/// * `CommandResult` - The result of executing the function with a success message
fn execute_command_with_unit_result<F>(f: F, success_msg: &str) -> CommandResult
where
    F: FnOnce() -> Result<(), Box<dyn std::error::Error>>,
{
    match f() {
        Ok(_) => Ok(success_msg.to_string()),
        Err(err) => Err(CommandError::ExecutionError(err)),
    }
}

/// Execute a command with an argument that returns Result<(), Box<dyn std::error::Error>> and returns a success message
/// 
/// # Arguments
/// * `f` - The function to execute with an argument
/// * `arg` - The argument to pass to the function
/// * `success_format_template` - The template for the success message (e.g. "Mode set to {}")
/// * `cmd_name` - The name of the command for error reporting
/// 
/// # Returns
/// * `CommandResult` - The result of executing the function with formatted success message
fn execute_command_with_arg_result<F>(
    f: F,
    arg: &str,
    success_format_template: &str, // Template like "Mode set to {}"
    cmd_name: &str,
) -> CommandResult
where
    F: FnOnce(&str) -> Result<(), Box<dyn std::error::Error>>,
{
    if arg.is_empty() {
        return Err(CommandError::InvalidArguments(format!(
            "{} requires an argument",
            cmd_name
        )));
    }

    match f(arg) {
        Ok(_) => {
            // We cannot pass a runtime string into the format! macro's format string,
            // so handle templates at runtime: if the template contains "{}", replace
            // the first occurrence; otherwise append the argument.
            let result = if success_format_template.contains("{}") {
                success_format_template.replacen("{}", arg, 1)
            } else {
                format!("{} {}", success_format_template, arg)
            };
            Ok(result)
        }
        Err(err) => Err(CommandError::ExecutionError(err)),
    }
}
