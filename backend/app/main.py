"""Mascota Segura · backend (esqueleto).

Tres responsabilidades, cada una en su módulo:
- carnet:     historial clínico cifrado fuera de la cadena + hash sellado en CajaMascota (Soroban).
- cajita:     lectura de saldo y preparación de pagos (el dueño firma; el backend nunca custodia).
- asistente:  webhook de WhatsApp Cloud API, recordatorios y la idea diaria.

Correr: uvicorn app.main:app --reload
"""
from datetime import date

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

from . import reminders, stellar

app = FastAPI(title="Mascota Segura", version="0.1.0")

# Datos de ejemplo en memoria. En producción: Postgres con el historial cifrado por mascota.
PETS = {
    1: {
        "name": "Firulais",
        "owner_phone": "+591XXXXXXXX",
        "records": [
            {"visit_id": 1, "kind": "vacuna_antirrabica", "date": "2026-08-12", "vet": "Veterinaria Norte"},
            {"visit_id": 2, "kind": "desparasitacion", "date": "2026-08-12", "vet": "Veterinaria Norte"},
        ],
    }
}


class Record(BaseModel):
    visit_id: int
    kind: str
    date: date
    vet: str
    notes: str = ""


@app.get("/pets/{pet_id}/card")
def pet_card(pet_id: int):
    """Lo que ve el veterinario al escanear el QR (tras la autorización del dueño)."""
    pet = PETS.get(pet_id)
    if not pet:
        raise HTTPException(404, "mascota no encontrada")
    return {"pet": pet["name"], "records": pet["records"], "next": reminders.upcoming(pet["records"])}


@app.post("/pets/{pet_id}/records")
def add_record(pet_id: int, record: Record):
    """Guarda el registro fuera de la cadena y devuelve el hash que la veterinaria sella en CajaMascota."""
    pet = PETS.get(pet_id)
    if not pet:
        raise HTTPException(404, "mascota no encontrada")
    pet["records"].append(record.model_dump(mode="json"))
    return {"hash": stellar.record_hash(pet_id, record.model_dump(mode="json"))}


@app.get("/pets/{pet_id}/box")
def box(pet_id: int):
    """Saldo de la cajita y para qué alcanza (lo que el asistente convierte a lenguaje humano)."""
    balance = stellar.balance()
    return {"balance_stroops": balance, "covers": reminders.what_it_covers(balance)}


@app.post("/webhook/whatsapp")
def whatsapp_webhook(payload: dict):
    """Entrada de mensajes de WhatsApp Cloud API. TODO: verificar firma, enrutar intents."""
    text = (payload.get("text") or "").strip().lower()
    if text in {"si", "sí", "dale"}:
        return {"reply": "Listo ✅ Mantengo el aporte mensual de Bs 20."}
    return {"reply": "Hola 👋 Escribe 'saldo', 'carnet' o 'turno'."}


@app.get("/daily-idea/{pet_id}")
def daily_idea(pet_id: int):
    return {"idea": reminders.daily_idea(pet_id, date.today())}
