#!/bin/bash

# Default options
DEFAULT_MAX_PASSENGERS=1
DEFAULT_CONFIG_FILE="configs/basic_config.json"
DEFAULT_LOG=false

# Parse arguments
MAX_PASSENGERS=$DEFAULT_MAX_PASSENGERS
CONFIG_FILE=$DEFAULT_CONFIG_FILE
ENABLE_LOG=$DEFAULT_LOG

# Ensure logs directory exists
mkdir -p logs

cargo build --quiet --release --bin passenger

while [[ $# -gt 0 ]]; do
    case $1 in
        --passengers)
            MAX_PASSENGERS=$2
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
echo "Starting $MAX_PASSENGERS passenger processes."
echo "Logging enabled: $ENABLE_LOG"

echo ""

# Run commands in the background
for (( i=1; i<=MAX_PASSENGERS; i++ )); do
    sleep 1

    if $ENABLE_LOG; then
        cargo run --quiet --release --bin passenger $i $CONFIG_FILE > logs/passenger_$i.log 2>&1 &
    else
        cargo run --quiet --release --bin passenger $i $CONFIG_FILE &
    fi
done

echo ""

if $ENABLE_LOG; then
    echo "Logs are being saved in the "logs" directory."

    # Read commands from the user
    echo ""

    echo "Enter commands:"
    echo "  - To start a passenger: start <passenger_id>"
    echo "  - To stop a passenger: kill <passenger_id>"

    echo ""

    while read -r line; do
        if [[ $line == "kill "* ]]; then
            passenger_id=$(echo $line | cut -d' ' -f2)
            echo "Killing passenger $passenger_id"
            pkill -f "passenger $passenger_id"
        elif [[ $line == "start "* ]]; then
            passenger_id=$(echo $line | cut -d' ' -f2)
            echo "Starting passenger $passenger_id"
            cargo run --quiet --release --bin passenger $passenger_id $CONFIG_FILE > logs/passenger_$passenger_id.log 2>&1 &
        else
            echo "Unknown command: $line"
        fi

        echo ""
    done
fi


# Wait indefinitely (until the script is interrupted)
wait
