//! # BMOX-12 → Direct3D 12 (2015, Windows 10)
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
//! - **Memoria explícita**: tú eliges el heap (UPLOAD, DEFAULT…). Por simplicidad, los
//!   vértices e índices viven en un heap UPLOAD (lo típico en un primer ejemplo; en un
//!   motor real se copian a un heap DEFAULT).
//! - Swap chain obligatoriamente "flip model" (FLIP_DISCARD) con 2+ buffers.

use std::mem::{size_of, transmute_copy, ManuallyDrop};

use bmox_base::{compilar_hlsl, cubo, error, BmoxRender, Constantes, Vertice, COLOR_FONDO, VSYNC};
use windows::core::{s, Interface, Result};
use windows::Win32::Foundation::{CloseHandle, FALSE, HANDLE, HWND, RECT, TRUE};
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject, INFINITE};

const HLSL: &str = include_str!("cubo.hlsl");

/// Frames en vuelo = número de back buffers.
const FRAMES: usize = 2;
/// Los constant buffers en D3D12 deben estar alineados a 256 bytes.
const TAM_CB: usize = 256;

pub struct Bmox12 {
    _dispositivo: ID3D12Device,
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
    fence: ID3D12Fence,
    valores_fence: [u64; FRAMES],
    ultimo_fence: u64,
    evento: HANDLE,
    viewport: D3D12_VIEWPORT,
    scissor: RECT,
    num_indices: u32,
}

impl Bmox12 {
    pub fn nuevo(hwnd: HWND, ancho: u32, alto: u32) -> Result<Self> {
        unsafe {
            // 1) Fábrica DXGI + adaptador de alto rendimiento + device
            let fabrica: IDXGIFactory6 = CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0))?;
            let adaptador: IDXGIAdapter1 = fabrica.EnumAdapterByGpuPreference(0, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE)?;
            let desc = adaptador.GetDesc1()?;
            let fin = desc.Description.iter().position(|&c| c == 0).unwrap_or(desc.Description.len());
            println!("[BMOX-12] GPU: {}", String::from_utf16_lossy(&desc.Description[..fin]));

            let mut dispositivo: Option<ID3D12Device> = None;
            D3D12CreateDevice(&adaptador, D3D_FEATURE_LEVEL_11_0, &mut dispositivo)?;
            let dispositivo = dispositivo.unwrap();

            // 2) Cola de comandos (el swap chain se crea sobre la cola, no sobre el device)
            let cola: ID3D12CommandQueue = dispositivo.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                ..Default::default()
            })?;

            // 3) Swap chain flip model
            let desc_swap = DXGI_SWAP_CHAIN_DESC1 {
                Width: ancho,
                Height: alto,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: FRAMES as u32,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
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

            let inicio_rtv = heap_rtv.GetCPUDescriptorHandleForHeapStart();
            let mut render_targets = Vec::with_capacity(FRAMES);
            for i in 0..FRAMES {
                let rt: ID3D12Resource = swap.GetBuffer(i as u32)?;
                dispositivo.CreateRenderTargetView(&rt, None, D3D12_CPU_DESCRIPTOR_HANDLE { ptr: inicio_rtv.ptr + i * tam_rtv });
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

            // 6) Root signature: un solo parámetro = CBV en b0, visible para el vertex shader
            let parametro = D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_CBV,
                Anonymous: D3D12_ROOT_PARAMETER_0 { Descriptor: D3D12_ROOT_DESCRIPTOR { ShaderRegister: 0, RegisterSpace: 0 } },
                ShaderVisibility: D3D12_SHADER_VISIBILITY_VERTEX,
            };
            let desc_firma = D3D12_ROOT_SIGNATURE_DESC {
                NumParameters: 1,
                pParameters: &parametro,
                Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
                ..Default::default()
            };
            let mut blob_firma = None;
            D3D12SerializeRootSignature(&desc_firma, D3D_ROOT_SIGNATURE_VERSION_1, &mut blob_firma, None)?;
            let firma_raiz: ID3D12RootSignature = dispositivo.CreateRootSignature(0, bmox_base::bytes_blob(&blob_firma.unwrap()))?;

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
                    ..Default::default()
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

            // 8) Vertex/index buffers (heap UPLOAD) + sus "views"
            let (vertices, indices) = cubo();
            let bytes_vb = size_of::<Vertice>() * vertices.len();
            let bytes_ib = size_of::<u16>() * indices.len();
            let vb = buffer_subida(&dispositivo, bytes_vb)?;
            let ib = buffer_subida(&dispositivo, bytes_ib)?;
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
            let cb = buffer_subida(&dispositivo, TAM_CB * FRAMES)?;
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
                _dispositivo: dispositivo,
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
                fence,
                valores_fence: [0; FRAMES],
                ultimo_fence: 0,
                evento,
                viewport: D3D12_VIEWPORT { TopLeftX: 0.0, TopLeftY: 0.0, Width: ancho as f32, Height: alto as f32, MinDepth: 0.0, MaxDepth: 1.0 },
                scissor: RECT { left: 0, top: 0, right: ancho as i32, bottom: alto as i32 },
                num_indices: indices.len() as u32,
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
}

