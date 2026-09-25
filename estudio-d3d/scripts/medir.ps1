# FPS sin VSync: cada .exe dibuja los fotogramas 0..N-1 (--medir N), 3 veces.
#
#   powershell -ExecutionPolicy Bypass -File scripts\medir.ps1 [-Fotogramas 20000]
#
# Y el tiempo del rasterizador de referencia por CPU para los fotogramas 0, 30 y 60.

param([int]$Fotogramas = 20000)
$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

cargo build --release --quiet
if ($LASTEXITCODE -ne 0) { throw "cargo build falló" }

foreach ($v in "d3d9", "d3d10", "d3d11", "d3d12") {
    foreach ($i in 1..3) {
        & "target\release\estudio_$v.exe" --medir $Fotogramas | Select-String "FPS" | Write-Output
    }
}
foreach ($n in 0, 30, 60) {
    & "target\release\estudio_cpu.exe" --fotograma $n --salida "target\capturas\medir_cpu.png" | Write-Output
}
