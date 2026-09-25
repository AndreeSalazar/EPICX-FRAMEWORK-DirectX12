# BMOX — un cubo 3D por versión de Direct3D

Laboratorio para estudiar cómo evoluciona la API de Direct3D (9 → 10 → 11 → 12) y
portearla a **BMO-X**. Cada carpeta dibuja **el mismo cubo** (misma geometría, misma
cámara, misma luz, mismo fondo), así que lo único que cambia entre ellas es la API.

```
BMOX/
├── base/       lo común: ventana Win32, cubo (24 vértices / 36 índices), matrices, compilador HLSL
├── BMOX-9/     Direct3D 9   (2002)  → bmox9::Bmox9
├── BMOX-10/    Direct3D 10  (2006)  → bmox10::Bmox10
├── BMOX-11/    Direct3D 11  (2009)  → bmox11::Bmox11
└── BMOX-12/    Direct3D 12  (2015)  → bmox12::Bmox12
```

Cada versión tiene 3 archivos: `main.rs` (5 líneas), `bmoxN.rs` (toda la API) y `cubo.hlsl`.
Todas implementan el mismo trait `BmoxRender { fn dibujar(&mut self, &Constantes) }`, así
BMO-X puede elegir el backend sin tocar el resto.

## Ejecutar

```bash
cargo run -p bmox9 --release
cargo run -p bmox10 --release
cargo run -p bmox11 --release
cargo run -p bmox12 --release
```

ESC o cerrar la ventana termina y muestra el promedio de FPS. `VSYNC` está en
[`base/src/lib.rs`](base/src/lib.rs) (por defecto `false`, igual que el ejemplo de EPICX).

## Qué cambia en cada versión

| Concepto | D3D9 | D3D10 | D3D11 | D3D12 |
|---|---|---|---|---|
| Objeto que dibuja | `IDirect3DDevice9` | `ID3D10Device` | `ID3D11DeviceContext` | `ID3D12GraphicsCommandList` |
| Objeto que crea recursos | el mismo device | el mismo device | `ID3D11Device` | `ID3D12Device` |
| Swap chain | dentro del device (`D3DPRESENT_PARAMETERS`) | DXGI (`DISCARD`) | DXGI (`DISCARD`) | DXGI flip model (`FLIP_DISCARD`, 2 buffers) sobre la **cola** |
| Depth buffer | automático (`EnableAutoDepthStencil`) | textura + DSV | textura + DSV | recurso + heap DSV |
| Formato de vértice | `IDirect3DVertexDeclaration9` | input layout | input layout | input layout dentro del PSO |
| Constantes | registros `c0..c7` (`SetVertexShaderConstantF`) | constant buffer b0 | constant buffer b0 | root CBV b0 (ranuras de 256 bytes por frame) |
| Estado | `SetRenderState` suelto | objetos de estado | objetos de estado | todo en un **PSO** inmutable |
| Shaders | SM3 (`vs_3_0`), opcionales | SM4 obligatorio | SM5 | SM5 con FXC (SM6 necesita DXC) |
| Semánticas HLSL | `POSITION` / `COLOR0` | `SV_POSITION` / `SV_TARGET` | igual | igual |
| Escribir buffers | `Lock` / `Unlock` | `UpdateSubresource` | `UpdateSubresource` | `Map` persistente en heap UPLOAD |
| Sincronización CPU↔GPU | la hace el driver | la hace el driver | la hace el driver | **tú**: fences + eventos |
| Transiciones de recursos | driver | driver | driver | **tú**: `ResourceBarrier` |
| Pérdida de device | sí: `TestCooperativeLevel` + `Reset` | no | no | (device removed, no se maneja aquí) |
| Líneas en `bmoxN.rs` | 183 | 183 | 190 | 410 |

### Medido en esta máquina (RTX 3060, 1280×720, sin VSync, ~3 s)

| | BMOX-9 | BMOX-10 | BMOX-11 | BMOX-12 |
|---|---|---|---|---|
| FPS promedio | ~8 700 | ~9 200 | ~8 900 | ~6 000 |

Con un solo cubo, el cuello de botella es el ciclo presentar/esperar y no la GPU. BMOX-12 da
menos FPS porque espera al fence en cada frame y usa flip model en ventana sin
`DXGI_FEATURE_PRESENT_ALLOW_TEARING`. D3D12 gana cuando hay miles de draw calls o varios
hilos grabando command lists, y eso con un cubo no se ve.

## Qué es igual en las cuatro (y vive en `base`)

- Sistema de mano izquierda, profundidad 0..1 (`look_at_lh`, `perspective_lh`).
- Caras frontales en sentido horario; se descartan las traseras (es el valor por defecto en todas las versiones).
- Matrices subidas por columnas (`to_cols_array`) y `mul(M, v)` en HLSL. Funciona igual con
  registros D3D9 y con cbuffers, porque el empaquetado por defecto de HLSL es `column_major`.
- Shaders compilados en tiempo de ejecución con `D3DCompile` (d3dcompiler_47.dll, viene con Windows).
  D3DX9 ya no está en el SDK moderno, así que D3D9 también usa `D3DCompile`.

## Límites (a propósito, para que el código se lea fácil)

- La ventana tiene tamaño fijo, así que no se recrea el swap chain al redimensionar.
- No hay capa de debug (`D3D12GetDebugInterface` / `D3D11_CREATE_DEVICE_DEBUG`), porque requiere
  instalar "Graphics Tools" en Windows.
- En BMOX-12 los vértices viven en un heap UPLOAD. Un motor real los copia a un heap DEFAULT
  con una command list de copia.
- DX7 y DX8 quedaron fuera: `d3d8.dll` solo existe en 32 bits en Windows 11 y el SDK no trae
  `d3d8.h`. DX7 es DirectDraw + pipeline fijo.

## Relación con EPICX

`examples/cube_dx12.rs` (en la raíz del repo) **no dibuja con DirectX 12**: es un rasterizador
por software (`softbuffer`) y usa DXGI solo para detectar la GPU. `BMOX-12` es el primer cubo del
repo que sí se dibuja con D3D12 en la GPU, y sirve de base para completar `src/dx12/`.
