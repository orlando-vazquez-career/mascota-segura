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

### Ya corre en testnet (17 de octubre de 2026)

Contrato `CajaMascota`: [`CAKSO4XWX4LNQ5JUXHEKXEGEEGHRS4PUR3TGQGOMIMOLPS5XWIAXX23Y`](https://stellar.expert/explorer/testnet/contract/CAKSO4XWX4LNQ5JUXHEKXEGEEGHRS4PUR3TGQGOMIMOLPS5XWIAXX23Y)

| Paso | Quién firma | Transacción |
|---|---|---|
| Despliegue | dueño | [a1b81eb2…](https://stellar.expert/explorer/testnet/tx/a1b81eb2bd673fd718a5d950dde86027d89ea27f64d03455602dc7fe4a8e2f12) |
| `init` (dueño + token XLM) | dueño | [b0287e80…](https://stellar.expert/explorer/testnet/tx/b0287e800d6f181c96f77355377b297a2569edc15778f025e526cf03614bf3c0) |
| `add_vet` (verifica la veterinaria) | dueño | [06c359f4…](https://stellar.expert/explorer/testnet/tx/06c359f425380163eb2fa6daf22bfceb0cfa20d0312bdd7062fd07eeeebb07a5) |
| `deposit` 200 XLM a la cajita | dueño | [40d6c57a…](https://stellar.expert/explorer/testnet/tx/40d6c57aea1e938d8051b892890a2b540a036ca8a2f4fcdaf91d693bb431e189) |
| `seal_record` consulta 1, mascota 1 | veterinaria | [d6ddcede…](https://stellar.expert/explorer/testnet/tx/d6ddcede5c7b0058810c7a67720f8ca3f2b032872cb5e3c4f71969a41d6ea7c3) |
| `pay_vet` 120 XLM por la consulta 1 | dueño | [b1ba7298…](https://stellar.expert/explorer/testnet/tx/b1ba7298f222597f3261dd5707ebf7fb376b2d094f82f0ac187fdbc0a1703a34) |

Saldo final de la cajita: 80 XLM. Cuentas: [dueño](https://stellar.expert/explorer/testnet/account/GD73EA76JFDNMMXHT2GQWEBQDMD3FCX7JXNQRN7CWHT3VTDNKKKBXAKS) · [veterinaria](https://stellar.expert/explorer/testnet/account/GAQTLCPPQ5GO3EDLLI3FVSCLAHHMUVFHAQUWCMY447Q5MQRMQBY33K2U).

## Material de la ideatón

En [`docs/entrega/`](docs/entrega/): estudio de validación de mercado (PDF, 112 fuentes), logo y, el domingo, el deck de 8 slides y la evidencia de las entrevistas. El video de 3 minutos va en el proyecto de Stellar Apex.

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
