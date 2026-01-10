#!/bin/bash

# Always kill any previous instance
pm2 delete db-writter >/dev/null 2>&1

# Build the binary
cargo build --release

# Start cleanly
pm2 start ./target/release/db_writter \
  --name db-writter \
  --cwd "$(pwd)" \
  --restart-delay=3000 \
  --watch=false

