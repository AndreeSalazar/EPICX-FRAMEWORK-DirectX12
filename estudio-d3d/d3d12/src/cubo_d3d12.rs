//! # Direct3D 12 (2015, Windows 10)
//!
//! D3D12 deja de hacer cosas "por ti". Lo que D3D9/10/11 hacían dentro del driver
//! ahora lo hace la aplicación:
//! - **Command lists + allocators + queue**: los comandos se graban en una lista y se
//!   envían a una cola. Nada se ejecuta hasta `ExecuteCommandLists`.
//! - **Fences**: la CPU debe esperar a la GPU antes de reutilizar memoria o allocators.
//!   Aquí hay 2 frames en vuelo (`FRAMES`), cada uno con su allocator y su constant buffer.
//! - **Descriptor heaps**: las "vistas" (RTV, DSV) son descriptores en un heap que tú manejas.
//! - **Resource barriers**: tú declaras cada cambio de estado (PRESENT ↔ RENDER_TARGET).
//! - **Root signature**: define qué recursos ve cada shader (aquí: 1 root CBV en b0).
//! - **Pipeline State Object (PSO)**: shaders + input layout + rasterizer + blend + depth,
//!   todo compilado en UN objeto inmutable (en D3D11 eran estados sueltos).
//! - **Memoria explícita**: tú eliges el heap (UPLOAD, DEFAULT, READBACK). Por simplicidad,
//!   los vértices e índices viven en un heap UPLOAD (en un motor real se copian a DEFAULT).
//! - Swap chain obligatoriamente "flip model" (FLIP_DISCARD) con 2+ buffers. Sin VSync hace falta
//!   `ALLOW_TEARING`: si no, el compositor (DWM) retiene el buffer mostrado hasta el siguiente
//!   refresco y `Present(0)` queda atado al monitor aunque no haya VSync.
//!
//! Al crearse imprime cuánta memoria pide cada recurso y cuánta asigna el driver
//! (`GetResourceAllocationInfo`): esas son las cifras de DICCIONARIO.md.

use std::mem::{size_of, transmute_copy, ManuallyDrop};

use estudio_base::{compilar_hlsl, cubo_neutro, error, rgba_a_argb, Config, Constantes, Renderizador, Vertice};
use windows::core::{s, Interface, Result};
use windows::Win32::Foundation::{CloseHandle, BOOL, FALSE, HANDLE, HWND, RECT, TRUE};
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject, INFINITE};

const HLSL: &str = include_str!("cubo.hlsl");

/// Frames en vuelo = número de back buffers.
const FRAMES: usize = 2;
/// Cada constant buffer debe empezar en múltiplo de 256 bytes
/// (D3D12_CONSTANT_BUFFER_DATA_PLACEMENT_ALIGNMENT).
const TAM_CB: usize = 256;

/// Memoria de lectura para `--fotograma`: el back buffer se copia aquí.
struct Readback {
    buffer: ID3D12Resource,
    huella: D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
}

pub struct CuboD3d12 {
    dispositivo: ID3D12Device,
    cola: ID3D12CommandQueue,
    swap: IDXGISwapChain3,
    heap_rtv: ID3D12DescriptorHeap,
    tam_rtv: usize,
    heap_dsv: ID3D12DescriptorHeap,
    render_targets: Vec<ID3D12Resource>,
    _profundidad: ID3D12Resource,
    asignadores: Vec<ID3D12CommandAllocator>,
    lista: ID3D12GraphicsCommandList,
    firma_raiz: ID3D12RootSignature,
    pso: ID3D12PipelineState,
    _vb: ID3D12Resource,
    vbv: D3D12_VERTEX_BUFFER_VIEW,
    _ib: ID3D12Resource,
    ibv: D3D12_INDEX_BUFFER_VIEW,
    cb: ID3D12Resource,
    cb_ptr: *mut u8,
    readback: Option<Readback>,
    fence: ID3D12Fence,
    valores_fence: [u64; FRAMES],
    ultimo_fence: u64,
    evento: HANDLE,
    viewport: D3D12_VIEWPORT,
    scissor: RECT,
    config: Config,
    /// `Present(0, ALLOW_TEARING)` cuando no hay VSync y el sistema lo soporta.
    tearing: bool,
}

