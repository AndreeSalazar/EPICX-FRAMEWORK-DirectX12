# Diccionario: qué hace cada llamada con la memoria y con la sincronización

> Estudio en Windows; no es BMO-X ni se porta tal cual.

Cada fila es una llamada que el cubo hace de verdad (ver `d3dN/src/cubo_d3dN.rs`).
Dos columnas: **memoria de la GPU** (qué se reserva, se escribe o se lee, y cuántos bytes) y
**sincronización** (quién espera a quién: CPU, cola de la GPU, compositor).

Las cifras de bytes salen de la propia ejecución:

```bash
target/release/estudio_d3d12.exe --fotograma 0
```

Ese comando imprime las líneas `[memoria]`: bytes pedidos, bytes que el driver reserva según
`GetResourceAllocationInfo` y tamaño de descriptor según `GetDescriptorHandleIncrementSize`.
Son valores de una RTX 3060 con su driver; otra GPU puede dar otros.
Las demás cifras son aritmética del código: 1280 × 720 × 4 B = 3 686 400 B, `Vertice` = 40 B,
24 vértices = 960 B, 36 índices `u16` = 72 B y `Constantes` = 144 B (comprobado en `cubo-neutro`,
test `malla`).

---

## Direct3D 12 (completo)

### Creación (una vez)

| Llamada | Memoria de la GPU | Sincronización |
|---|---|---|
| `CreateDXGIFactory2` | Ninguna. Objeto del runtime DXGI en la CPU. | Ninguna. |
| `EnumAdapterByGpuPreference` | Ninguna. Solo describe el adaptador. | Ninguna. |
| `D3D12CreateDevice` | Crea el contexto del driver para ese adaptador. Todavía no reserva memoria para recursos del cubo. | Ninguna visible. La llamada bloquea la CPU mientras el driver se inicializa. |
| `CreateCommandQueue` (DIRECT) | Estructuras del driver para una cola de comandos de la GPU. | Crea **la** cola: todo lo que se envíe se ejecuta en orden dentro de ella. |
| `CheckFeatureSupport(PRESENT_ALLOW_TEARING)` | Ninguna. | Solo pregunta si `Present` puede no esperar al refresco. |
| `CreateSwapChainForHwnd` (FLIP_DISCARD, 2 buffers) | 2 texturas RGBA8 de 3 686 400 B pedidos, **3 932 160 B reservados** cada una (alineación 65 536 B). | Se ata a la **cola**, no al device. Los buffers pasan por turnos entre la app y el compositor (DWM). |
| `MakeWindowAssociation(NO_ALT_ENTER)` | Ninguna. | Ninguna. |
| `CreateDescriptorHeap` (RTV × 2, DSV × 1) | Memoria de **CPU**: heaps no visibles para shaders. **RTV = 32 B, DSV = 8 B** por descriptor. | Ninguna. |
| `GetDescriptorHandleIncrementSize` | Ninguna. Devuelve el tamaño de descriptor (32 / 8 B). | Ninguna. |
| `GetBuffer` + `CreateRenderTargetView` | Escribe un descriptor de 32 B en el heap RTV. No toca la textura. | Ninguna. Es escritura de CPU. |
| `CreateCommittedResource` (DEFAULT, D32) | Profundidad: 3 686 400 B pedidos, **3 932 160 B reservados** en memoria de la GPU. Guarda el valor de borrado 1.0. | La CPU espera la reserva. El recurso nace en estado `DEPTH_WRITE`. |
| `CreateDepthStencilView` | Escribe un descriptor de 8 B en el heap DSV. | Ninguna. |
| `CreateCommittedResource` (UPLOAD) × 3 | Vértices 960 B, índices 72 B y constantes 512 B (2 × 256). **65 536 B reservados cada uno**, en memoria que la CPU escribe y la GPU lee. | Estado `GENERIC_READ` desde el principio. No hay barreras para estos buffers. |
| `Map` / `memcpy` / `Unmap` (vértices, índices) | La CPU escribe 960 B y 72 B directamente en la memoria que leerá la GPU. No hay copia intermedia. | Ninguna. Es seguro porque la GPU todavía no ha recibido nada que los use. |
| `Map` persistente (constantes) | Puntero de CPU fijo a los 512 B. Cada fotograma escribe 144 B en su ranura de 256 B. | **Ninguna automática.** Si la CPU escribe una ranura que la GPU está leyendo, es una carrera. Por eso hay 2 ranuras y el fence (abajo). |
| `GetGPUVirtualAddress` | Ninguna. Devuelve la dirección virtual de 64 bits del buffer en la GPU. | Ninguna. |
| `D3D12SerializeRootSignature` + `CreateRootSignature` | Objeto de CPU. Declara 1 parámetro: un root CBV, es decir, una dirección de 8 B por draw. | Ninguna. |
| `D3DCompile` (d3dcompiler_47) | Ninguna. Produce bytecode DXBC en memoria de CPU. | Bloquea la CPU mientras compila. |
| `CreateGraphicsPipelineState` | El driver traduce el DXBC a código de la GPU y lo guarda (tamaño interno, no visible). Junta en un objeto shaders, input layout (40 B por vértice), raster, blend y depth. | Bloquea la CPU mientras el driver compila. Después, cambiar de PSO no compila nada. |
| `CreateCommandAllocator` × 2 | Memoria donde se escriben los comandos grabados, una por fotograma en vuelo. | **Regla:** no se puede reiniciar mientras la GPU ejecuta comandos que salieron de él. |
| `CreateCommandList` + `Close` | Objeto de grabación. Nace abierto y se cierra enseguida. | Ninguna. |
| `CreateFence(0)` + `CreateEventW` | Un contador de 64 bits que escribe la GPU y lee la CPU, más un evento del sistema operativo. | Es **el** mecanismo de espera entre la CPU y la GPU en este cubo. |
| `GetCopyableFootprints` + `CreateCommittedResource` (READBACK), solo con `--fotograma` | 3 686 400 B pedidos (fila de 5120 B, múltiplo de 256), **3 735 552 B reservados**, en memoria que la GPU escribe y la CPU lee. | Nace en `COPY_DEST`. |

