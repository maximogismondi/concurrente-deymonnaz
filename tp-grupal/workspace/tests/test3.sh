#!/bin/bash
# Test 3: Se cae un conductor

commands=(
    "./target/release/driver 1 configs/test_config.json"
    "./target/release/gateway 1 configs/test_config.json"
    "./target/release/passenger 1 configs/test_config.json"
)

echo "Test 3: Se cae el conductor en viaje"


pids=()
for cmd in "${commands[@]}"; do
    xterm -e "bash -c '$cmd; exit'" &
    pids+=($!)
    sleep 1
done

sleep 15

#Mato el proceso del driver 1 (Leader)
kill ${pids[0]}

#Termino la ejecución de los drivers restantes a mano
wait ${pids[2]}

sleep 2

kill ${pids[1]}

echo "Test 3: Finalizado"