/// Imprime bytes pedidos y bytes que el driver reserva para `desc`.
fn informar(d: &ID3D12Device, nombre: &str, desc: &D3D12_RESOURCE_DESC, pedidos: u64) {
    let info = unsafe { d.GetResourceAllocationInfo(0, &[*desc]) };
    println!(
        "[memoria] {nombre:<28} pedidos {pedidos:>9} B  asignados {:>9} B  alineación {:>7} B",
        info.SizeInBytes, info.Alignment
    );
}

impl CuboD3d12 {
    pub fn nuevo(hwnd: HWND, config: &Config) -> Result<Self> {
        let (ancho, alto) = (config.ancho, config.alto);
        unsafe {
            // 1) Fábrica DXGI + adaptador de alto rendimiento + device
            let fabrica: IDXGIFactory6 = CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0))?;
            let adaptador: IDXGIAdapter1 = fabrica.EnumAdapterByGpuPreference(0, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE)?;
            let desc = adaptador.GetDesc1()?;
            let fin = desc.Description.iter().position(|&c| c == 0).unwrap_or(desc.Description.len());
            println!("[Estudio D3D12] GPU: {}", String::from_utf16_lossy(&desc.Description[..fin]));

            let mut dispositivo: Option<ID3D12Device> = None;
            D3D12CreateDevice(&adaptador, D3D_FEATURE_LEVEL_11_0, &mut dispositivo)?;
            let dispositivo = dispositivo.unwrap();