### Cada fotograma

| Llamada | Memoria de la GPU | Sincronización |
|---|---|---|
| `GetCurrentBackBufferIndex` | Ninguna. Dice cuál de los 2 back buffers toca. | Ninguna. |
| `GetCompletedValue` / `SetEventOnCompletion` / `WaitForSingleObject` | Ninguna. | **La CPU se bloquea** hasta que la GPU termina el fotograma que usó esta ranura hace 2 fotogramas. Así hay como máximo 2 fotogramas en vuelo. |
| `memcpy` a la ranura de constantes | Escribe 144 B en la ranura `i` (de 256 B). | Es seguro solo porque la espera anterior garantiza que la GPU ya no lee esa ranura. |
| `ID3D12CommandAllocator::Reset` | Reutiliza la memoria de comandos del allocator `i`. | Es válido solo por la misma espera. Si la GPU todavía lo usa, es un error. |
| `ID3D12GraphicsCommandList::Reset(allocator, pso)` | Empieza a grabar en ese allocator. | Solo CPU. **Grabar no envía nada a la GPU.** |
| `SetGraphicsRootSignature` | Graba comando. | Ninguna. |
| `SetGraphicsRootConstantBufferView(0, dirección + i·256)` | Graba la dirección de 8 B. Al ejecutar el draw, la GPU lee los 144 B de la memoria UPLOAD. | Ninguna. La coherencia depende de la espera por fence. |
| `RSSetViewports` / `RSSetScissorRects` | Graba comandos. | Ninguna. |
| `ResourceBarrier` PRESENT → RENDER_TARGET | Graba una transición del back buffer. | Dentro de la cola: la GPU no escribe en el buffer hasta que termina lo anterior sobre él (el compositor ya lo soltó). La CPU no espera. |
| `OMSetRenderTargets` | Graba los descriptores RTV (32 B) y DSV (8 B) que se usarán. | Ninguna. |
| `ClearRenderTargetView` / `ClearDepthStencilView` | Al ejecutarse, la GPU escribe 3 686 400 B de color y borra la profundidad (el driver puede usar compresión y no escribir todos los bytes). | Orden de cola. |
| `IASetPrimitiveTopology` / `IASetVertexBuffers` / `IASetIndexBuffer` | Graba dirección y tamaño: 960 B con paso de 40, y 72 B `R16_UINT`. | Ninguna. |
| `DrawIndexedInstanced(36, 1, …)` | Al ejecutarse, la GPU lee 72 B de índices y los vértices que usan, lee 144 B de constantes y escribe color y profundidad donde cubren los 12 triángulos. | Orden de cola. |
| `ResourceBarrier` RENDER_TARGET → COPY_SOURCE (captura) | Graba una transición. | La copia no empieza hasta que terminan las escrituras del draw. |
| `CopyTextureRegion` (captura) | Al ejecutarse, la GPU copia 3 686 400 B del back buffer al buffer READBACK, con filas de 5120 B. | Orden de cola. |
| `ResourceBarrier` → PRESENT | Graba la transición de vuelta. | El compositor no lee el buffer antes de que la GPU termine con él. |
| `Close` | Termina la grabación. | Solo CPU. |
| `ExecuteCommandLists` | Envía la lista a la cola. | **La CPU no espera**: vuelve enseguida y la GPU ejecuta cuando le toca. |
| `Present(sync, ALLOW_TEARING si no hay VSync)` | Entrega el back buffer al compositor. | Con VSync (1): espera el refresco. Sin VSync con tearing: no espera el refresco. Sin tearing, el compositor retiene el buffer hasta el refresco y el siguiente fotograma queda atado al monitor aunque `sync = 0`. Por eso el cubo pide `ALLOW_TEARING` cuando no hay VSync. |
| `Signal(fence, n)` | Encola "escribe n en el fence" detrás de lo anterior. | La CPU no espera. Cuando la GPU llega ahí, el fence vale n: ese es el valor que esperará el fotograma que reutilice esta ranura. |
| (captura) espera por fence + `Map` del READBACK | La CPU lee 3 686 400 B, fila por fila con paso de 5120 B. | **La CPU se bloquea** hasta que la GPU terminó la copia. Leer antes daría basura. |