fn buffer_subida(d: &ID3D12Device, bytes: usize) -> Result<ID3D12Resource> {
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
    let mut recurso = None;
    unsafe {
        d.CreateCommittedResource(
            &D3D12_HEAP_PROPERTIES { Type: D3D12_HEAP_TYPE_UPLOAD, ..Default::default() },
            D3D12_HEAP_FLAG_NONE,
            &desc,
            D3D12_RESOURCE_STATE_GENERIC_READ,
            None,
            &mut recurso,
        )?;
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

impl BmoxRender for Bmox12 {
    fn dibujar(&mut self, c: &Constantes) -> Result<()> {
        unsafe {
            let i = self.swap.GetCurrentBackBufferIndex() as usize;

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

            l.ResourceBarrier(&[transicion(&self.render_targets[i], D3D12_RESOURCE_STATE_PRESENT, D3D12_RESOURCE_STATE_RENDER_TARGET)]);

            let rtv = D3D12_CPU_DESCRIPTOR_HANDLE { ptr: self.heap_rtv.GetCPUDescriptorHandleForHeapStart().ptr + i * self.tam_rtv };
            let dsv = self.heap_dsv.GetCPUDescriptorHandleForHeapStart();
            l.OMSetRenderTargets(1, Some(&rtv), FALSE, Some(&dsv));
            l.ClearRenderTargetView(rtv, &COLOR_FONDO, None);
            l.ClearDepthStencilView(dsv, D3D12_CLEAR_FLAG_DEPTH, 1.0, 0, &[]);

            l.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            l.IASetVertexBuffers(0, Some(&[self.vbv]));
            l.IASetIndexBuffer(Some(&self.ibv));
            l.DrawIndexedInstanced(self.num_indices, 1, 0, 0, 0);

            l.ResourceBarrier(&[transicion(&self.render_targets[i], D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_PRESENT)]);
            l.Close()?;

            // 4) Enviar a la GPU, presentar y marcar el fence de este frame
            self.cola.ExecuteCommandLists(&[Some(l.cast()?)]);
            self.swap.Present(if VSYNC { 1 } else { 0 }, DXGI_PRESENT(0)).ok()?;

            self.ultimo_fence += 1;
            self.cola.Signal(&self.fence, self.ultimo_fence)?;
            self.valores_fence[i] = self.ultimo_fence;
        }
        Ok(())
    }
}

impl Drop for Bmox12 {
    fn drop(&mut self) {
        // No liberar recursos mientras la GPU todavía los está usando
        let _ = self.esperar_gpu(self.ultimo_fence);
        unsafe {
            self.cb.Unmap(0, None);
            let _ = CloseHandle(self.evento);
        }
    }
}
