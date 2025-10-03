# Demo

This demo showcases the durable runner (druns) by starting multiple Docker services and monitoring their logs in a styled TUI.

## Requirements

- Docker and Docker Compose
- VHS (for recording the demo)
- agg (log aggregator, if used)
- druns (the durable runner binary)

## Running the Demo

1. Build druns: `cargo build --release`
2. Install VHS and agg if not available
3. Run the VHS tape: `vhs demo/demo.tape`

This will generate a looping GIF showing the demo.