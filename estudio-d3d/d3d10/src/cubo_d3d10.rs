//! # Direct3D 10 (2006, Windows Vista)
//!
//! El gran corte con D3D9:
//! - Aparece **DXGI**: el swap chain (back buffers + Present) ya no es del device.
//! - Ya no hay pipeline fijo ni "caps": los shaders son obligatorios (SM4).
//! - Los recursos son genéricos (`ID3D10Buffer`, `ID3D10Texture2D`) y se usan a través
//!   de **vistas**: Render Target View, Depth Stencil View, Shader Resource View.
//! - El depth buffer ya no es automático: es una textura + su vista.
//! - **Input layout** validado contra la firma del vertex shader.
//! - **Constant buffers** en lugar de registros sueltos.
//! - El estado se agrupa en objetos inmutables (rasterizer/blend/depth state);
//!   aquí se usan los valores por defecto: depth test ON, cull back, sentido horario.
//! - Centros de píxel en +0.5 (en D3D9 estaban en coordenadas enteras).
//! - Desaparece el "device lost" de D3D9.
//! - Sigue habiendo UN solo objeto (device) que crea recursos y dibuja.

use std::mem::size_of;

use estudio_base::{compilar_hlsl, cubo_neutro, rgba_a_argb, Config, Constantes, Renderizador, Vertice};
use windows::core::{s, Result};
use windows::Win32::Foundation::{HMODULE, HWND, TRUE};
use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
use windows::Win32::Graphics::Direct3D10::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;

const HLSL: &str = include_str!("cubo.hlsl");

pub struct CuboD3d10 {
    dispositivo: ID3D10Device,
    swap: IDXGISwapChain,
    back_buffer: ID3D10Texture2D,
    rtv: ID3D10RenderTargetView,
    dsv: ID3D10DepthStencilView,
    layout: ID3D10InputLayout,
    vs: ID3D10VertexShader,
    ps: ID3D10PixelShader,
    vb: ID3D10Buffer,
    ib: ID3D10Buffer,
    cb: ID3D10Buffer,
    viewport: D3D10_VIEWPORT,
    config: Config,
}

