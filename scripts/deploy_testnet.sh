#!/usr/bin/env bash
# Despliega CajaMascota en Stellar testnet y corre el flujo de demo
# (aporte del dueño -> la veterinaria sella la consulta -> la cajita paga a la veterinaria).
# Requisitos: stellar-cli >= 22, Rust con target wasm32v1-none, jq (opcional).
# Uso: scripts/deploy_testnet.sh            | SKIP_BUILD=1 scripts/deploy_testnet.sh
set -euo pipefail
cd "$(dirname "$0")/.."

NETWORK="${NETWORK:-testnet}"
PET_ID="${PET_ID:-1}"
VISIT_ID="${VISIT_ID:-1}"
DEPOSIT="${DEPOSIT:-2000000000}"   # 200 XLM en stroops
PAY="${PAY:-1200000000}"           # 120 XLM
WASM=target/wasm32v1-none/release/caja_mascota.wasm
OUT=deployments/testnet.json

ensure_key() {
  if ! stellar keys address "$1" >/dev/null 2>&1; then
    stellar keys generate "$1" --network "$NETWORK" --fund >/dev/null
  fi
  stellar keys address "$1"
}
tx_hash() { grep -oE '[0-9a-f]{64}' <<<"$1" | tail -n1 || true; }

DUENO=$(ensure_key ms-dueno)
VET=$(ensure_key ms-vet)
TOKEN=$(stellar contract id asset --asset native --network "$NETWORK")
echo "Dueño: $DUENO"; echo "Veterinaria: $VET"; echo "Token XLM: $TOKEN"

[[ "${SKIP_BUILD:-0}" == "1" ]] || stellar contract build
DEPLOY_OUT=$(stellar contract deploy --wasm "$WASM" --source ms-dueno --network "$NETWORK" --alias caja_mascota 2>&1)
CONTRACT_ID=$(grep -oE 'C[A-Z2-7]{55}' <<<"$DEPLOY_OUT" | tail -n1)
echo "Contrato: $CONTRACT_ID"

INIT_OUT=$(stellar contract invoke --id caja_mascota --source ms-dueno --network "$NETWORK" -- init --owner "$DUENO" --token "$TOKEN" 2>&1)
ADDVET_OUT=$(stellar contract invoke --id caja_mascota --source ms-dueno --network "$NETWORK" -- add_vet --vet "$VET" 2>&1)
DEPOSIT_OUT=$(stellar contract invoke --id caja_mascota --source ms-dueno --network "$NETWORK" -- deposit --from "$DUENO" --amount "$DEPOSIT" 2>&1)

REGISTRO="mascota:$PET_ID|consulta:$VISIT_ID|vacuna antirrabica|$(date +%F)"
HASH=$(printf '%s' "$REGISTRO" | sha256sum | cut -d' ' -f1)
SEAL_OUT=$(stellar contract invoke --id caja_mascota --source ms-vet --network "$NETWORK" -- seal_record --vet "$VET" --pet_id "$PET_ID" --visit_id "$VISIT_ID" --hash "$HASH" 2>&1)
PAY_OUT=$(stellar contract invoke --id caja_mascota --source ms-dueno --network "$NETWORK" -- pay_vet --visit_id "$VISIT_ID" --amount "$PAY" 2>&1)
BALANCE=$(stellar contract invoke --id caja_mascota --source ms-dueno --network "$NETWORK" -- balance 2>/dev/null | tr -d '"')

mkdir -p deployments
cat > "$OUT" <<JSON
{
  "network": "$NETWORK",
  "contract_id": "$CONTRACT_ID",
  "owner": "$DUENO",
  "vet": "$VET",
  "token": "$TOKEN",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "tx": { "deploy": "$(tx_hash "$DEPLOY_OUT")", "init": "$(tx_hash "$INIT_OUT")", "add_vet": "$(tx_hash "$ADDVET_OUT")" },
  "visits": [ { "pet_id": $PET_ID, "visit_id": $VISIT_ID, "registro": "$REGISTRO", "hash": "$HASH", "deposit": $DEPOSIT, "paid": $PAY, "balance_after": "$BALANCE",
    "tx": { "deposit": "$(tx_hash "$DEPOSIT_OUT")", "seal_record": "$(tx_hash "$SEAL_OUT")", "pay_vet": "$(tx_hash "$PAY_OUT")" } } ],
  "links": { "contract": "https://stellar.expert/explorer/$NETWORK/contract/$CONTRACT_ID", "owner": "https://stellar.expert/explorer/$NETWORK/account/$DUENO", "vet": "https://stellar.expert/explorer/$NETWORK/account/$VET", "tx": "https://stellar.expert/explorer/$NETWORK/tx/<hash>" }
}
JSON
echo "Saldo de la cajita: $BALANCE stroops"
echo "Contrato: https://stellar.expert/explorer/$NETWORK/contract/$CONTRACT_ID"
echo "Detalle en $OUT"