            // 2) Cola de comandos (el swap chain se crea sobre la cola, no sobre el device)
            let cola: ID3D12CommandQueue = dispositivo.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                ..Default::default()
            })?;

            // 3) Swap chain flip model. Sin VSync se pide ALLOW_TEARING si el sistema lo soporta.
            let mut soporta_tearing = FALSE;
            let tearing = !config.vsync
                && fabrica
                    .CheckFeatureSupport(DXGI_FEATURE_PRESENT_ALLOW_TEARING, &mut soporta_tearing as *mut _ as _, size_of::<BOOL>() as u32)
                    .is_ok()
                && soporta_tearing.as_bool();
            println!("[Estudio D3D12] tearing (VSync apagado de verdad en flip model): {}", if tearing { "sí" } else { "no" });
            let desc_swap = DXGI_SWAP_CHAIN_DESC1 {
                Width: ancho,
                Height: alto,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 }, // sin MSAA
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: FRAMES as u32,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                Flags: if tearing { DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING.0 as u32 } else { 0 },
                ..Default::default()
            };
            let swap: IDXGISwapChain3 = fabrica.CreateSwapChainForHwnd(&cola, hwnd, &desc_swap, None, None)?.cast()?;
            fabrica.MakeWindowAssociation(hwnd, DXGI_MWA_NO_ALT_ENTER)?;

            // 4) Descriptor heaps: RTV (uno por back buffer) y DSV
            let heap_rtv: ID3D12DescriptorHeap = dispositivo.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                NumDescriptors: FRAMES as u32,
                ..Default::default()
            })?;
            let tam_rtv = dispositivo.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) as usize;
            let heap_dsv: ID3D12DescriptorHeap = dispositivo.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
                NumDescriptors: 1,
                ..Default::default()
            })?;
            println!(
                "[memoria] descriptor RTV {tam_rtv} B, DSV {} B (GetDescriptorHandleIncrementSize)",
                dispositivo.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_DSV)
            );

            let inicio_rtv = heap_rtv.GetCPUDescriptorHandleForHeapStart();
            let mut render_targets = Vec::with_capacity(FRAMES);
            for i in 0..FRAMES {
                let rt: ID3D12Resource = swap.GetBuffer(i as u32)?;
                dispositivo.CreateRenderTargetView(&rt, None, D3D12_CPU_DESCRIPTOR_HANDLE { ptr: inicio_rtv.ptr + i * tam_rtv });
                informar(&dispositivo, &format!("back buffer {i} RGBA8"), &rt.GetDesc(), ancho as u64 * alto as u64 * 4);
                render_targets.push(rt);
            }

            // 5) Depth buffer: recurso en heap DEFAULT (memoria de la GPU) + DSV
            let desc_prof = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Width: ancho as u64,
                Height: alto,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: DXGI_FORMAT_D32_FLOAT,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Flags: D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
                ..Default::default()
            };
            informar(&dispositivo, "depth D32 (DEFAULT)", &desc_prof, ancho as u64 * alto as u64 * 4);
            let limpiar = D3D12_CLEAR_VALUE {
                Format: DXGI_FORMAT_D32_FLOAT,
                Anonymous: D3D12_CLEAR_VALUE_0 { DepthStencil: D3D12_DEPTH_STENCIL_VALUE { Depth: 1.0, Stencil: 0 } },
            };
            let mut profundidad: Option<ID3D12Resource> = None;
            dispositivo.CreateCommittedResource(
                &D3D12_HEAP_PROPERTIES { Type: D3D12_HEAP_TYPE_DEFAULT, ..Default::default() },
                D3D12_HEAP_FLAG_NONE,
                &desc_prof,
                D3D12_RESOURCE_STATE_DEPTH_WRITE,
                Some(&limpiar),
                &mut profundidad,
            )?;
            let profundidad = profundidad.unwrap();
            dispositivo.CreateDepthStencilView(&profundidad, None, heap_dsv.GetCPUDescriptorHandleForHeapStart());

            // 6) Root signature: un solo parámetro = CBV en b0, visible para VS y PS
            let parametro = D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_CBV,
                Anonymous: D3D12_ROOT_PARAMETER_0 { Descriptor: D3D12_ROOT_DESCRIPTOR { ShaderRegister: 0, RegisterSpace: 0 } },
                ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
            };
            let desc_firma = D3D12_ROOT_SIGNATURE_DESC {
                NumParameters: 1,
                pParameters: &parametro,
                Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
                ..Default::default()
            };
            let mut blob_firma = None;
            D3D12SerializeRootSignature(&desc_firma, D3D_ROOT_SIGNATURE_VERSION_1, &mut blob_firma, None)?;
            let firma_raiz: ID3D12RootSignature = dispositivo.CreateRootSignature(0, estudio_base::bytes_blob(&blob_firma.unwrap()))?;

            // 7) PSO: shaders + input layout + estados, todo en un objeto
            let codigo_vs = compilar_hlsl(HLSL, s!("VSMain"), s!("vs_5_0"))?;
            let codigo_ps = compilar_hlsl(HLSL, s!("PSMain"), s!("ps_5_0"))?;
            let elemento = |semantica, formato, offset| D3D12_INPUT_ELEMENT_DESC {
                SemanticName: semantica,
                SemanticIndex: 0,
                Format: formato,
                InputSlot: 0,
                AlignedByteOffset: offset,
                InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            };
            let elementos = [
                elemento(s!("POSITION"), DXGI_FORMAT_R32G32B32_FLOAT, 0),
                elemento(s!("NORMAL"), DXGI_FORMAT_R32G32B32_FLOAT, 12),
                elemento(s!("COLOR"), DXGI_FORMAT_R32G32B32A32_FLOAT, 24),
            ];
            let mut desc_pso = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
                // Referencia prestada (sin AddRef): la root signature vive en `Self`
                pRootSignature: transmute_copy(&firma_raiz),
                VS: D3D12_SHADER_BYTECODE { pShaderBytecode: codigo_vs.as_ptr() as _, BytecodeLength: codigo_vs.len() },
                PS: D3D12_SHADER_BYTECODE { pShaderBytecode: codigo_ps.as_ptr() as _, BytecodeLength: codigo_ps.len() },
                BlendState: D3D12_BLEND_DESC {
                    RenderTarget: [D3D12_RENDER_TARGET_BLEND_DESC {
                        SrcBlend: D3D12_BLEND_ONE,
                        DestBlend: D3D12_BLEND_ZERO,
                        BlendOp: D3D12_BLEND_OP_ADD,
                        SrcBlendAlpha: D3D12_BLEND_ONE,
                        DestBlendAlpha: D3D12_BLEND_ZERO,
                        BlendOpAlpha: D3D12_BLEND_OP_ADD,
                        LogicOp: D3D12_LOGIC_OP_NOOP,
                        RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
                        ..Default::default()
                    }; 8],
                    ..Default::default()
                },
                SampleMask: u32::MAX,
                RasterizerState: D3D12_RASTERIZER_DESC {
                    FillMode: D3D12_FILL_MODE_SOLID,
                    CullMode: D3D12_CULL_MODE_BACK,
                    DepthClipEnable: TRUE,
                    ..Default::default() // MultisampleEnable/AntialiasedLineEnable = FALSE
                },
                DepthStencilState: D3D12_DEPTH_STENCIL_DESC {
                    DepthEnable: TRUE,
                    DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
                    DepthFunc: D3D12_COMPARISON_FUNC_LESS,
                    ..Default::default()
                },
                InputLayout: D3D12_INPUT_LAYOUT_DESC { pInputElementDescs: elementos.as_ptr(), NumElements: elementos.len() as u32 },
                PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
                NumRenderTargets: 1,
                DSVFormat: DXGI_FORMAT_D32_FLOAT,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                ..Default::default()
            };
            desc_pso.RTVFormats[0] = DXGI_FORMAT_R8G8B8A8_UNORM;
            let pso: ID3D12PipelineState = dispositivo.CreateGraphicsPipelineState(&desc_pso)?;

            // 8) Vertex/index buffers (heap UPLOAD) + sus "views", malla de cubo-neutro
            let vertices = cubo_neutro::vertices();
            let indices = cubo_neutro::indices();
            let bytes_vb = size_of_val(&vertices);
            let bytes_ib = size_of_val(&indices);
            let vb = buffer(&dispositivo, "vertex buffer (UPLOAD)", bytes_vb, D3D12_HEAP_TYPE_UPLOAD)?;
            let ib = buffer(&dispositivo, "index buffer (UPLOAD)", bytes_ib, D3D12_HEAP_TYPE_UPLOAD)?;
            copiar(&vb, &vertices)?;
            copiar(&ib, &indices)?;
            let vbv = D3D12_VERTEX_BUFFER_VIEW {
                BufferLocation: vb.GetGPUVirtualAddress(),
                SizeInBytes: bytes_vb as u32,
                StrideInBytes: size_of::<Vertice>() as u32,
            };
            let ibv = D3D12_INDEX_BUFFER_VIEW {
                BufferLocation: ib.GetGPUVirtualAddress(),
                SizeInBytes: bytes_ib as u32,
                Format: DXGI_FORMAT_R16_UINT,
            };

            // 9) Constant buffer: una ranura de 256 bytes por frame, mapeado para siempre
            let cb = buffer(&dispositivo, "constant buffer (UPLOAD)", TAM_CB * FRAMES, D3D12_HEAP_TYPE_UPLOAD)?;
            let mut cb_ptr = std::ptr::null_mut();
            cb.Map(0, Some(&D3D12_RANGE { Begin: 0, End: 0 }), Some(&mut cb_ptr))?;

            // 10) Allocators (uno por frame) + command list + fence
            let mut asignadores = Vec::with_capacity(FRAMES);
            for _ in 0..FRAMES {
                asignadores.push(dispositivo.CreateCommandAllocator::<ID3D12CommandAllocator>(D3D12_COMMAND_LIST_TYPE_DIRECT)?);
            }
            let lista: ID3D12GraphicsCommandList = dispositivo.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &asignadores[0], &pso)?;
            lista.Close()?; // se crea abierta; la cerramos y se reabre con Reset en cada frame

            let fence: ID3D12Fence = dispositivo.CreateFence(0, D3D12_FENCE_FLAG_NONE)?;
            let evento = CreateEventW(None, FALSE, FALSE, None)?;

            Ok(Self {
                dispositivo,
                cola,
                swap,
                heap_rtv,
                tam_rtv,
                heap_dsv,
                render_targets,
                _profundidad: profundidad,
                asignadores,
                lista,
                firma_raiz,
                pso,
                _vb: vb,
                vbv,
                _ib: ib,
                ibv,
                cb,
                cb_ptr: cb_ptr as *mut u8,
                readback: None,
                fence,
                valores_fence: [0; FRAMES],
                ultimo_fence: 0,
                evento,
                viewport: D3D12_VIEWPORT { TopLeftX: 0.0, TopLeftY: 0.0, Width: ancho as f32, Height: alto as f32, MinDepth: 0.0, MaxDepth: 1.0 },
                scissor: RECT { left: 0, top: 0, right: ancho as i32, bottom: alto as i32 },
                config: *config,
                tearing,
            })
        }
    }

    /// Bloquea la CPU hasta que la GPU haya pasado el valor `valor` del fence.
    fn esperar_gpu(&self, valor: u64) -> Result<()> {
        unsafe {
            if self.fence.GetCompletedValue() < valor {
                self.fence.SetEventOnCompletion(valor, self.evento)?;
                WaitForSingleObject(self.evento, INFINITE);
            }
        }
        Ok(())
    }

    /// Buffer READBACK con la huella (row pitch alineado a 256) que pide `CopyTextureRegion`.
    fn preparar_readback(&mut self) -> Result<()> {
        if self.readback.is_some() {
            return Ok(());
        }
        unsafe {
            let desc_rt = self.render_targets[0].GetDesc();
            let mut huella = D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default();
            let mut total = 0u64;
            self.dispositivo.GetCopyableFootprints(&desc_rt, 0, 1, 0, Some(&mut huella), None, None, Some(&mut total));
            let buffer = buffer(&self.dispositivo, "readback (READBACK)", total as usize, D3D12_HEAP_TYPE_READBACK)?;
            self.readback = Some(Readback { buffer, huella });
        }
        Ok(())
    }
}

