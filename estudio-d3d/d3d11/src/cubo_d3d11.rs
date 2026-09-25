//! # Direct3D 11 (2009, Windows 7)
//!
//! Qué cambia frente a D3D10:
//! - El device se divide en dos objetos:
//!   - `ID3D11Device`: solo CREA recursos (thread-safe, se puede usar desde varios hilos).
//!   - `ID3D11DeviceContext` ("immediate context"): solo DIBUJA y cambia estado.
//!   También existen "deferred contexts" para grabar comandos en otros hilos.
//! - **Feature levels**: se pide una lista (11_0, 10_1, 10_0…) y el driver elige la
//!   mejor que soporte, así la misma API corre en hardware más viejo.
//! - Compute shaders, teselación (hull/domain) y UAVs (no se usan en este cubo).
//! - El resto (DXGI, vistas, input layout, constant buffers) es igual que en D3D10.

use std::mem::size_of;

use estudio_base::{compilar_hlsl, cubo_neutro, rgba_a_argb, Config, Constantes, Renderizador, Vertice};
use windows::core::{s, Result};
use windows::Win32::Foundation::{HMODULE, HWND, TRUE};
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;

const HLSL: &str = include_str!("cubo.hlsl");

pub struct CuboD3d11 {
    dispositivo: ID3D11Device,
    contexto: ID3D11DeviceContext,
    swap: IDXGISwapChain,
    back_buffer: ID3D11Texture2D,
    rtv: ID3D11RenderTargetView,
    dsv: ID3D11DepthStencilView,
    layout: ID3D11InputLayout,
    vs: ID3D11VertexShader,
    ps: ID3D11PixelShader,
    vb: ID3D11Buffer,
    ib: ID3D11Buffer,
    cb: ID3D11Buffer,
    viewport: D3D11_VIEWPORT,
    config: Config,
}

impl CuboD3d11 {
    pub fn nuevo(hwnd: HWND, config: &Config) -> Result<Self> {
        let (ancho, alto) = (config.ancho, config.alto);
        unsafe {
            // 1) Device + immediate context + swap chain (mismo DXGI_SWAP_CHAIN_DESC que D3D10)
            let desc_swap = DXGI_SWAP_CHAIN_DESC {
                BufferDesc: DXGI_MODE_DESC { Width: ancho, Height: alto, Format: DXGI_FORMAT_R8G8B8A8_UNORM, ..Default::default() },
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 }, // sin MSAA
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 1,
                OutputWindow: hwnd,
                Windowed: TRUE,
                SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
                Flags: 0,
            };
            let niveles = [D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_10_1, D3D_FEATURE_LEVEL_10_0];
            let mut nivel = D3D_FEATURE_LEVEL::default();
            let (mut swap, mut dispositivo, mut contexto) = (None, None, None);
            D3D11CreateDeviceAndSwapChain(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_FLAG(0),
                Some(&niveles),
                D3D11_SDK_VERSION,
                Some(&desc_swap),
                Some(&mut swap),
                Some(&mut dispositivo),
                Some(&mut nivel),
                Some(&mut contexto),
            )?;
            let (swap, dispositivo, contexto): (IDXGISwapChain, ID3D11Device, ID3D11DeviceContext) =
                (swap.unwrap(), dispositivo.unwrap(), contexto.unwrap());
            println!("[Estudio D3D11] feature level: {}.{}", nivel.0 >> 12, (nivel.0 >> 8) & 0xF);

            // 2) Render target view sobre el back buffer
            let back_buffer: ID3D11Texture2D = swap.GetBuffer(0)?;
            let mut rtv = None;
            dispositivo.CreateRenderTargetView(&back_buffer, None, Some(&mut rtv))?;

            // 3) Depth buffer: textura D24S8 + depth stencil view
            let desc_prof = D3D11_TEXTURE2D_DESC {
                Width: ancho,
                Height: alto,
                MipLevels: 1,
                ArraySize: 1,
                Format: DXGI_FORMAT_D24_UNORM_S8_UINT,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_DEPTH_STENCIL.0 as u32,
                CPUAccessFlags: 0,
                MiscFlags: 0,
            };
            let mut profundidad = None;
            dispositivo.CreateTexture2D(&desc_prof, None, Some(&mut profundidad))?;
            let mut dsv = None;
            dispositivo.CreateDepthStencilView(&profundidad.unwrap(), None, Some(&mut dsv))?;

            // 4) Shaders SM5 + input layout
            let codigo_vs = compilar_hlsl(HLSL, s!("VSMain"), s!("vs_5_0"))?;
            let codigo_ps = compilar_hlsl(HLSL, s!("PSMain"), s!("ps_5_0"))?;
            let (mut vs, mut ps) = (None, None);
            dispositivo.CreateVertexShader(&codigo_vs, None, Some(&mut vs))?;
            dispositivo.CreatePixelShader(&codigo_ps, None, Some(&mut ps))?;

            let elemento = |semantica, formato, offset| D3D11_INPUT_ELEMENT_DESC {
                SemanticName: semantica,
                SemanticIndex: 0,
                Format: formato,
                InputSlot: 0,
                AlignedByteOffset: offset,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            };
            let elementos = [
                elemento(s!("POSITION"), DXGI_FORMAT_R32G32B32_FLOAT, 0),
                elemento(s!("NORMAL"), DXGI_FORMAT_R32G32B32_FLOAT, 12),
                elemento(s!("COLOR"), DXGI_FORMAT_R32G32B32A32_FLOAT, 24),
            ];
            let mut layout = None;
            dispositivo.CreateInputLayout(&elementos, &codigo_vs, Some(&mut layout))?;

            // 5) Buffers: malla de cubo-neutro
            let vertices = cubo_neutro::vertices();
            let indices = cubo_neutro::indices();
            let vb = crear_buffer(&dispositivo, D3D11_BIND_VERTEX_BUFFER, size_of_val(&vertices), Some(vertices.as_ptr() as _))?;
            let ib = crear_buffer(&dispositivo, D3D11_BIND_INDEX_BUFFER, size_of_val(&indices), Some(indices.as_ptr() as _))?;
            let cb = crear_buffer(&dispositivo, D3D11_BIND_CONSTANT_BUFFER, size_of::<Constantes>(), None)?;

            let viewport = D3D11_VIEWPORT { TopLeftX: 0.0, TopLeftY: 0.0, Width: ancho as f32, Height: alto as f32, MinDepth: 0.0, MaxDepth: 1.0 };

            Ok(Self {
                dispositivo,
                contexto,
                swap,
                back_buffer,
                rtv: rtv.unwrap(),
                dsv: dsv.unwrap(),
                layout: layout.unwrap(),
                vs: vs.unwrap(),
                ps: ps.unwrap(),
                vb,
                ib,
                cb,
                viewport,
                config: *config,
            })
        }
    }

    /// Back buffer → textura STAGING → `destino`. El `Map` lo hace el context, no la textura.
    fn capturar(&self, destino: &mut [u32]) -> Result<()> {
        unsafe {
            let desc = D3D11_TEXTURE2D_DESC {
                Width: self.config.ancho,
                Height: self.config.alto,
                MipLevels: 1,
                ArraySize: 1,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Usage: D3D11_USAGE_STAGING,
                BindFlags: 0,
                CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                MiscFlags: 0,
            };
            let mut staging = None;
            self.dispositivo.CreateTexture2D(&desc, None, Some(&mut staging))?;
            let staging = staging.unwrap();
            let ctx = &self.contexto;
            ctx.CopyResource(&staging, &self.back_buffer);
            let mut m = D3D11_MAPPED_SUBRESOURCE::default();
            ctx.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut m))?; // bloquea hasta que la copia termina
            estudio_base::copiar_filas(m.pData as *const u8, m.RowPitch as usize, self.config.ancho, self.config.alto, destino, rgba_a_argb);
            ctx.Unmap(&staging, 0);
        }
        Ok(())
    }
}

