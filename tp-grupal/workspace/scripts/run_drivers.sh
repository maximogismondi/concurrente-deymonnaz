#!/bin/bash

# Default options
DEFAULT_MAX_DRIVERS=2
DEFAULT_CONFIG_FILE="configs/basic_config.json"
DEFAULT_LOG=false

# Parse arguments
MAX_DRIVERS=$DEFAULT_MAX_DRIVERS
CONFIG_FILE=$DEFAULT_CONFIG_FILE
ENABLE_LOG=$DEFAULT_LOG

# Ensure logs directory exists
mkdir -p logs

cargo build --quiet --release --bin driver

while [[ $# -gt 0 ]]; do
    case $1 in
        --drivers)
            MAX_DRIVERS=$2
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
echo "Starting $MAX_DRIVERS driver processes"
echo "Logging enabled: $ENABLE_LOG"

echo ""

# Run commands in the background
for (( i=1; i<=MAX_DRIVERS; i++ )); do
    sleep 1

    if $ENABLE_LOG; then
        echo "Starting driver $i"
        cargo run --quiet --release --bin driver $i $CONFIG_FILE > logs/driver_$i.log 2>&1 &
    else
        cargo run --quiet --release --bin driver $i $CONFIG_FILE &
    fi
done

echo ""

if $ENABLE_LOG; then
    echo "Logs are being saved in the "logs" directory."

    # Read commands from the user
    echo ""

    echo "Enter commands to start or stop drivers:"
    echo "  - To start a driver: start <driver_id>"	
    echo "  - To stop a driver: kill <driver_id>"

    echo ""

    while read -r line; do
        if [[ $line == "kill "* ]]; then
            driver_id=$(echo $line | cut -d' ' -f2)
            echo "Killing driver $driver_id"
            pkill -f "driver $driver_id"
        elif [[ $line == "start "* ]]; then
            driver_id=$(echo $line | cut -d' ' -f2)
            echo "Starting driver $driver_id"
            cargo run --quiet --release --bin driver $driver_id $CONFIG_FILE > logs/driver_$driver_id.log 2>&1 &
        else
            echo "Unknown command: $line"
        fi

        echo ""
    done
fi

# Wait indefinitely (until the script is interrupted)
wait