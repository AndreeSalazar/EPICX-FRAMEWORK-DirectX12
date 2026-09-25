# cubo-neutro

El cubo sin plataforma. Es **la parte que puede cruzar a BMO-X**. Todo lo que llama a
Windows vive en [`../estudio-d3d`](../estudio-d3d) y no cruza.

Reglas:
- `#![no_std]`, sin dependencias (`cargo tree` muestra solo este crate) y sin memoria dinámica.
- Solo `f32` y aritmética propia ([`src/num.rs`](src/num.rs)): seno, coseno, raíz y
  redondeos escritos con `+ - * /`. No usa libm ni intrínsecos.

| Módulo / función | Qué es |
|---|---|
| `vertices()`, `indices()` | 24 vértices (posición, normal, color; 40 B cada uno) y 36 índices `u16` |
| `mat::{rotacion_x, rotacion_y, vista, perspectiva, mul, transformar}` | Matrices 4×4 en orden de columnas, mano izquierda, profundidad 0..1 |
| `angulo_de_fotograma(n)`, `constantes(angulo, aspecto)` | Rotación por **número de fotograma** (1° por fotograma), WVP, World y luz (144 B, `repr(C)`) |
| `iluminar`, `a_unorm8`, `a_unorm8_truncar_12`, `empaquetar` | El mismo cálculo que el pixel shader y las dos conversiones a 8 bits |
| `dibujar_por_cpu(angulo, ancho, alto, &mut [u32])` | El rasterizador de referencia ("el juez"), con `Reglas::D3D10` |
| `dibujar_por_cpu_con(…, &Reglas)` | Igual, con cada una de las 4 reglas de abajo encendida o apagada |

El juez hace, como D3D10+: culling por signo del área, regla top-left, depth test `LESS` y
nada de antialiasing. Los cuatro cubos D3D toman de aquí la malla, las matrices y la luz, así
que CPU y GPU parten de los mismos números.

## Las 4 reglas

Son lo que hizo falta para que la CPU diera **los mismos píxeles** que la GPU. Ninguna salió de
la documentación: cada una apareció como una diferencia medida. Todas son campos de
`Reglas`, y `estudio_cpu` tiene una opción para apagar cada una.

| # | Regla | Campo de `Reglas` | Qué pasa si no se cumple |
|---|---|---|---|
| 1 | **Centro de píxel.** D3D10/11/12 muestrean en (x + 0.5, y + 0.5); D3D9 en (x, y). | `centro_medio_pixel` (`Reglas::D3D9` lo pone en `false`) | La imagen de D3D9 sale medio píxel desplazada: cambian los píxeles de todos los bordes. |
| 2 | **Subpíxel con redondeo al par.** Las posiciones se ajustan a 1/256 de píxel, y un empate (x·256 = n + 0.5) va al **par**. | `subpixel_al_par` | Un vértice en x·256 = 218350.5 (fotograma 60) cae en el subpíxel vecino y un píxel de borde cambia de cara. |
| 3 | **Orden del viewport.** `x = ndc·(ancho/2) + ancho/2`, no `(ndc·0.5 + 0.5)·ancho`. Con x·256 ≈ 2·10⁵ un `f32` solo tiene 1/64 de subpíxel, y los dos órdenes redondean distinto. | `viewport_escalado` | Algunos vértices cambian de subpíxel y un píxel de borde cambia de cara. |
| 4 | **UNORM8 = truncar a 12 bits y redondear**: `(floor(x·4096)·255 + 2048) / 4096`. Es lo que hace la GPU medida (RTX 3060). D3D permite hasta 0,6 unidades de error, así que **otra GPU puede hacerlo distinto**. | `unorm8: a_unorm8_truncar_12` (el juez usa `a_unorm8`, el redondeo exacto) | Caras enteras salen 1 unidad más claras en un canal cuando el valor exacto cae cerca de ,5. |

### Cómo se demuestra cada una

Desde `../estudio-d3d`, en Windows, con la GPU real. Cada fila captura un fotograma de la GPU,
lo dibuja con la CPU con la regla y sin ella, y cuenta los píxeles distintos con `comparar`:

```bash
powershell -ExecutionPolicy Bypass -File scripts/reglas.ps1
```

| # | Comandos (en `target/release/`) | Con la regla | Sin la regla |
|---|---|---:|---:|
| 1 | `estudio_d3d9 --fotograma 30 --salida g.png`<br>`estudio_cpu --fotograma 30 --centro-entero --salida con.png`<br>`estudio_cpu --fotograma 30 --salida sin.png` | **0** | 732 |
| 2 | `estudio_d3d12 --fotograma 60 --salida g.png`<br>`estudio_cpu --fotograma 60 --salida con.png`<br>`estudio_cpu --fotograma 60 --subpixel-mitad-arriba --salida sin.png` | **0** | 1 |
| 3 | `estudio_d3d12 --fotograma 10 --salida g.png`<br>`estudio_cpu --fotograma 10 --unorm-12bits --salida con.png`<br>`estudio_cpu --fotograma 10 --unorm-12bits --viewport-normalizado --salida sin.png` | **0** | 1 |
| 4 | `estudio_d3d12 --fotograma 252 --salida g.png`<br>`estudio_cpu --fotograma 252 --unorm-12bits --salida con.png`<br>`estudio_cpu --fotograma 252 --salida sin.png` | **0** | 74 348 |

En cada fila, las dos últimas columnas son `comparar g.png con.png` y `comparar g.png sin.png`.
Son valores medidos en una RTX 3060 a 1280×720. La regla 3 usa `--unorm-12bits` para que solo
cuenten las diferencias de cobertura y no las de color.
Con las 4 reglas, en la vuelta completa (fotogramas 0..359) queda **1 píxel** distinto sin
explicar: fotograma 32, píxel (523, 199). Ver [`../estudio-d3d/README.md`](../estudio-d3d/README.md).

Para comprobarlas en otro sistema, lo que hay que medir es:
1. Dónde muestrea el centro del píxel.
2. Cómo se redondean los empates al pasar a punto fijo.
3. En qué orden se calcula el viewport.
4. Cómo convierte a 8 bits la salida de color.

```bash
cargo test
```

Para comprobarlo en un target sin sistema operativo (no lo hice en esta máquina, que solo
tiene instalado `x86_64-pc-windows-msvc`):

```bash
rustup target add x86_64-unknown-none
```
```bash
cargo build --target x86_64-unknown-none
```
