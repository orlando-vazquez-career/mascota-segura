# Mascota Segura

**El carnet de salud y la cajita de ahorro en dólares digitales de tu mascota, con un asistente en WhatsApp que te avisa qué le toca y cómo hacerla feliz hoy.**

Proyecto presentado en la **Ideatón Stellar Bolivia 2026** (17 y 18 de octubre), track *Finanzas para todos*. Cochabamba, Bolivia.

## El problema

En Cochabamba viven 350.000 perros y gatos y 8 de cada 10 tienen dueño. Las familias los tratan como a un hijo, pero su historial de salud vive en un cuaderno o en fotos de WhatsApp y se pierde al cambiar de veterinaria; el 45 % va al veterinario solo cuando ya es una emergencia, y esa emergencia (una fractura cuesta Bs 4.000) se paga con deuda. Ahorrar en bolivianos para eso perdió 40 % de valor en 2026.

## La solución: tres piezas en un celular

| Pieza | Qué hace | Dónde vive |
|---|---|---|
| **Carnet de salud con QR** | El veterinario escanea, el dueño autoriza y ve el historial completo. Cada consulta queda sellada en Stellar (solo el hash; el registro va cifrado fuera de la cadena). | `backend/` + contrato |
| **Cajita de ahorro en dólares digitales** | Aportes desde Bs 20 por QR, guardados en USDC. Solo paga a veterinarias verificadas y solo con una consulta sellada. Cualquier familiar aporta, también desde el exterior. | `contracts/caja_mascota` (Soroban) |
| **Asistente en WhatsApp** | Dice para qué alcanza lo ahorrado, recuerda vacunas y desparasitaciones, avisa cuando la cajita se queda corta y manda una idea diaria para hacer feliz a la mascota. | `backend/app/reminders.py` |

Una maqueta de las tres pantallas está en [`mockup/index.html`](mockup/index.html) (abrir en el navegador, no necesita servidor).

## Por qué Stellar

Sin Stellar esto sería una alcancía más y un cuaderno más. Hoy una familia boliviana no puede abrir una cuenta en dólares para su mascota; con Stellar la cajita guarda USDC desde Bs 20 con una comisión de red de 0,00001 XLM, y el contrato en Soroban hace cumplir las reglas sin confiar en nadie: **solo el dueño firma pagos, solo a veterinarias verificadas, solo por consultas selladas, una vez por consulta.** Los dólares digitales entran y salen por rampas con certificado ASFI (Banexcoin, Taspri, Vita Wallet); el proyecto no custodia fondos ni toca bolivianos.

## El contrato `CajaMascota`

```
init(owner, token)                      crea la cajita (dueño + token: USDC en mainnet, XLM en testnet)
add_vet(vet) / remove_vet(vet)          el dueño verifica o quita una veterinaria
deposit(from, amount) -> balance        cualquiera aporta a la cajita
seal_record(vet, pet_id, visit_id, hash) la veterinaria sella una consulta (hash SHA-256 del registro)
pay_vet(visit_id, amount) -> balance    el dueño paga la consulta sellada; el destino sale del sello
balance() / owner() / is_vet(vet) / visit(visit_id) / paid(visit_id)
```

Errores: `AlreadyInitialized`, `NotInitialized`, `InvalidAmount`, `VetNotVerified`, `VisitAlreadySealed`, `VisitNotFound`, `VisitAlreadyPaid`. Eventos: `VetAdded`, `VetRemoved`, `Deposited`, `Sealed`, `Paid` (el asistente los escucha para avisar al dueño).

### Probar y compilar

```bash
cargo test -p caja-mascota          # 7 pruebas: flujo completo y cada regla que el contrato rechaza
stellar contract build              # genera target/wasm32v1-none/release/caja_mascota.wasm
```

Requisitos: Rust estable con el target `wasm32v1-none` (`rustup target add wasm32v1-none`) y [stellar-cli](https://developers.stellar.org/docs/tools/cli) 25.2 o superior.

### Desplegar en testnet y correr la demo

```powershell
.\scripts\deploy_testnet.ps1          # Windows
```
```bash
scripts/deploy_testnet.sh             # Linux / macOS
```

El script crea dos identidades locales fondeadas con Friendbot (`ms-dueno`, `ms-vet`), compila, despliega con el alias `caja_mascota`, inicializa el contrato y ejecuta el flujo real:

1. el dueño aporta 200 XLM a la cajita (`deposit`),
2. la veterinaria sella la consulta 1 de la mascota 1 con el hash de su registro (`seal_record`),
3. el dueño paga 120 XLM por esa consulta y el contrato los envía a la veterinaria (`pay_vet`),
4. el saldo queda en 80 XLM (`balance`).

Los ids, hashes de transacción y links a Stellar Expert quedan en [`deployments/testnet.json`](deployments/testnet.json).

## Estructura del repositorio

```
contracts/caja_mascota/   contrato Soroban (Rust) y sus pruebas
scripts/                  despliegue y demo en testnet (PowerShell y bash)
deployments/              ids y transacciones de cada red
backend/                  esqueleto FastAPI: carnet, cajita, webhook de WhatsApp, recordatorios
mockup/                   maqueta estática de las tres pantallas
docs/                     arquitectura y decisiones
```

## Modelo y cumplimiento

Plan Mascota Bs 15/mes por mascota (carnet + asistente; la cajita es gratis), 2,5 % sobre cada pago de la cajita a una veterinaria y Bs 100/mes por veterinaria. Es ahorro propio del dueño con reglas de uso: no hay prima ni pool (no es seguro ni prepago, Ley 1883); no custodial y sin bolivianos (Ley 393, DS 5384), con rampa en PSAV certificado por ASFI; datos clínicos cifrados fuera de la cadena, a Stellar solo va el sello (DS 1793).

## Próximos pasos

Piloto con 2 veterinarias y 50 mascotas en Cochabamba; conversación con la Clínica Veterinaria Municipal y el SUMA; postulación al Builder Accelerator de BAF. En el contrato: cajita multi-mascota, pagos en USDC con límite por consulta y firma de la veterinaria al cobrar.

## Autor

Orlando Vázquez (DevZen), Cochabamba. La idea original y la mirada de dueña de mascota son de mi pareja. Hecho durante el fin de semana de la ideatón; usé IA como apoyo para la investigación de mercado y el andamiaje del código, y las decisiones de diseño, las pruebas y el despliegue son propios.

Licencia MIT.
