# Server Module

This directory contains the core server components for the regmsg daemon, which manages display settings for REG Linux systems. The server uses a modular architecture to handle client requests and communicate with display backends.

## Architecture

The server module is organized into three main components:

### 1. Command Handler (`command_handler.rs`)

The command handler processes incoming commands from clients and routes them to the appropriate display management functions. It provides:

- **Command Parsing**: Splits command strings into command and arguments
- **Command Dispatch**: Routes commands to the appropriate backend functions
- **Error Handling**: Comprehensive error handling with different error types
- **Type Safety**: Function type aliases for all supported operations

#### Supported Commands

- `listModes`: List available display modes
- `listOutputs`: List available display outputs
- `currentMode`: Get current display mode
- `currentOutput`: Get current display output
- `currentResolution`: Get current resolution
- `currentRotation`: Get current rotation
- `currentRefresh`: Get current refresh rate
- `currentBackend`: Get current display backend
- `setMode <mode>`: Set display mode (e.g., "1920x1080@60")
- `setOutput <output>`: Set display output
- `setRotation <rotation>`: Set display rotation (0, 90, 180, 270)
- `getScreenshot`: Take a screenshot
- `mapTouchScreen`: Map touchscreen to display
- `minTomaxResolution`: Set resolution to maximum

### 2. Response Handler (`response_handler.rs`)

The response handler formats command results into string responses for clients:

- **Result Formatting**: Converts command results to string responses
- **Error Formatting**: Properly formats error messages for client consumption
- **Consistent Output**: Ensures all responses follow the same format

### 3. Server (`server.rs`)

The main server implementation handles ZeroMQ communication:

- **ZeroMQ Integration**: Uses REP socket for request-reply pattern
- **Message Loop**: Processes incoming messages in an infinite loop
- **Socket Management**: Handles socket creation, binding, and cleanup
- **Client Communication**: Receives commands and sends formatted responses

## Communication Protocol

The server uses ZeroMQ IPC communication with the following characteristics:

- **Transport**: IPC (`ipc:///var/run/regmsgd.sock`)
- **Pattern**: Request-Reply (REQ-REP)
- **Message Format**: UTF-8 encoded strings
- **Socket Path**: `/var/run/regmsgd.sock`

## Usage

The server module is designed to work with the main daemon entry point and integrates with the screen management modules to provide display configuration capabilities. It supports both Wayland and KMS/DRM backends through the screen module abstraction.

## Error Handling

The module implements comprehensive error handling through the `CommandError` enum with the following error types:

- `InvalidArguments`: When commands receive invalid or missing arguments
- `UnknownCommand`: When an unrecognized command is received
- `ExecutionError`: When command execution fails
- `EmptyCommand`: When an empty command string is received

## Dependencies

- `zeromq`: For ZeroMQ communication
- `async-std`: For async runtime support
- Screen management modules: For actual display operations