# Arquitectura

```
 Dueño (WhatsApp)                 Veterinaria (web/QR)               Familia en el exterior
      |                                   |                                   |
      | aporta por QR (rampa ASFI → USDC)  | escanea QR, pide autorización     | aporta USDC directo
      v                                   v                                   v
 +-----------------------------------------------------------------------------------+
 |  Backend Mascota Segura (FastAPI)                                                  |
 |  - carnet: historial cifrado por mascota (Postgres), hash canónico por registro    |
 |  - asistente: WhatsApp Cloud API, recordatorios por reglas, idea diaria            |
 |  - cajita: lee saldo y eventos del contrato; nunca custodia ni firma por el dueño  |
 +-----------------------------------------------------------------------------------+
      |  prepara transacciones; cada parte firma desde su wallet
      v
 +-----------------------------------------------------------------------------------+
 |  Stellar / Soroban: contrato CajaMascota (uno por mascota o por familia)            |
 |  deposit → seal_record (vet) → pay_vet (owner)      token: USDC (mainnet)           |
 +-----------------------------------------------------------------------------------+
```

## Decisiones

- **Qué va a la cadena**: solo el hash SHA-256 del registro clínico y el flujo de dinero. El historial vive cifrado fuera de la cadena y lo abre el dueño (DS 1793, art. 56). Un hash no revela datos y prueba que el registro existe y no fue alterado.
- **Quién firma qué**: el dueño firma cada pago (no custodial); la veterinaria firma el sello de la consulta. El backend solo prepara transacciones y lee eventos. Si el backend desaparece, la plata sigue siendo del dueño.
- **Por qué un contrato y no una cuenta con multifirma**: la regla "solo veterinarias verificadas y solo con consulta sellada" no se expresa con firmantes; sí con un contrato. La versión simple (cuenta + memo) sirve de demo, el contrato es el producto.
- **Token**: USDC de Circle en mainnet; XLM nativo en testnet para la demo (Friendbot no entrega USDC). Entrada y salida por rampas con certificado ASFI (Banexcoin, Taspri, Vita Wallet); la veterinaria cobra en USDC y los pasa a bolivianos por la misma vía.
- **Aportes de terceros**: `deposit` acepta cualquier `from`, así la abuela en España aporta directo sin remesadora.
- **Asistente sin IA generativa en la ruta crítica**: recordatorios y "para qué alcanza" son reglas revisables por un veterinario; la IA entra solo en el tono de los mensajes.

## Pendientes de producto

- Cajita multi-mascota y límite de pago por consulta (hoy el dueño decide el monto al pagar).
- Verificación de veterinarias contra el registro profesional (Ley 1763) antes de `add_vet`.
- Integración del QR con autorización por WhatsApp (token de un solo uso con vencimiento).
- Exportar el carnet al Registro Único de Mascotas (RUM) del municipio.