fn crear_buffer(d: &ID3D11Device, uso: D3D11_BIND_FLAG, bytes: usize, datos: Option<*const std::ffi::c_void>) -> Result<ID3D11Buffer> {
    let desc = D3D11_BUFFER_DESC {
        ByteWidth: bytes as u32,
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: uso.0 as u32,
        CPUAccessFlags: 0,
        MiscFlags: 0,
        StructureByteStride: 0,
    };
    let iniciales = datos.map(|p| D3D11_SUBRESOURCE_DATA { pSysMem: p, SysMemPitch: 0, SysMemSlicePitch: 0 });
    let mut buffer = None;
    unsafe { d.CreateBuffer(&desc, iniciales.as_ref().map(|x| x as *const _), Some(&mut buffer))? };
    Ok(buffer.unwrap())
}

impl Renderizador for CuboD3d11 {
    fn dibujar(&mut self, c: &Constantes, captura: Option<&mut [u32]>) -> Result<()> {
        unsafe {
            // Todo el dibujo pasa por el context, no por el device
            let ctx = &self.contexto;
            ctx.UpdateSubresource(&self.cb, 0, None, c as *const Constantes as _, 0, 0);

            ctx.OMSetRenderTargets(Some(&[Some(self.rtv.clone())]), &self.dsv);
            ctx.RSSetViewports(Some(&[self.viewport]));
            ctx.ClearRenderTargetView(&self.rtv, &cubo_neutro::FONDO_F);
            ctx.ClearDepthStencilView(&self.dsv, D3D11_CLEAR_DEPTH.0 as u32, 1.0, 0);

            let stride = size_of::<Vertice>() as u32;
            let offset = 0u32;
            ctx.IASetInputLayout(&self.layout);
            ctx.IASetVertexBuffers(0, 1, Some(&Some(self.vb.clone())), Some(&stride), Some(&offset));
            ctx.IASetIndexBuffer(&self.ib, DXGI_FORMAT_R16_UINT, 0);
            ctx.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            ctx.VSSetShader(&self.vs, None);
            ctx.VSSetConstantBuffers(0, Some(&[Some(self.cb.clone())]));
            ctx.PSSetShader(&self.ps, None);
            ctx.PSSetConstantBuffers(0, Some(&[Some(self.cb.clone())]));

            ctx.DrawIndexed(cubo_neutro::NUM_INDICES as u32, 0, 0);
            if let Some(destino) = captura {
                self.capturar(destino)?;
            }
            self.swap.Present(self.config.vsync as u32, DXGI_PRESENT(0)).ok()
        }
    }
}
