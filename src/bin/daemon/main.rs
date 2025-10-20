#![cfg(feature = "daemon")]

mod server {
    pub mod command_handler;
    pub mod response_handler;
    pub mod server;
}
mod screen;

use server::server::DaemonServer;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut daemon_server = DaemonServer::new()?;

    daemon_server
        .run(
            screen::list_modes,
            screen::list_outputs,
            screen::current_mode,
            screen::current_output,
            screen::current_resolution,
            screen::current_rotation,
            screen::current_refresh,
            screen::current_backend,
            screen::set_mode,
            screen::set_output,
            screen::set_rotation,
            screen::get_screenshot,
            screen::map_touch_screen,
            screen::min_to_max_resolution,
        )
        .await
}