### Al cerrar

| Llamada | Memoria | Sincronización |
|---|---|---|
| `Drop`: esperar el último fence | Ninguna. | La CPU espera que la GPU termine todo antes de liberar recursos. |
| `Unmap` (constantes) + `CloseHandle(evento)` | Suelta el puntero persistente. | Ninguna. |

---

## Direct3D 9 (punto de partida del estudio)

No se estudia D3D8, así que D3D9 va completo pero resumido.

| Llamada | Memoria de la GPU | Sincronización |
|---|---|---|
| `Direct3DCreate9` | Ninguna. | Ninguna. |
| `CreateDevice` | **El device crea solo** el back buffer X8R8G8B8 (3 686 400 B) y la profundidad D24S8 automática (3 686 400 B). Los bytes reservados no se pueden consultar. | El device tiene un búfer de comandos interno. **El driver decide** cuándo enviarlo a la GPU. |
| `CreateVertexBuffer` / `CreateIndexBuffer` (`D3DPOOL_MANAGED`) | **Dos copias**: una en RAM del sistema (960 B / 72 B) que guarda el runtime, y otra en la GPU que el runtime sube cuando hace falta. | Ninguna visible. |
| `Lock` / `Unlock` | Se escribe en la copia de RAM. El runtime sube los datos a la GPU antes de usarlos. | Sin espera, porque la copia de la GPU no se toca hasta la subida. |
| `CreateVertexDeclaration` / `CreateVertexShader` / `CreatePixelShader` | Objetos del driver (shaders traducidos a código de la GPU). | La CPU espera la creación. |
| `SetVertexShaderConstantF` (9 registros = 144 B) / `SetPixelShaderConstantF` (1 registro = 16 B) | **No hay buffer**: el runtime copia los floats dentro de su búfer de comandos. | Ninguna. La copia es inmediata en la CPU. |
| `Clear` / `BeginScene` / `DrawIndexedPrimitive` / `EndScene` | Se añaden comandos al búfer interno. | El driver puede enviar el búfer en `EndScene`, en `Present` o cuando se llena. |
| `Present(INTERVAL_IMMEDIATE)` | Muestra el back buffer. | La CPU puede bloquearse si el driver ya tiene demasiados fotogramas encolados (el límite lo pone el driver). |
| `TestCooperativeLevel` / `Reset` | **Device lost**: la memoria de la GPU se pierde. Los recursos `MANAGED` se restauran desde su copia en RAM y los `DEFAULT` hay que recrearlos. | La app debe dejar de dibujar hasta que `Reset` funcione. |
| `GetRenderTargetData` (captura) | La GPU copia 3 686 400 B del back buffer a una superficie en RAM del sistema. | **La CPU se bloquea** hasta que la GPU termina todo lo anterior y la copia. |

