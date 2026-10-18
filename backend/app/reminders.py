"""Reglas de recordatorio y la idea diaria. Sin IA: reglas claras que un veterinario puede revisar."""
from datetime import date, timedelta

# Frecuencia de cada tratamiento (días) y costo de referencia en Bs (Cochabamba, 2026).
SCHEDULE = {
    "vacuna_antirrabica": {"every_days": 365, "cost_bs": 70, "label": "vacuna antirrábica"},
    "desparasitacion": {"every_days": 90, "cost_bs": 30, "label": "desparasitación"},
    "limpieza_dental": {"every_days": 365, "cost_bs": 420, "label": "limpieza dental"},
}
BS_PER_XLM = 2.0  # TODO: tipo de cambio real (en producción la cajita guarda USDC: Bs ~11,7 por USDC)
STROOPS = 10_000_000

IDEAS = [
    "Paseo corto antes de las 9 y agua fresca cada 2 horas.",
    "Juego de olfato: tres vasitos y una croqueta escondida. Cinco minutos.",
    "Cepillado de 3 minutos mientras le hablas: menos pelo en casa y más calma.",
    "Enséñale a sentarse antes de comer: 2 minutos, premio pequeño.",
    "Cambia la ruta del paseo: olores nuevos cansan más que correr.",
    "Un hueso de juguete congelado con un poco de su comida: entretenido y fresco.",
    "Diez minutos de pelota en el patio y luego sombra y agua.",
]


def upcoming(records: list[dict]) -> list[dict]:
    """Próximos tratamientos según la última fecha registrada de cada tipo."""
    last: dict[str, date] = {}
    for r in records:
        d = date.fromisoformat(str(r["date"]))
        if r["kind"] not in last or d > last[r["kind"]]:
            last[r["kind"]] = d
    out = []
    for kind, rule in SCHEDULE.items():
        due = last[kind] + timedelta(days=rule["every_days"]) if kind in last else date.today()
        out.append({"kind": kind, "label": rule["label"], "due": due.isoformat(), "cost_bs": rule["cost_bs"]})
    return sorted(out, key=lambda x: x["due"])


def what_it_covers(balance_stroops: int) -> list[dict]:
    """Qué tratamientos cubre el saldo, en orden de vencimiento."""
    bs = balance_stroops / STROOPS * BS_PER_XLM
    covers = []
    for kind, rule in SCHEDULE.items():
        covered = bs >= rule["cost_bs"]
        covers.append({"label": rule["label"], "cost_bs": rule["cost_bs"], "covered": covered,
                       "missing_bs": 0 if covered else round(rule["cost_bs"] - bs)})
        if covered:
            bs -= rule["cost_bs"]
    return covers


def daily_idea(pet_id: int, today: date) -> str:
    return IDEAS[(today.toordinal() + pet_id) % len(IDEAS)]
