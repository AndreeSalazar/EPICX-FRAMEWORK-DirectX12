# Censo de importaciones: qué DLL y cuántas funciones pide cada .exe de release.
#
#   powershell -ExecutionPolicy Bypass -File scripts\censo.ps1
#
# Usa dumpbin.exe de Visual Studio (vswhere lo encuentra). Imprime tablas Markdown.
# estudio_cpu.exe es la línea base: lo que pide Rust (std + png) sin Direct3D.

$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

cargo build --release --quiet
if ($LASTEXITCODE -ne 0) { throw "cargo build falló" }

$vs = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -products * -property installationPath
$dumpbin = Get-ChildItem "$vs\VC\Tools\MSVC\*\bin\Hostx64\x64\dumpbin.exe" | Select-Object -First 1 -ExpandProperty FullName
if (-not $dumpbin) { throw "no encontré dumpbin.exe" }

$exes = "estudio_cpu", "estudio_d3d9", "estudio_d3d10", "estudio_d3d11", "estudio_d3d12"

# censo[exe][dll] = lista de funciones
$censo = [ordered]@{}
foreach ($exe in $exes) {
    $tabla = [ordered]@{}
    $dll = $null
    foreach ($linea in & $dumpbin /nologo /imports "target\release\$exe.exe") {
        if ($linea -match '^\s{4}(\S+\.dll)\s*$') {
            $dll = $Matches[1].ToLower()
            $tabla[$dll] = New-Object System.Collections.Generic.List[string]
        } elseif ($dll -and $linea -match '^\s{10,}([0-9A-F]+)\s+(\S+)\s*$') {
            $tabla[$dll].Add($Matches[2])
        } elseif ($dll -and $linea -match '^\s{10,}Ordinal\s+(\d+)\s*$') {
            $tabla[$dll].Add("Ordinal $($Matches[1])")
        } elseif ($linea -match 'Summary') {
            $dll = $null
        }
    }
    $censo[$exe] = $tabla
}

$todas = $censo.Values | ForEach-Object { $_.Keys } | Sort-Object -Unique

Write-Output "| DLL | $($exes -join ' | ') |"
Write-Output "|---|$(($exes | ForEach-Object { '---:' }) -join '|')|"
foreach ($dll in $todas) {
    $celdas = foreach ($exe in $exes) { if ($censo[$exe].Contains($dll)) { $censo[$exe][$dll].Count } else { "" } }
    Write-Output "| $dll | $($celdas -join ' | ') |"
}
$totales = foreach ($exe in $exes) { ($censo[$exe].Values | ForEach-Object { $_.Count } | Measure-Object -Sum).Sum }
$dlls = foreach ($exe in $exes) { $censo[$exe].Count }
Write-Output "| **total funciones** | $($totales -join ' | ') |"
Write-Output "| **total DLL** | $($dlls -join ' | ') |"

Write-Output ""
Write-Output "Funciones que cada .exe pide DE MÁS respecto a estudio_cpu.exe:"
Write-Output ""
$base = $censo["estudio_cpu"]
foreach ($exe in $exes | Select-Object -Skip 1) {
    $extra = foreach ($dll in $censo[$exe].Keys) {
        $antes = if ($base.Contains($dll)) { $base[$dll] } else { @() }
        $nuevas = $censo[$exe][$dll] | Where-Object { $antes -notcontains $_ } | Sort-Object
        if ($nuevas) { "``$dll``: $($nuevas -join ', ')" }
    }
    $n = ($censo[$exe].Keys | ForEach-Object { $dll = $_; $antes = if ($base.Contains($dll)) { $base[$dll] } else { @() }; ($censo[$exe][$dll] | Where-Object { $antes -notcontains $_ }).Count } | Measure-Object -Sum).Sum
    Write-Output "- **$exe** (+$n): $($extra -join '; ')"
}
