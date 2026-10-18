<#
.SYNOPSIS
  Despliega CajaMascota en Stellar testnet y corre el flujo de demo:
  aporte del dueno -> la veterinaria sella la consulta -> la cajita paga a la veterinaria.

.DESCRIPTION
  Requisitos: stellar-cli (>= 22), Rust con el target wasm32v1-none y acceso a internet.
  Crea dos identidades locales (ms-dueno, ms-vet) fondeadas con Friendbot si no existen,
  compila el contrato, lo despliega con el alias "caja_mascota", lo inicializa y ejecuta
  el flujo. Deja ids, hashes y links en deployments/testnet.json.

.EXAMPLE
  .\scripts\deploy_testnet.ps1
  .\scripts\deploy_testnet.ps1 -SkipBuild                  # reutiliza el .wasm ya compilado
  .\scripts\deploy_testnet.ps1 -OnlyDemo -VisitId 2        # otra consulta sobre el contrato ya desplegado
#>
param(
    [switch]$SkipBuild,
    [switch]$OnlyDemo,
    [string]$Network = "testnet",
    [int]$PetId = 1,
    [int]$VisitId = 1,
    [long]$DepositStroops = 2000000000,   # 200 XLM (1 XLM = 10^7 stroops)
    [long]$PayStroops = 1200000000        # 120 XLM
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root
$Wasm = "target/wasm32v1-none/release/caja_mascota.wasm"
$Deployments = Join-Path $Root "deployments/testnet.json"

function Run([string]$cmd) {
    # cmd /c pasa los argumentos tal cual, incluido el separador "--" que usa stellar-cli.
    Write-Host ">> $cmd" -ForegroundColor DarkGray
    $out = cmd /c "$cmd 2>&1" | Out-String
    if ($LASTEXITCODE -ne 0) { throw "Fallo: $cmd`n$out" }
    return $out
}

function RunQuiet([string]$cmd) {
    # Igual que Run pero sin stderr: para lecturas (balance) stellar-cli avisa por stderr que no envia la transaccion.
    Write-Host ">> $cmd" -ForegroundColor DarkGray
    $out = cmd /c "$cmd 2>nul" | Out-String
    if ($LASTEXITCODE -ne 0) { throw "Fallo: $cmd" }
    return $out
}

function Tx-Hash([string]$output) {
    # stellar-cli imprime "Signing transaction: <hash>"; lo rescatamos del output combinado.
    $m = [regex]::Match($output, 'Signing transaction:\s*([0-9a-f]{64})')
    if ($m.Success) { return $m.Groups[1].Value }
    $all = [regex]::Matches($output, '[0-9a-f]{64}')
    if ($all.Count -gt 0) { return $all[$all.Count - 1].Value }
    return ""
}

function Ensure-Key([string]$name) {
    $addr = (cmd /c "stellar keys address $name 2>nul" | Out-String).Trim()
    if (-not $addr) {
        Write-Host "Creando y fondeando la identidad $name en $Network..."
        Run "stellar keys generate $name --network $Network --fund" | Out-Null
        $addr = (cmd /c "stellar keys address $name" | Out-String).Trim()
    }
    return $addr
}

$Dueno = Ensure-Key "ms-dueno"
$Vet = Ensure-Key "ms-vet"
$Token = (Run "stellar contract id asset --asset native --network $Network").Trim()
Write-Host "Dueno:       $Dueno"
Write-Host "Veterinaria: $Vet"
Write-Host "Token (XLM): $Token"

$prev = $null
if (Test-Path $Deployments) { $prev = Get-Content $Deployments -Raw | ConvertFrom-Json }

if (-not $OnlyDemo) {
    if (-not $SkipBuild) { Run "stellar contract build" | Out-Null }
    if (-not (Test-Path $Wasm)) { throw "No encuentro $Wasm. Compila con: stellar contract build" }

    $deployOut = Run "stellar contract deploy --wasm $Wasm --source ms-dueno --network $Network --alias caja_mascota"
    $ContractId = ([regex]::Match($deployOut, 'C[A-Z2-7]{55}')).Value
    if (-not $ContractId) { throw "No pude leer el id del contrato:`n$deployOut" }
    Write-Host "Contrato desplegado: $ContractId" -ForegroundColor Green

    $initOut = Run "stellar contract invoke --id caja_mascota --source ms-dueno --network $Network -- init --owner $Dueno --token $Token"
    $addVetOut = Run "stellar contract invoke --id caja_mascota --source ms-dueno --network $Network -- add_vet --vet $Vet"

    $state = @{
        network     = $Network
        contract_id = $ContractId
        owner       = $Dueno
        vet         = $Vet
        token       = $Token
        deployed_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        tx          = @{ deploy = (Tx-Hash $deployOut); init = (Tx-Hash $initOut); add_vet = (Tx-Hash $addVetOut) }
        visits      = @()
    }
} else {
    if (-not $prev -or -not $prev.contract_id) { throw "No hay contrato en $Deployments; corre el script sin -OnlyDemo." }
    $state = @{
        network     = $prev.network
        contract_id = $prev.contract_id
        owner       = $prev.owner
        vet         = $prev.vet
        token       = $prev.token
        deployed_at = $prev.deployed_at
        tx          = @{ deploy = $prev.tx.deploy; init = $prev.tx.init; add_vet = $prev.tx.add_vet }
        visits      = @($prev.visits)
    }
}

# ---- Flujo de demo: aporte -> sello -> pago ----
$depositOut = Run "stellar contract invoke --id caja_mascota --source ms-dueno --network $Network -- deposit --from $Dueno --amount $DepositStroops"

$registro = "mascota:$PetId|consulta:$VisitId|vacuna antirrabica|$(Get-Date -Format yyyy-MM-dd)"
$sha = [System.Security.Cryptography.SHA256]::Create()
$hash = ([System.BitConverter]::ToString($sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($registro)))).Replace("-", "").ToLower()
$sealOut = Run "stellar contract invoke --id caja_mascota --source ms-vet --network $Network -- seal_record --vet $Vet --pet_id $PetId --visit_id $VisitId --hash $hash"

