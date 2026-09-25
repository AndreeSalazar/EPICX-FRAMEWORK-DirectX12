# Demuestra las 4 reglas de cubo_neutro::Reglas contra la GPU.
#
#   powershell -ExecutionPolicy Bypass -File scripts\reglas.ps1
#
# Para cada regla: captura el fotograma de la GPU, dibuja el mismo con la CPU con la regla
# ENCENDIDA y APAGADA, y cuenta los píxeles distintos. La regla está demostrada si
# encendida da 0 y apagada da más de 0.

$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

cargo build --release --quiet
if ($LASTEXITCODE -ne 0) { throw "cargo build falló" }
$bin = "target\release"
$dir = "target\capturas\reglas"

function Distintos($a, $b) {
    $linea = & "$bin\comparar.exe" $a $b | Select-String "distintos: (\d+)"
    [int]$linea.Matches[0].Groups[1].Value
}

# regla, qué GPU, fotograma, opciones con la regla encendida, opción que la apaga
$casos = @(
    @{ n = 1; nombre = "centro de píxel entero en D3D9";  gpu = "estudio_d3d9";  f = 30;  base = @("--centro-entero"); apagar = $null },
    @{ n = 2; nombre = "subpíxel con empates al par";     gpu = "estudio_d3d12"; f = 60;  base = @();                  apagar = "--subpixel-mitad-arriba" },
    @{ n = 3; nombre = "viewport ndc*(W/2)+W/2";          gpu = "estudio_d3d12"; f = 10;  base = @("--unorm-12bits");  apagar = "--viewport-normalizado" },
    @{ n = 4; nombre = "UNORM8: truncar a 12 bits";       gpu = "estudio_d3d12"; f = 252; base = @("--unorm-12bits");  apagar = "sin --unorm-12bits" }
)

Write-Output "| Regla | GPU | Fotograma | Píxeles distintos con la regla | Sin la regla |"
Write-Output "|---|---|---:|---:|---:|"
foreach ($c in $casos) {
    $f = "{0:D4}" -f $c.f
    $gpu = "$dir\$($c.gpu)_f$f.png"
    & "$bin\$($c.gpu).exe" --fotograma $c.f --salida $gpu | Out-Null

    $con = "$dir\regla$($c.n)_con_f$f.png"
    $con_args = @("--fotograma", "$($c.f)") + $c.base + @("--salida", $con)
    & "$bin\estudio_cpu.exe" $con_args | Out-Null

    # Apagar: la regla 1 se apaga quitando --centro-entero (D3D9 contra el juez de D3D10),
    # la 4 quitando --unorm-12bits; la 2 y la 3 añadiendo su opción.
    $sin_opciones = if ($c.n -eq 1 -or $c.n -eq 4) { @() } else { $c.base + @($c.apagar) }
    $sin = "$dir\regla$($c.n)_sin_f$f.png"
    $sin_args = @("--fotograma", "$($c.f)") + $sin_opciones + @("--salida", $sin)
    & "$bin\estudio_cpu.exe" $sin_args | Out-Null

    Write-Output ("| {0}. {1} | {2} | {3} | {4} | {5} |" -f $c.n, $c.nombre, $c.gpu, $c.f, (Distintos $gpu $con), (Distintos $gpu $sin))
}
