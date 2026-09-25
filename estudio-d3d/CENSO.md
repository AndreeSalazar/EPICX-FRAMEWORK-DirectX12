# Censo de importaciones

> Estudio en Windows; no es BMO-X ni se porta tal cual.

Qué DLL y cuántas funciones pide cada `.exe` de release. Es la tabla de importaciones del
PE, leída con `dumpbin /imports` (MSVC 14.44, VS 2022). Para regenerarla:

```bash
powershell -ExecutionPolicy Bypass -File scripts/censo.ps1
```

`estudio_cpu.exe` es la **línea base**: el rasterizador de `cubo-neutro` con `std` y `png`,
sin ventana ni Direct3D. Lo que los otros piden de más es el costo de Windows + D3D.

## Funciones importadas por DLL

| DLL | estudio_cpu | estudio_d3d9 | estudio_d3d10 | estudio_d3d11 | estudio_d3d12 |
|---|---:|---:|---:|---:|---:|
| api-ms-win-core-synch-l1-2-0.dll | 3 | 3 | 3 | 3 | 3 |
| api-ms-win-core-winrt-error-l1-1-0.dll |  | 1 | 1 | 1 | 1 |
| api-ms-win-crt-heap-l1-1-0.dll | 2 | 2 | 2 | 2 | 2 |
| api-ms-win-crt-locale-l1-1-0.dll | 1 | 1 | 1 | 1 | 1 |
| api-ms-win-crt-math-l1-1-0.dll | 2 | 1 | 1 | 1 | 1 |
| api-ms-win-crt-runtime-l1-1-0.dll | 18 | 18 | 18 | 18 | 18 |
| api-ms-win-crt-stdio-l1-1-0.dll | 2 | 2 | 2 | 2 | 2 |
| d3d10.dll |  |  | 1 |  |  |
| d3d11.dll |  |  |  | 1 |  |
| d3d12.dll |  |  |  |  | 2 |
| d3d9.dll |  | 1 |  |  |  |
| d3dcompiler_47.dll |  | 1 | 1 | 1 | 1 |
| dxgi.dll |  |  |  |  | 1 |
| kernel32.dll | 53 | 57 | 54 | 54 | 55 |
| ntdll.dll | 2 | 2 | 2 | 2 | 2 |
| oleaut32.dll |  | 3 | 3 | 3 | 3 |
| user32.dll |  | 11 | 11 | 11 | 11 |
| vcruntime140.dll | 9 | 9 | 9 | 9 | 9 |
| **total funciones** | 92 | 112 | 109 | 109 | 112 |
| **total DLL** | 9 | 14 | 14 | 14 | 15 |

## Lo que cada versión pide de más respecto a `estudio_cpu.exe`

| .exe | Extra | De la API gráfica | De la ventana (user32) | Otras |
|---|---:|---|---|---|
| estudio_d3d9 | +21 | `d3d9.dll`: Direct3DCreate9 · `d3dcompiler_47.dll`: D3DCompile | 11 | `kernel32`: CreateWaitableTimerExW, SetWaitableTimer, Sleep (el `sleep` del *device lost*), LoadLibraryExA · `oleaut32`: GetErrorInfo, SysFreeString, SysStringLen · RoOriginateErrorW |
| estudio_d3d10 | +18 | `d3d10.dll`: D3D10CreateDeviceAndSwapChain · D3DCompile | 11 | LoadLibraryExA · `oleaut32` × 3 · RoOriginateErrorW |
| estudio_d3d11 | +18 | `d3d11.dll`: D3D11CreateDeviceAndSwapChain · D3DCompile | 11 | LoadLibraryExA · `oleaut32` × 3 · RoOriginateErrorW |
| estudio_d3d12 | +21 | `dxgi.dll`: CreateDXGIFactory2 · `d3d12.dll`: D3D12CreateDevice, D3D12SerializeRootSignature · D3DCompile | 11 | `kernel32`: CreateEventW (el evento del fence), LoadLibraryExA · `oleaut32` × 3 · RoOriginateErrorW |

Las 11 de `user32.dll` son las mismas en las cuatro versiones: AdjustWindowRect,
CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, LoadCursorW, PeekMessageW,
PostQuitMessage, RegisterClassExW, SetWindowTextW, TranslateMessage.
`oleaut32` (GetErrorInfo…), `RoOriginateErrorW` y `LoadLibraryExA` los trae `windows-result`,
una dependencia del crate `windows`, para convertir errores COM/HRESULT en mensajes
(`windows-result-0.2.0/src/bindings.rs`).

## Lo que el censo NO ve

La tabla de importaciones solo lista las **funciones exportadas** de cada DLL. Direct3D es
COM: después de `D3D12CreateDevice`, todas las demás llamadas (`CreateCommandQueue`,
`ExecuteCommandLists`, `Present`, `ResourceBarrier`…) se hacen **a través de punteros en la
vtable** del objeto, y no aparecen aquí. Por eso `d3d12.dll` aporta 2 importaciones aunque el
cubo haga decenas de llamadas distintas a D3D12. Esas están en [DICCIONARIO.md](DICCIONARIO.md).

Para portar a BMO-X, cada fila de esta tabla es algo que el sistema operativo tiene que dar
(ventana, eventos, temporizadores, cargador de DLL) o algo que desaparece al no haber Windows.
Cada fila del diccionario es algo que un driver propio tiene que hacer.
