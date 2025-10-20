use std::fs;
use zeromq::prelude::*;
use zeromq::{RepSocket, ZmqMessage};
use super::command_handler;
use super::response_handler;

/// Path for the daemon socket
const SOCKET_PATH: &str = "/var/run/regmsgd.sock";

/// Main daemon server structure that handles ZeroMQ communication
pub struct DaemonServer {
    /// ZeroMQ reply socket for communication with clients
    socket: RepSocket,
}

impl DaemonServer {
    /// Create a new daemon server instance
    /// 
    /// # Returns
    /// * `Result<DaemonServer, Box<dyn std::error::Error>>` - A new daemon server instance or an error
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Remove existing socket if present
        let _ = fs::remove_file(SOCKET_PATH);

        let mut socket = RepSocket::new();
        // Use blocking operation for bind to ensure it completes
        async_std::task::block_on(async {
            socket.bind(&format!("ipc://{}", SOCKET_PATH)).await
        })?;

        println!("Daemon running on ipc://{} ...", SOCKET_PATH);

        Ok(DaemonServer { socket })
    }

    /// Run the daemon server loop, processing incoming commands
    /// 
    /// # Arguments
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
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if the server runs successfully, or an error
    pub async fn run(
        &mut self,
        list_modes: command_handler::ListModesFn,
        list_outputs: command_handler::ListOutputsFn,
        current_mode: command_handler::CurrentModeFn,
        current_output: command_handler::CurrentOutputFn,
        current_resolution: command_handler::CurrentResolutionFn,
        current_rotation: command_handler::CurrentRotationFn,
        current_refresh: command_handler::CurrentRefreshFn,
        current_backend: command_handler::CurrentBackendFn,
        set_mode: command_handler::SetModeFn,
        set_output: command_handler::SetOutputFn,
        set_rotation: command_handler::SetRotationFn,
        get_screenshot: command_handler::GetScreenshotFn,
        map_touch_screen: command_handler::MapTouchScreenFn,
        min_to_max_resolution: command_handler::MinToMaxResolutionFn,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let msg = self.socket.recv().await;

            match msg {
                Ok(cmdline) => {
                    // Convert first frame to String
                    let cmdline_str = match cmdline.get(0) {
                        Some(frame) => match String::from_utf8(frame.to_vec()) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("Invalid UTF-8 message: {:?}", e);
                                continue;
                            }
                        },
                        None => {
                            eprintln!("Received empty message");
                            continue;
                        }
                    };

                    // Handle the command and get result
                    let result = command_handler::handle_command(
                        &cmdline_str,
                        list_modes,
                        list_outputs,
                        current_mode,
                        current_output,
                        current_resolution,
                        current_rotation,
                        current_refresh,
                        current_backend,
                        set_mode,
                        set_output,
                        set_rotation,
                        get_screenshot,
                        map_touch_screen,
                        min_to_max_resolution,
                    );

                    // Format the response
                    let reply = response_handler::format_response(result);

                    // Send the response to the client
                    if let Err(e) = self.socket.send(ZmqMessage::from(reply)).await {
                        eprintln!("Failed to send reply: {:?}", e);
                    }
                }
                Err(e) => eprintln!("Error receiving message: {:?}", e),
            }
        }
    }
}