## Direct3D 10: qué cambia respecto a D3D9

| Cambio | Memoria de la GPU | Sincronización |
|---|---|---|
| `D3D10CreateDeviceAndSwapChain` | El back buffer es de **DXGI** (`IDXGISwapChain`) y se obtiene como `ID3D10Texture2D` con `GetBuffer`. Ya no hay profundidad automática. | `Present` pasa a ser de DXGI. |
| `CreateTexture2D` (D24S8) + `CreateDepthStencilView` | La profundidad (3 686 400 B) se reserva **a mano**, y se accede por una **vista**. | Ninguna. |
| `CreateBuffer` con datos iniciales (`USAGE_DEFAULT`) | Una sola copia en la GPU. **No hay copia de respaldo en RAM** como en `MANAGED`, y no hay `Lock`. | Los datos se copian al crear. No hay "device lost" que los borre. |
| Constantes: `CreateBuffer(CONSTANT_BUFFER, 144 B)` + `UpdateSubresource` | Aparece el **constant buffer**: un objeto de memoria de 144 B en lugar de registros sueltos. El mismo buffer se ata al VS y al PS. | `UpdateSubresource` copia los 144 B en el runtime y programa la copia a la GPU. La CPU no espera a que la GPU suelte la versión anterior. |
| `CreateInputLayout` | Igual que la declaración de D3D9, pero **validada contra la firma del VS** al crearla. | Ninguna. |
| Sin `BeginScene` / `EndScene` | — | Desaparece esa posible frontera de envío. El driver sigue decidiendo cuándo enviar. |
| Sin `TestCooperativeLevel` / `Reset` | La memoria ya no "se pierde" en uso normal. Existe `DXGI_ERROR_DEVICE_REMOVED` para fallos graves (no se maneja aquí). | — |
| Centros de píxel en +0.5 | — | — (cambia *qué* píxeles se cubren: ver README, 536–732 píxeles de diferencia en D3D9) |
| Captura: `CopyResource` → textura `STAGING` + `Map(READ)` | La GPU copia 3 686 400 B a una textura que la CPU puede leer. | `Map(READ)` **bloquea la CPU** hasta que la copia terminó. La espera es implícita, como en `GetRenderTargetData`. |

## Direct3D 11: qué cambia respecto a D3D10

| Cambio | Memoria de la GPU | Sincronización |
|---|---|---|
| `D3D11CreateDeviceAndSwapChain` devuelve **device + immediate context** | Igual que D3D10 para este cubo. | `ID3D11Device` (crear recursos) se puede usar desde varios hilos. `ID3D11DeviceContext` (dibujar, `UpdateSubresource`, `CopyResource`, `Map`) es de un solo hilo. |
| Feature levels `[11_0, 10_1, 10_0]` | — | El runtime elige el nivel más alto que soporta el hardware. En esta máquina, 11.0 (lo imprime al arrancar). |
| `Map` / `Unmap` pasan al **context** (`ctx.Map(recurso)`) | Igual. | Igual: `Map(READ)` sobre `STAGING` bloquea la CPU hasta terminar la copia. |
| `VSSetShader` / `PSSetShader` reciben *class instances* | — | — (no se usan) |

## D3D11 → D3D12: el salto

Lo que en D3D9/10/11 hacía el driver y en D3D12 hace la app:

| Antes (driver) | D3D12 (la app) |
|---|---|
| Decidir cuándo enviar comandos a la GPU | `ExecuteCommandLists` |
| No sobrescribir memoria que la GPU aún lee (`UpdateSubresource`, *renaming*) | Ranuras de constantes por fotograma + `Signal` y espera por fence |
| Transiciones de estado de los recursos | `ResourceBarrier` |
| Reservar memoria y elegir dónde vive | `CreateCommittedResource` con heap `DEFAULT` / `UPLOAD` / `READBACK` |
| Guardar las vistas | Descriptor heaps (RTV 32 B, DSV 8 B en esta GPU) |
| Esperar en `Map(READ)` | Esperar el fence y después `Map` del READBACK |
