#!/bin/bash

# Default options
DEFAULT_ID=1
DEFAULT_CONFIG_FILE="configs/basic_config.json"
DEFAULT_LOG=false

# Parse arguments
ID=$DEFAULT_ID
CONFIG_FILE=$DEFAULT_CONFIG_FILE
ENABLE_LOG=$DEFAULT_LOG

# Ensure logs directory exists
mkdir -p logs

cargo build --quiet --release --bin gateway

while [[ $# -gt 0 ]]; do
    case $1 in
        --id)
            ID=$2
            shift 2
            ;;
        --config)
            CONFIG_FILE=$2
            shift 2
            ;;
        --log)
            ENABLE_LOG=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Function to kill all child processes upon receiving a signal
cleanup() {
    echo "Deteniendo todos los procesos."
    pkill -P $$
    exit 0
}

# Trap signals (Ctrl+C or kill)
trap cleanup SIGINT SIGTERM

echo "Using config file: $CONFIG_FILE"
echo "Starting gateway process."

# Run commands in the background
if $ENABLE_LOG; then
    cargo run --quiet --release --bin gateway $ID $CONFIG_FILE > logs/gateway.log 2>&1 &
else
    cargo run --quiet --release --bin gateway $i $CONFIG_FILE &
fi

echo ""

if $ENABLE_LOG; then
    echo "Logs are being saved in the "logs" directory."
fi

# Wait indefinitely (until the script is interrupted)
wait