fn buffer(d: &ID3D12Device, nombre: &str, bytes: usize, tipo: D3D12_HEAP_TYPE) -> Result<ID3D12Resource> {
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
        Width: bytes as u64,
        Height: 1,
        DepthOrArraySize: 1,
        MipLevels: 1,
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
        ..Default::default()
    };
    informar(d, nombre, &desc, bytes as u64);
    // UPLOAD: la CPU escribe y la GPU lee → GENERIC_READ. READBACK: la GPU escribe → COPY_DEST.
    let estado = if tipo == D3D12_HEAP_TYPE_READBACK { D3D12_RESOURCE_STATE_COPY_DEST } else { D3D12_RESOURCE_STATE_GENERIC_READ };
    let mut recurso = None;
    unsafe {
        d.CreateCommittedResource(&D3D12_HEAP_PROPERTIES { Type: tipo, ..Default::default() }, D3D12_HEAP_FLAG_NONE, &desc, estado, None, &mut recurso)?;
    }
    recurso.ok_or_else(|| error("CreateCommittedResource no devolvió recurso"))
}

fn copiar<T: Copy>(recurso: &ID3D12Resource, datos: &[T]) -> Result<()> {
    unsafe {
        let mut p = std::ptr::null_mut();
        recurso.Map(0, Some(&D3D12_RANGE { Begin: 0, End: 0 }), Some(&mut p))?;
        std::ptr::copy_nonoverlapping(datos.as_ptr(), p as *mut T, datos.len());
        recurso.Unmap(0, None);
    }
    Ok(())
}