impl CuboD3d10 {
    pub fn nuevo(hwnd: HWND, config: &Config) -> Result<Self> {
        let (ancho, alto) = (config.ancho, config.alto);
        unsafe {
            // 1) Device + swap chain DXGI en una llamada (modelo "blt" DISCARD, el típico de la época)
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
            let mut swap = None;
            let mut dispositivo = None;
            D3D10CreateDeviceAndSwapChain(
                None,
                D3D10_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                0,
                D3D10_SDK_VERSION,
                Some(&desc_swap),
                Some(&mut swap),
                Some(&mut dispositivo),
            )?;
            let (swap, dispositivo): (IDXGISwapChain, ID3D10Device) = (swap.unwrap(), dispositivo.unwrap());

            // 2) Render target view sobre el back buffer del swap chain
            let back_buffer: ID3D10Texture2D = swap.GetBuffer(0)?;
            let mut rtv = None;
            dispositivo.CreateRenderTargetView(&back_buffer, None, Some(&mut rtv))?;

            // 3) Depth buffer: textura D24S8 + depth stencil view
            let desc_prof = D3D10_TEXTURE2D_DESC {
                Width: ancho,
                Height: alto,
                MipLevels: 1,
                ArraySize: 1,
                Format: DXGI_FORMAT_D24_UNORM_S8_UINT,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Usage: D3D10_USAGE_DEFAULT,
                BindFlags: D3D10_BIND_DEPTH_STENCIL.0 as u32,
                CPUAccessFlags: 0,
                MiscFlags: 0,
            };
            let profundidad = dispositivo.CreateTexture2D(&desc_prof, None)?;
            let mut dsv = None;
            dispositivo.CreateDepthStencilView(&profundidad, None, Some(&mut dsv))?;

            // 4) Shaders SM4 + input layout (se valida contra la firma de entrada del VS)
            let codigo_vs = compilar_hlsl(HLSL, s!("VSMain"), s!("vs_4_0"))?;
            let codigo_ps = compilar_hlsl(HLSL, s!("PSMain"), s!("ps_4_0"))?;
            let mut vs = None;
            let mut ps = None;
            dispositivo.CreateVertexShader(&codigo_vs, Some(&mut vs))?;
            dispositivo.CreatePixelShader(&codigo_ps, Some(&mut ps))?;

            let elemento = |semantica, formato, offset| D3D10_INPUT_ELEMENT_DESC {
                SemanticName: semantica,
                SemanticIndex: 0,
                Format: formato,
                InputSlot: 0,
                AlignedByteOffset: offset,
                InputSlotClass: D3D10_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            };
            let elementos = [
                elemento(s!("POSITION"), DXGI_FORMAT_R32G32B32_FLOAT, 0),
                elemento(s!("NORMAL"), DXGI_FORMAT_R32G32B32_FLOAT, 12),
                elemento(s!("COLOR"), DXGI_FORMAT_R32G32B32A32_FLOAT, 24),
            ];
            let mut layout = None;
            dispositivo.CreateInputLayout(&elementos, &codigo_vs, Some(&mut layout))?;

            // 5) Buffers: malla de cubo-neutro con datos iniciales; constantes vacío
            let vertices = cubo_neutro::vertices();
            let indices = cubo_neutro::indices();
            let vb = crear_buffer(&dispositivo, D3D10_BIND_VERTEX_BUFFER, size_of_val(&vertices), Some(vertices.as_ptr() as _))?;
            let ib = crear_buffer(&dispositivo, D3D10_BIND_INDEX_BUFFER, size_of_val(&indices), Some(indices.as_ptr() as _))?;
            let cb = crear_buffer(&dispositivo, D3D10_BIND_CONSTANT_BUFFER, size_of::<Constantes>(), None)?;

            let viewport = D3D10_VIEWPORT { TopLeftX: 0, TopLeftY: 0, Width: ancho, Height: alto, MinDepth: 0.0, MaxDepth: 1.0 };

            Ok(Self {
                dispositivo,
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

    /// Back buffer → textura STAGING (legible por la CPU) → `destino`.
    fn capturar(&self, destino: &mut [u32]) -> Result<()> {
        unsafe {
            let desc = D3D10_TEXTURE2D_DESC {
                Width: self.config.ancho,
                Height: self.config.alto,
                MipLevels: 1,
                ArraySize: 1,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Usage: D3D10_USAGE_STAGING,
                BindFlags: 0,
                CPUAccessFlags: D3D10_CPU_ACCESS_READ.0 as u32,
                MiscFlags: 0,
            };
            let staging = self.dispositivo.CreateTexture2D(&desc, None)?;
            self.dispositivo.CopyResource(&staging, &self.back_buffer);
            let m = staging.Map(0, D3D10_MAP_READ, 0)?; // bloquea hasta que la copia termina
            estudio_base::copiar_filas(m.pData as *const u8, m.RowPitch as usize, self.config.ancho, self.config.alto, destino, rgba_a_argb);
            staging.Unmap(0);
        }
        Ok(())
    }
}

fn crear_buffer(d: &ID3D10Device, uso: D3D10_BIND_FLAG, bytes: usize, datos: Option<*const std::ffi::c_void>) -> Result<ID3D10Buffer> {
    let desc = D3D10_BUFFER_DESC {
        ByteWidth: bytes as u32,
        Usage: D3D10_USAGE_DEFAULT,
        BindFlags: uso.0 as u32,
        CPUAccessFlags: 0,
        MiscFlags: 0,
    };
    let iniciales = datos.map(|p| D3D10_SUBRESOURCE_DATA { pSysMem: p, SysMemPitch: 0, SysMemSlicePitch: 0 });
    let mut buffer = None;
    unsafe { d.CreateBuffer(&desc, iniciales.as_ref().map(|x| x as *const _), Some(&mut buffer))? };
    Ok(buffer.unwrap())
}

impl Renderizador for CuboD3d10 {
    fn dibujar(&mut self, c: &Constantes, captura: Option<&mut [u32]>) -> Result<()> {
        unsafe {
            let d = &self.dispositivo;
            d.UpdateSubresource(&self.cb, 0, None, c as *const Constantes as _, 0, 0);

            d.OMSetRenderTargets(Some(&[Some(self.rtv.clone())]), &self.dsv);
            d.RSSetViewports(Some(&[self.viewport]));
            d.ClearRenderTargetView(&self.rtv, &cubo_neutro::FONDO_F);
            d.ClearDepthStencilView(&self.dsv, D3D10_CLEAR_DEPTH.0 as u32, 1.0, 0);

            let stride = size_of::<Vertice>() as u32;
            let offset = 0u32;
            d.IASetInputLayout(&self.layout);
            d.IASetVertexBuffers(0, 1, Some(&Some(self.vb.clone())), Some(&stride), Some(&offset));
            d.IASetIndexBuffer(&self.ib, DXGI_FORMAT_R16_UINT, 0);
            d.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            d.VSSetShader(&self.vs);
            d.VSSetConstantBuffers(0, Some(&[Some(self.cb.clone())]));
            d.PSSetShader(&self.ps);
            d.PSSetConstantBuffers(0, Some(&[Some(self.cb.clone())]));

            d.DrawIndexed(cubo_neutro::NUM_INDICES as u32, 0, 0);
            if let Some(destino) = captura {
                self.capturar(destino)?;
            }
            self.swap.Present(self.config.vsync as u32, DXGI_PRESENT(0)).ok()
        }
    }
}
