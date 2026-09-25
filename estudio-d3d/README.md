# estudio-d3d

> **Estudio en Windows; no es BMO-X ni se porta tal cual.**
>
> Es un envoltorio de la API de Microsoft (Direct3D 9, 10, 11 y 12) para estudiar cómo se
> comporta. Nada de lo que llama a Windows cruza a BMO-X. Lo que sí puede cruzar está en
> [`../cubo-neutro`](../cubo-neutro): malla, matrices, luz y un rasterizador por CPU, en
> `no_std` y sin dependencias.

```
cubo-neutro/          (raíz del repo) no_std, sin dependencias: lo portable
estudio-d3d/
├── base/             ventana Win32, bucle, D3DCompile, PNG           (Windows)
├── cpu/              estudio_cpu (el juez a PNG) y comparar          (sin Direct3D)
├── d3d9/  d3d10/  d3d11/  d3d12/   un cubo por versión              (Windows)
├── referencia/       PNG de DX12 y CPU, fotogramas 0, 30 y 60
├── scripts/          referencia.ps1, reglas.ps1, medir.ps1, censo.ps1
├── DICCIONARIO.md    cada llamada: qué hace con la memoria y con la sincronización
└── CENSO.md          DLL y funciones que importa cada .exe
```

Los cuatro cubos toman de `cubo-neutro` la malla, las matrices y la luz: CPU y GPU parten de
los mismos números. La rotación depende del **número de fotograma** (1° por fotograma), no
del reloj.

## Uso

```bash
cargo run -p estudio_d3d12 --release
```

| Argumento | Qué hace |
|---|---|
| (ninguno) | Interactivo con VSync. ESC sale. |
| `--sin-vsync` | Interactivo sin VSync. |
| `--fotograma N [--salida ruta.png]` | Dibuja solo el fotograma N en una ventana oculta, lee el back buffer, lo guarda como PNG y sale. |
| `--medir N` | Dibuja los fotogramas 0..N-1 sin VSync e imprime los FPS. |
| `--corregir-medio-pixel` | Solo D3D9: desplaza la geometría −0.5 px (ver abajo). |

`estudio_cpu --fotograma N [--salida ruta.png]` hace lo mismo con el rasterizador de
`cubo-neutro`. Tiene una opción para apagar o cambiar cada una de las 4 reglas:
`--centro-entero`, `--subpixel-mitad-arriba`, `--viewport-normalizado` y `--unorm-12bits`.
`scripts/reglas.ps1` las demuestra (tabla en [`../cubo-neutro/README.md`](../cubo-neutro/README.md)).
`comparar a.png b.png [--mapa m.png] [--listar N]` cuenta los
píxeles distintos.

## GPU contra CPU

Medido en esta máquina: NVIDIA GeForce RTX 3060, Windows 11, 1280×720, **sin antialiasing**
(`SampleDesc.Count = 1` en todas las versiones, `MultisampleEnable = FALSE`). Para repetirlo:

```bash
powershell -ExecutionPolicy Bypass -File scripts/referencia.ps1
```

Píxeles distintos respecto a `referencia/cpu/` (921 600 por imagen):

| | f0000 | f0030 | f0060 |
|---|---:|---:|---:|
| **DX12** (`referencia/dx12/`) | **0** | **0** | **0** |
| D3D11 | 0 | 0 | 0 |
| D3D10 | 0 | 0 | 0 |
| D3D9 | 536 | 732 | 677 |
| D3D9 `--corregir-medio-pixel` | 0 | 0 | 0 |

Tres fotogramas pueden salir bien por casualidad, así que también se barrió la vuelta completa:

```bash
powershell -ExecutionPolicy Bypass -File scripts/referencia.ps1 -Barrido 1
```

DX12 contra CPU en los fotogramas 0..359:

| Juez | Fotogramas con diferencias | Píxeles distintos | De ellos, diferencia de 1 en un canal | Otra cara u otro fondo |
|---|---:|---:|---:|---:|
| `dibujar_por_cpu` (redondeo exacto a 8 bits) | 37 de 360 | 1 491 885 | 1 491 884 | 1 |
| `--unorm-12bits` | 1 de 360 | 1 | 0 | 1 |

### Lo que se aprendió al hacerlos coincidir

Cada punto salió de una diferencia medida, no de la documentación:

