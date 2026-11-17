# ============================================================
# FCR-RSA API Test Script (PowerShell)
# ============================================================
# Prasyarat:
#   1. Server sudah jalan: cargo run -- serve
#   2. Punya API key (dari `cargo run -- apikey create --user emen`)
# ============================================================
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
param(
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [Parameter(Mandatory=$true)]
    [string]$ApiKey
)

$ErrorActionPreference = "Stop"
$headers = @{
    "Authorization" = "Bearer $ApiKey"
    "Content-Type"  = "application/json"
}

function Section($title) {
    Write-Host ""
    Write-Host ("=" * 60) -ForegroundColor Cyan
    Write-Host "  $title" -ForegroundColor Cyan
    Write-Host ("=" * 60) -ForegroundColor Cyan
}

function B64Encode($text) {
    [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($text))
}

function B64Decode($b64) {
    [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($b64))
}

# --- 1. Health check (tanpa auth) ---
Section "1. Health Check"
Invoke-RestMethod -Uri "$BaseUrl/health" -Method Get | ConvertTo-Json -Depth 5

# --- 2. Keygen ---
Section "2. Keygen (buat kunci 'kunci-utama')"
$keygenBody = @{
    name = "kunci-utama"
    tier = "b"
    seed = "fcr-rsa-demo-seed"
    n    = 10
    q    = "1000000007"
} | ConvertTo-Json
$keygenResp = Invoke-RestMethod -Uri "$BaseUrl/keygen" -Method Post -Headers $headers -Body $keygenBody
$keygenResp | ConvertTo-Json -Depth 5

# --- 3. Encrypt ---
Section "3. Encrypt 'Halo Dunia'"
$plaintext = "Halo Dunia"
$encBody = @{
    key_name = "kunci-utama"
    text_b64 = B64Encode $plaintext
    tier     = "b"
} | ConvertTo-Json
$encResp = Invoke-RestMethod -Uri "$BaseUrl/encrypt" -Method Post -Headers $headers -Body $encBody
Write-Host "Blocks: $($encResp.blocks)"
Write-Host "Ciphertext (base64, first 80 char): $($encResp.ciphertext_b64.Substring(0, [Math]::Min(80, $encResp.ciphertext_b64.Length)))..."

# --- 4. Decrypt ---
Section "4. Decrypt"
$decBody = @{
    key_name        = "kunci-utama"
    ciphertext_b64  = $encResp.ciphertext_b64
    tier            = "b"
} | ConvertTo-Json
$decResp = Invoke-RestMethod -Uri "$BaseUrl/decrypt" -Method Post -Headers $headers -Body $decBody
$decoded = B64Decode $decResp.text_b64
Write-Host "Decrypted: '$decoded'"

if ($decoded -eq $plaintext) {
    Write-Host "✅ ROUND-TRIP OK: '$plaintext' -> ciphertext -> '$decoded'" -ForegroundColor Green
} else {
    Write-Host "❌ ROUND-TRIP GAGAL" -ForegroundColor Red
    exit 1
}

# --- 5. Test tanpa auth (harus 401) ---
Section "5. Test tanpa Authorization (harus 401)"
try {
    Invoke-RestMethod -Uri "$BaseUrl/encrypt" -Method Post -Body $encBody -ContentType "application/json"
    Write-Host "❌ FAIL: seharusnya 401" -ForegroundColor Red
} catch {
    $status = $_.Exception.Response.StatusCode.value__
    if ($status -eq 401) {
        Write-Host "✅ PASS: 401 Unauthorized" -ForegroundColor Green
    } else {
        Write-Host "❌ FAIL: dapat status $status" -ForegroundColor Red
    }
}

# --- 6. Test API key invalid (harus 401) ---
Section "6. Test API key invalid (harus 401)"
$badHeaders = @{
    "Authorization" = "Bearer invalid-key-12345"
    "Content-Type"  = "application/json"
}
try {
    Invoke-RestMethod -Uri "$BaseUrl/encrypt" -Method Post -Headers $badHeaders -Body $encBody
    Write-Host "❌ FAIL: seharusnya 401" -ForegroundColor Red
} catch {
    $status = $_.Exception.Response.StatusCode.value__
    if ($status -eq 401) {
        Write-Host "✅ PASS: 401 Unauthorized" -ForegroundColor Green
    } else {
        Write-Host "❌ FAIL: dapat status $status" -ForegroundColor Red
    }
}

Section "SELESAI"
Write-Host "Semua test selesai." -ForegroundColor Green