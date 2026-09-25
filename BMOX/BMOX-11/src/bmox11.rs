//! # BMOX-11 → Direct3D 11 (2009, Windows 7)
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

use bmox_base::{compilar_hlsl, cubo, BmoxRender, Constantes, Vertice, COLOR_FONDO, VSYNC};
use windows::core::{s, Result};
use windows::Win32::Foundation::{HMODULE, HWND, TRUE};
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;

const HLSL: &str = include_str!("cubo.hlsl");

pub struct Bmox11 {
    contexto: ID3D11DeviceContext,
    swap: IDXGISwapChain,
    rtv: ID3D11RenderTargetView,
    dsv: ID3D11DepthStencilView,
    layout: ID3D11InputLayout,
    vs: ID3D11VertexShader,
    ps: ID3D11PixelShader,
    vb: ID3D11Buffer,
    ib: ID3D11Buffer,
    cb: ID3D11Buffer,
    viewport: D3D11_VIEWPORT,
    num_indices: u32,
    _dispositivo: ID3D11Device,
}

impl Bmox11 {
    pub fn nuevo(hwnd: HWND, ancho: u32, alto: u32) -> Result<Self> {
        unsafe {
            // 1) Device + immediate context + swap chain (mismo DXGI_SWAP_CHAIN_DESC que D3D10)
            let desc_swap = DXGI_SWAP_CHAIN_DESC {
                BufferDesc: DXGI_MODE_DESC { Width: ancho, Height: alto, Format: DXGI_FORMAT_R8G8B8A8_UNORM, ..Default::default() },
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
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
            println!("[BMOX-11] feature level: {}.{}", nivel.0 >> 12, (nivel.0 >> 8) & 0xF);

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

            // 5) Buffers
            let (vertices, indices) = cubo();
            let vb = crear_buffer(&dispositivo, D3D11_BIND_VERTEX_BUFFER, size_of::<Vertice>() * vertices.len(), Some(vertices.as_ptr() as _))?;
            let ib = crear_buffer(&dispositivo, D3D11_BIND_INDEX_BUFFER, size_of::<u16>() * indices.len(), Some(indices.as_ptr() as _))?;
            let cb = crear_buffer(&dispositivo, D3D11_BIND_CONSTANT_BUFFER, size_of::<Constantes>(), None)?;

            let viewport = D3D11_VIEWPORT { TopLeftX: 0.0, TopLeftY: 0.0, Width: ancho as f32, Height: alto as f32, MinDepth: 0.0, MaxDepth: 1.0 };

            Ok(Self {
                contexto,
                swap,
                rtv: rtv.unwrap(),
                dsv: dsv.unwrap(),
                layout: layout.unwrap(),
                vs: vs.unwrap(),
                ps: ps.unwrap(),
                vb,
                ib,
                cb,
                viewport,
                num_indices: indices.len() as u32,
                _dispositivo: dispositivo,
            })
        }
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

impl BmoxRender for Bmox11 {
    fn dibujar(&mut self, c: &Constantes) -> Result<()> {
        unsafe {
            // Todo el dibujo pasa por el context, no por el device
            let ctx = &self.contexto;
            ctx.UpdateSubresource(&self.cb, 0, None, c as *const Constantes as _, 0, 0);

            ctx.OMSetRenderTargets(Some(&[Some(self.rtv.clone())]), &self.dsv);
            ctx.RSSetViewports(Some(&[self.viewport]));
            ctx.ClearRenderTargetView(&self.rtv, &COLOR_FONDO);
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

            ctx.DrawIndexed(self.num_indices, 0, 0);
            self.swap.Present(if VSYNC { 1 } else { 0 }, DXGI_PRESENT(0)).ok()
        }
    }
}