1. **D3D9 pone los centros de píxel en coordenadas enteras.** D3D10+ los pone en +0.5. Las
   536–732 diferencias de D3D9 son todas de borde, y desaparecen al mover la geometría medio
   píxel en el vertex shader (`--corregir-medio-pixel`). Al portar desde D3D9 hay que tenerlo en cuenta.
2. **Ajuste a subpíxel con redondeo al par.** La GPU pasa las posiciones a punto fijo de 1/256
   de píxel. Un vértice en x·256 = 218350.5 exacto (fotograma 60) caía en otro subpíxel si se
   redondeaba "mitades hacia arriba". Ahora `num::redondear_par` lo resuelve, y el test
   `redondeo_al_par` guarda ese caso.
3. **El orden de las operaciones del viewport importa en `f32`.** Con x·256 ≈ 2·10⁵, un `f32`
   solo distingue 1/64 de subpíxel. `(ndc·0.5 + 0.5)·ancho` y `ndc·(ancho/2) + ancho/2` dan
   resultados distintos, y la GPU coincide con el segundo.
4. **La conversión a 8 bits de esta GPU no es `round(x·255)`.** Todas las diferencias de 1 en un
   canal desaparecen si se modela como "truncar a 12 bits (`floor(x·4096)`) y luego redondear".
   D3D permite 0,6 unidades de error en esa conversión, así que es legal y puede cambiar en otra
   GPU. Por eso el juez (`Reglas::D3D10`) sigue usando el redondeo exacto, y el modelo de
   12 bits (`a_unorm8_truncar_12`) hay que pedirlo aparte.
5. **Queda 1 píxel sin explicar** (fotograma 32, píxel 523,199: la GPU pinta la cara
   verde y la CPU el fondo). Un vértice cae a 1/256 de subpíxel de un empate. Se probaron FMA en
   el vertex shader, cinco órdenes para el viewport y ±1 ulp en `1/w`, y ninguno lo reproduce.

## Rendimiento

Sin VSync, 20 000 fotogramas, 3 corridas por versión (`--medir 20000`):

```bash
powershell -ExecutionPolicy Bypass -File scripts/medir.ps1
```

| | D3D9 | D3D10 | D3D11 | D3D12 |
|---|---|---|---|---|
| FPS (3 corridas) | 8833 · 8502 · 8535 | 9444 · 9380 · 9479 | 9451 · 9387 · 9504 | 3952 · 3790 · 3610 |

El rasterizador por CPU (un hilo, mismo 1280×720) tardó 5,5 / 4,6 / 4,8 ms en los fotogramas
0 / 30 / 60, según la salida de `estudio_cpu` en la misma corrida.

Estas cifras son de esta máquina en ese momento, y cambian con la carga del sistema. Con un
solo cubo no se mide el costo de dibujar: domina el ciclo de presentar y esperar. D3D12 hace
en cada fotograma lo que en 9/10/11 hace el driver (espera por fence, reinicio del allocator,
barreras), y presenta con flip model a través del compositor. No se midió cuál de esas partes
pesa más.

D3D12 sin VSync necesita `DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING` + `DXGI_PRESENT_ALLOW_TEARING`.
Sin eso, el compositor retiene el buffer mostrado hasta el refresco y `Present(0)` queda atado
al monitor. El cubo lo activa si el sistema lo soporta, e imprime `tearing: sí/no` al arrancar.

## Tamaño del código

Líneas por archivo (`(Get-Content archivo).Count`):

| `d3d9/…/cubo_d3d9.rs` | `d3d10/…/cubo_d3d10.rs` | `d3d11/…/cubo_d3d11.rs` | `d3d12/…/cubo_d3d12.rs` | `base/src/lib.rs` | `cubo-neutro/src/*.rs` |
|---:|---:|---:|---:|---:|---:|
| 204 | 216 | 226 | 495 | 274 | 717 |

## Límites (a propósito)

- La ventana tiene tamaño fijo, así que nunca se recrea el swap chain.
- No hay capa de debug de D3D (requiere instalar "Graphics Tools").
- En D3D12, los vértices viven en un heap UPLOAD. Un motor real los copia a DEFAULT.
- D3D7/D3D8 quedaron fuera: `d3d8.dll` solo existe en 32 bits en este Windows 11 y el SDK no
  trae `d3d8.h`.

## Relación con EPICX

`examples/cube_dx12.rs` (en la raíz del repo) dibuja en la CPU con `softbuffer`; de DX12 solo
usa DXGI para detectar la GPU. `estudio-d3d/d3d12` es el primer cubo del repo que dibuja con
D3D12 en la GPU.