fn transicion(recurso: &ID3D12Resource, antes: D3D12_RESOURCE_STATES, despues: D3D12_RESOURCE_STATES) -> D3D12_RESOURCE_BARRIER {
    D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                // Referencia prestada (sin AddRef/Release), igual que en los samples de windows-rs
                pResource: unsafe { transmute_copy(recurso) },
                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                StateBefore: antes,
                StateAfter: despues,
            }),
        },
    }
}

impl Renderizador for CuboD3d12 {
    fn dibujar(&mut self, c: &Constantes, captura: Option<&mut [u32]>) -> Result<()> {
        if captura.is_some() {
            self.preparar_readback()?;
        }
        unsafe {
            let i = self.swap.GetCurrentBackBufferIndex() as usize;
            let rt = &self.render_targets[i];

            // 1) Esperar a que la GPU termine el frame que usó esta ranura la vez anterior
            self.esperar_gpu(self.valores_fence[i])?;

            // 2) Escribir las constantes en la ranura de este frame
            std::ptr::copy_nonoverlapping(c as *const Constantes as *const u8, self.cb_ptr.add(i * TAM_CB), size_of::<Constantes>());

            // 3) Grabar la command list
            let asignador = &self.asignadores[i];
            asignador.Reset()?;
            let l = &self.lista;
            l.Reset(asignador, &self.pso)?;

            l.SetGraphicsRootSignature(&self.firma_raiz);
            l.SetGraphicsRootConstantBufferView(0, self.cb.GetGPUVirtualAddress() + (i * TAM_CB) as u64);
            l.RSSetViewports(&[self.viewport]);
            l.RSSetScissorRects(&[self.scissor]);

            l.ResourceBarrier(&[transicion(rt, D3D12_RESOURCE_STATE_PRESENT, D3D12_RESOURCE_STATE_RENDER_TARGET)]);

            let rtv = D3D12_CPU_DESCRIPTOR_HANDLE { ptr: self.heap_rtv.GetCPUDescriptorHandleForHeapStart().ptr + i * self.tam_rtv };
            let dsv = self.heap_dsv.GetCPUDescriptorHandleForHeapStart();
            l.OMSetRenderTargets(1, Some(&rtv), FALSE, Some(&dsv));
            l.ClearRenderTargetView(rtv, &cubo_neutro::FONDO_F, None);
            l.ClearDepthStencilView(dsv, D3D12_CLEAR_FLAG_DEPTH, 1.0, 0, &[]);

            l.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            l.IASetVertexBuffers(0, Some(&[self.vbv]));
            l.IASetIndexBuffer(Some(&self.ibv));
            l.DrawIndexedInstanced(cubo_neutro::NUM_INDICES as u32, 1, 0, 0, 0);

            if let Some(rb) = &self.readback {
                // Captura: RENDER_TARGET → COPY_SOURCE, copiar al buffer READBACK, → PRESENT
                l.ResourceBarrier(&[transicion(rt, D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_COPY_SOURCE)]);
                let destino = D3D12_TEXTURE_COPY_LOCATION {
                    pResource: transmute_copy(&rb.buffer),
                    Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                    Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { PlacedFootprint: rb.huella },
                };
                let origen = D3D12_TEXTURE_COPY_LOCATION {
                    pResource: transmute_copy(rt),
                    Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                    Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
                };
                l.CopyTextureRegion(&destino, 0, 0, 0, &origen, None);
                l.ResourceBarrier(&[transicion(rt, D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_PRESENT)]);
            } else {
                l.ResourceBarrier(&[transicion(rt, D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_PRESENT)]);
            }
            l.Close()?;

            // 4) Enviar a la GPU, presentar y marcar el fence de este frame
            self.cola.ExecuteCommandLists(&[Some(l.cast()?)]);
            let banderas = if self.tearing { DXGI_PRESENT_ALLOW_TEARING } else { DXGI_PRESENT(0) };
            self.swap.Present(self.config.vsync as u32, banderas).ok()?;

            self.ultimo_fence += 1;
            self.cola.Signal(&self.fence, self.ultimo_fence)?;
            self.valores_fence[i] = self.ultimo_fence;

            // 5) Captura: esperar a que la copia termine y leer el buffer READBACK
            if let (Some(destino), Some(rb)) = (captura, &self.readback) {
                self.esperar_gpu(self.ultimo_fence)?;
                let mut p = std::ptr::null_mut();
                rb.buffer.Map(0, None, Some(&mut p))?;
                let pitch = rb.huella.Footprint.RowPitch as usize;
                estudio_base::copiar_filas(p as *const u8, pitch, self.config.ancho, self.config.alto, destino, rgba_a_argb);
                rb.buffer.Unmap(0, Some(&D3D12_RANGE { Begin: 0, End: 0 }));
            }
        }
        Ok(())
    }
}

impl Drop for CuboD3d12 {
    fn drop(&mut self) {
        // No liberar recursos mientras la GPU todavía los está usando
        let _ = self.esperar_gpu(self.ultimo_fence);
        unsafe {
            self.cb.Unmap(0, None);
            let _ = CloseHandle(self.evento);
        }
    }
}
