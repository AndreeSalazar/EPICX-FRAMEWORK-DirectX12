# Regenera las imágenes de referencia y compara GPU contra CPU.
#
#   powershell -ExecutionPolicy Bypass -File scripts\referencia.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\referencia.ps1 -Barrido 1   (además: DX12 vs CPU
#                                                  en los fotogramas 0..359 de 1 en 1)
#
# - referencia\dx12\cubo_fNNNN.png  (Direct3D 12, leído del back buffer)
# - referencia\cpu\cubo_fNNNN.png   (cubo_neutro::dibujar_por_cpu)
# - target\capturas\d3d9|d3d10|d3d11 (las otras versiones, no se guardan en git)
# - target\capturas\d3d9_corregido   (D3D9 con --corregir-medio-pixel)
# Sin antialiasing: SampleDesc.Count = 1 en todas las versiones y MultisampleEnable = FALSE.

param([int]$Barrido = 0)
$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

cargo build --release --quiet
if ($LASTEXITCODE -ne 0) { throw "cargo build falló" }

$bin = "target\release"
$fotogramas = 0, 30, 60

foreach ($n in $fotogramas) {
    $f = "{0:D4}" -f $n
    & "$bin\estudio_d3d12.exe" --fotograma $n --salida "referencia\dx12\cubo_f$f.png" | Out-Null
    & "$bin\estudio_cpu.exe" --fotograma $n --salida "referencia\cpu\cubo_f$f.png" | Out-Null
    foreach ($v in "d3d9", "d3d10", "d3d11") {
        & "$bin\estudio_$v.exe" --fotograma $n --salida "target\capturas\$v\cubo_f$f.png" | Out-Null
    }
    & "$bin\estudio_d3d9.exe" --fotograma $n --corregir-medio-pixel --salida "target\capturas\d3d9_corregido\cubo_f$f.png" | Out-Null
}

foreach ($n in $fotogramas) {
    $f = "{0:D4}" -f $n
    $cpu = "referencia\cpu\cubo_f$f.png"
    & "$bin\comparar.exe" "referencia\dx12\cubo_f$f.png" $cpu --listar 10
    foreach ($v in "d3d9", "d3d9_corregido", "d3d10", "d3d11") {
        & "$bin\comparar.exe" "target\capturas\$v\cubo_f$f.png" $cpu --mapa "target\capturas\$v\diferencias_f$f.png"
    }
}

if ($Barrido -gt 0) {
    # Dos jueces: el exacto (a_unorm8) y el que imita la conversión a 8 bits de la GPU (--unorm-12bits)
    $resumen = @{}
    foreach ($juez in "exacto", "12bits") {
        $resumen[$juez] = @{ fotogramas = 0; con_dif = 0; distintos = 0; de_1 = 0 }
    }
    for ($n = 0; $n -lt 360; $n += $Barrido) {
        $f = "{0:D4}" -f $n
        & "$bin\estudio_d3d12.exe" --fotograma $n --salida "target\capturas\barrido\dx12_f$f.png" | Out-Null
        & "$bin\estudio_cpu.exe" --fotograma $n --salida "target\capturas\barrido\exacto_f$f.png" | Out-Null
        & "$bin\estudio_cpu.exe" --fotograma $n --unorm-12bits --salida "target\capturas\barrido\12bits_f$f.png" | Out-Null
        foreach ($juez in "exacto", "12bits") {
            $salida = & "$bin\comparar.exe" "target\capturas\barrido\dx12_f$f.png" "target\capturas\barrido\${juez}_f$f.png" --listar 5
            $d = [int](($salida | Select-String "distintos: (\d+)").Matches[0].Groups[1].Value)
            $uno = ($salida | Select-String "canal\s+1: (\d+)") | ForEach-Object { [int]$_.Matches[0].Groups[1].Value }
            $r = $resumen[$juez]
            $r.fotogramas++
            $r.distintos += $d
            if ($uno) { $r.de_1 += $uno }
            if ($d -gt 0) {
                $r.con_dif++
                if ($juez -eq "12bits") { Write-Output "  [12bits] fotograma ${n}:"; $salida | Select-Object -Skip 2 | Write-Output }
            }
        }
    }
    foreach ($juez in "exacto", "12bits") {
        $r = $resumen[$juez]
        Write-Output ("Barrido DX12 vs CPU ({0}): {1} fotogramas, {2} con diferencias, {3} píxeles distintos ({4} con diferencia de 1 en un canal, {5} con más)" -f $juez, $r.fotogramas, $r.con_dif, $r.distintos, $r.de_1, ($r.distintos - $r.de_1))
    }
}