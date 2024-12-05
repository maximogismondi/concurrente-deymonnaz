#!/bin/bash
# Test 2: Elección de Conductor Lider

commands=(
    "./target/release/driver 1 configs/test_config.json"
    "./target/release/driver 2 configs/test_config.json"
    "./target/release/driver 3 configs/test_config.json"
)

echo "Test 2: Ejecución normal, sin caidas de ningún tipo"
echo "Para terminar el test, cierre las ventanas de xterm restantes"


pids=()
for cmd in "${commands[@]}"; do
    xterm -e "bash -c '$cmd; exec bash'" &
    pids+=($!)
    sleep 1
done

sleep 10

#Mato el proceso del driver 1 (Leader)
kill ${pids[0]}

#Termino la ejecución de los drivers restantes a mano
wait ${pids[1]}
wait ${pids[2]}

echo "Test 2: Finalizado"