$payOut = Run "stellar contract invoke --id caja_mascota --source ms-dueno --network $Network -- pay_vet --visit_id $VisitId --amount $PayStroops"
$balance = (RunQuiet "stellar contract invoke --id caja_mascota --source ms-dueno --network $Network -- balance").Trim().Trim('"')

$visit = @{
    pet_id        = $PetId
    visit_id      = $VisitId
    registro      = $registro
    hash          = $hash
    deposit       = $DepositStroops
    paid          = $PayStroops
    balance_after = $balance
    tx            = @{ deposit = (Tx-Hash $depositOut); seal_record = (Tx-Hash $sealOut); pay_vet = (Tx-Hash $payOut) }
}
$state.visits = @($state.visits) + @($visit)
$cid = $state.contract_id
$state.links = @{
    contract = "https://stellar.expert/explorer/$Network/contract/$cid"
    owner    = "https://stellar.expert/explorer/$Network/account/$($state.owner)"
    vet      = "https://stellar.expert/explorer/$Network/account/$($state.vet)"
    tx       = "https://stellar.expert/explorer/$Network/tx/<hash>"
}

New-Item -ItemType Directory -Force -Path (Split-Path $Deployments) | Out-Null
($state | ConvertTo-Json -Depth 6) | Set-Content -Path $Deployments -Encoding UTF8

Write-Host ""
Write-Host "Listo. Saldo de la cajita: $balance stroops" -ForegroundColor Green
Write-Host "Contrato: $($state.links.contract)"
Write-Host "Pago:     https://stellar.expert/explorer/$Network/tx/$($visit.tx.pay_vet)"
Write-Host "Detalle en deployments/testnet.json"
