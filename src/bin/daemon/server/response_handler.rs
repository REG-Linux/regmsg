use super::command_handler::{CommandResult, CommandError};

/// Format a command result into a string response
/// 
/// # Arguments
/// * `result` - The command result to format
/// 
/// # Returns
/// * `String` - The formatted response string
pub fn format_response(result: CommandResult) -> String {
    match result {
        Ok(msg) => msg,
        Err(CommandError::ExecutionError(err)) => format!("Error: {}", err),
        Err(err) => format!("Error: {}", err),
    }
}