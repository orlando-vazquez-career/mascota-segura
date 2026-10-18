"""Puente con el contrato CajaMascota (Soroban).

En este esqueleto el backend solo LEE la cadena y calcula hashes. Las transacciones
(deposit, seal_record, pay_vet) las firma cada parte desde su wallet: el backend no custodia.
Para la demo se usa stellar-cli; en producción, stellar-sdk (Python) + Soroban RPC.
"""
import hashlib
import json
import subprocess
from pathlib import Path

DEPLOYMENTS = Path(__file__).resolve().parents[2] / "deployments" / "testnet.json"


def deployment() -> dict:
    if not DEPLOYMENTS.exists():
        return {}
    return json.loads(DEPLOYMENTS.read_text(encoding="utf-8"))


def record_hash(pet_id: int, record: dict) -> str:
    """SHA-256 canónico del registro clínico: es lo único que va a la cadena."""
    canonical = json.dumps({"pet_id": pet_id, **record}, sort_keys=True, ensure_ascii=False, separators=(",", ":"))
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def balance() -> int:
    """Saldo de la cajita en stroops, leído del contrato en testnet con stellar-cli."""
    dep = deployment()
    if not dep.get("contract_id"):
        return 0
    try:
        out = subprocess.run(
            ["stellar", "contract", "invoke", "--id", dep["contract_id"], "--network", dep.get("network", "testnet"),
             "--source", "ms-dueno", "--", "balance"],
            capture_output=True, text=True, timeout=60, check=True,
        ).stdout.strip().strip('"')
        return int(out)
    except (OSError, subprocess.SubprocessError, ValueError):
        return 0
