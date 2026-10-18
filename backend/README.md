# Backend (esqueleto)

```bash
python -m venv .venv && . .venv/bin/activate     # Windows: .venv\Scripts\activate
pip install -r requirements.txt
uvicorn app.main:app --reload
```

- `GET /pets/1/card`: lo que ve el veterinario al escanear el QR (historial + próximos tratamientos).
- `POST /pets/1/records`: guarda un registro y devuelve el hash que la veterinaria sella con `seal_record`.
- `GET /pets/1/box`: saldo de la cajita (leído del contrato en testnet) y para qué alcanza.
- `POST /webhook/whatsapp`: entrada del asistente (WhatsApp Cloud API).
- `GET /daily-idea/1`: la idea del día.

El backend nunca firma por el dueño ni custodia fondos: prepara transacciones y lee eventos del contrato.
