#!/bin/bash
# Test 1: Ejecución normal, sin caidas de ningún tipo
# El viaje puede ser rechazado por el Gateway o el Driver

commands=(
    "./target/release/driver 1 configs/test_config.json"
    "./target/release/gateway 1 configs/test_config.json"
    "./target/release/passenger 1 configs/test_config.json"
)

echo "Test 1: Ejecución normal, sin caidas de ningún tipo"

pids=()
for cmd in "${commands[@]}"; do
    xterm -e "bash -c '$cmd; exit'" &
    pids+=($!)
    sleep 1
done

# Esperar a que el proceso de passenger termine
wait ${pids[2]}

sleep 3

# Matar los procesos de driver y gateway
kill ${pids[0]}
kill ${pids[1]}

echo "Test 1: Finalizado"