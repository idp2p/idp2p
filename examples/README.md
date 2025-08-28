# Multi-Node Communication CLI

A terminal-based communication application built with Rust and Ratatui that allows multiple nodes (Alice and Bob) to resolve each other's addresses and exchange messages.

## Features

- **Multi-Node Architecture**: Supports Alice and Bob nodes
- **Address Resolution**: Nodes can resolve each other's network addresses
- **Real-time Messaging**: Send and receive messages between nodes
- **Interactive TUI**: Clean terminal user interface
- **Command-based Input**: Simple command system for operations

For simplicity fake the network features 

## Usage

### Starting a Node

Start Alice:
```bash
cargo run -- --node alice
```

Start Bob:
```bash
cargo run -- --node bob
```


### Interface Overview

The application interface consists of four sections:

1. **Main Content**: Event and message history
2. **Input**: Command input with `>` prompt
3. **Footer**: Shows current node and mode

### Commands

#### Initial Mode

Multiple steps to init the messaging channel. Each step shows a confirmation for the user to confirm

  - Connect   : `Connect to Bob as <addr>`(when running as Alice), `Connect to Alice as <addr>` (when running as Bob)
  - Connected : `Connected to Bob`
  - Resolve   : `Resolve Bob with <id>`(when running as Alice and connected to Bob)  
  - Resolved  : `Resolved Bob with json body`

#### Messaging Mode (After Resolution)

- Type any message and press Enter to send
- Messages appear in the message history with timestamps

### Keyboard Shortcuts

- **Enter**: Execute command or send message
- **Ctrl+Q**: Quit application

### Project Structure

```
src/
├── main.rs      # Entry point and CLI argument parsing
├── app.rs       # Application state and event handling
└── ui.rs        # Terminal user interface rendering
```

### Dependencies

- **ratatui**: Terminal user interface framework
- **crossterm**: Cross-platform terminal manipulation
- **clap**: Command-line argument parsing
- **tokio**: Async runtime
- **serde**: Serialization framework
- **anyhow**: Error handling
