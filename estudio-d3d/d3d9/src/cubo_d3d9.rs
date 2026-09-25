//! # BMOX-9 → Direct3D 9 (2002)
//!
//! Lo característico de D3D9 frente a las versiones nuevas:
//! - Un solo objeto `IDirect3DDevice9` hace TODO: crea recursos, guarda el estado y dibuja.
//! - No hay DXGI: el back buffer y el depth buffer los crea el propio device
//!   (`D3DPRESENT_PARAMETERS`, `EnableAutoDepthStencil`).
//! - Estado global tipo "máquina de estados": Set* modifica el device y se queda así.
//! - Recursos con `Lock`/`Unlock` para escribir memoria.
//! - Shaders opcionales (existe el pipeline fijo). Aquí usamos HLSL SM3.
//! - Las constantes van a registros (`SetVertexShaderConstantF`), no a constant buffers.
//! - "Device lost": el device se puede perder (Ctrl+Alt+Supr, pantalla completa de otra
//!   app, etc.) y hay que llamar `Reset`. Por eso los buffers usan `D3DPOOL_MANAGED`:
//!   D3D9 guarda una copia en RAM y los restaura solo tras el `Reset`.
//! - Frame: `Clear` → `BeginScene` → dibujar → `EndScene` → `Present`.

use std::mem::size_of;

use bmox_base::{compilar_hlsl, cubo, error, BmoxRender, Constantes, Vertice, COLOR_FONDO, VSYNC};
use windows::core::{s, Result, HRESULT};
use windows::Win32::Foundation::{HWND, TRUE};
use windows::Win32::Graphics::Direct3D9::*;

const HLSL: &str = include_str!("cubo.hlsl");

// Valores de d3d9.h (MAKE_D3DHRESULT) que el crate `windows` no exporta.
const D3DERR_DEVICELOST: HRESULT = HRESULT(0x8876_0868_u32 as i32);
const D3DERR_DEVICENOTRESET: HRESULT = HRESULT(0x8876_0869_u32 as i32);

pub struct Bmox9 {
    dispositivo: IDirect3DDevice9,
    parametros: D3DPRESENT_PARAMETERS,
    vb: IDirect3DVertexBuffer9,
    ib: IDirect3DIndexBuffer9,
    declaracion: IDirect3DVertexDeclaration9,
    vs: IDirect3DVertexShader9,
    ps: IDirect3DPixelShader9,
    num_vertices: u32,
    num_triangulos: u32,
    _d3d: IDirect3D9,
}

impl Bmox9 {
    pub fn nuevo(hwnd: HWND, ancho: u32, alto: u32) -> Result<Self> {
        unsafe {
            // 1) Objeto raíz D3D9 (enumera adaptadores y crea devices)
            let d3d = Direct3DCreate9(D3D_SDK_VERSION).ok_or_else(|| error("Direct3DCreate9 falló"))?;

            // 2) Device + back buffer + depth buffer, todo en una sola llamada
            let mut parametros = D3DPRESENT_PARAMETERS {
                BackBufferWidth: ancho,
                BackBufferHeight: alto,
                BackBufferFormat: D3DFMT_X8R8G8B8,
                BackBufferCount: 1,
                SwapEffect: D3DSWAPEFFECT_DISCARD,
                hDeviceWindow: hwnd,
                Windowed: TRUE,
                EnableAutoDepthStencil: TRUE,
                AutoDepthStencilFormat: D3DFMT_D24S8,
                PresentationInterval: if VSYNC { D3DPRESENT_INTERVAL_ONE as u32 } else { D3DPRESENT_INTERVAL_IMMEDIATE as u32 },
                ..Default::default()
            };
            let mut dispositivo = None;
            d3d.CreateDevice(
                D3DADAPTER_DEFAULT,
                D3DDEVTYPE_HAL,
                hwnd,
                D3DCREATE_HARDWARE_VERTEXPROCESSING as u32,
                &mut parametros,
                &mut dispositivo,
            )?;
            let dispositivo = dispositivo.unwrap();

            // 3) Vertex buffer e index buffer (Lock → copiar → Unlock)
            let (vertices, indices) = cubo();
            let bytes_vb = (vertices.len() * size_of::<Vertice>()) as u32;
            let mut vb = None;
            dispositivo.CreateVertexBuffer(bytes_vb, D3DUSAGE_WRITEONLY as u32, 0, D3DPOOL_MANAGED, &mut vb, std::ptr::null_mut())?;
            let vb: IDirect3DVertexBuffer9 = vb.unwrap();
            let mut p = std::ptr::null_mut();
            vb.Lock(0, 0, &mut p, 0)?;
            std::ptr::copy_nonoverlapping(vertices.as_ptr(), p as *mut Vertice, vertices.len());
            vb.Unlock()?;

            let bytes_ib = (indices.len() * size_of::<u16>()) as u32;
            let mut ib = None;
            dispositivo.CreateIndexBuffer(bytes_ib, D3DUSAGE_WRITEONLY as u32, D3DFMT_INDEX16, D3DPOOL_MANAGED, &mut ib, std::ptr::null_mut())?;
            let ib: IDirect3DIndexBuffer9 = ib.unwrap();
            ib.Lock(0, 0, &mut p, 0)?;
            std::ptr::copy_nonoverlapping(indices.as_ptr(), p as *mut u16, indices.len());
            ib.Unlock()?;

            // 4) Declaración de vértices: describe el struct Vertice al device
            //    (el equivalente a lo que en D3D10+ es el "input layout")
            let elemento = |offset: u16, tipo: D3DDECLTYPE, uso: D3DDECLUSAGE| D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: offset,
                Type: tipo.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: uso.0 as u8,
                UsageIndex: 0,
            };
            let elementos = [
                elemento(0, D3DDECLTYPE_FLOAT3, D3DDECLUSAGE_POSITION),
                elemento(12, D3DDECLTYPE_FLOAT3, D3DDECLUSAGE_NORMAL),
                elemento(24, D3DDECLTYPE_FLOAT4, D3DDECLUSAGE_COLOR),
                // D3DDECL_END()
                D3DVERTEXELEMENT9 { Stream: 0xFF, Offset: 0, Type: D3DDECLTYPE_UNUSED.0 as u8, Method: 0, Usage: 0, UsageIndex: 0 },
            ];
            let declaracion = dispositivo.CreateVertexDeclaration(elementos.as_ptr())?;

            // 5) Shaders SM3
            let codigo_vs = compilar_hlsl(HLSL, s!("VSMain"), s!("vs_3_0"))?;
            let codigo_ps = compilar_hlsl(HLSL, s!("PSMain"), s!("ps_3_0"))?;
            let vs = dispositivo.CreateVertexShader(codigo_vs.as_ptr() as *const u32)?;
            let ps = dispositivo.CreatePixelShader(codigo_ps.as_ptr() as *const u32)?;

            Ok(Self {
                dispositivo,
                parametros,
                vb,
                ib,
                declaracion,
                vs,
                ps,
                num_vertices: vertices.len() as u32,
                num_triangulos: (indices.len() / 3) as u32,
                _d3d: d3d,
            })
        }
    }

    /// Maneja el "device lost" clásico de D3D9. Devuelve `false` si este frame no se puede dibujar.
    fn device_disponible(&mut self) -> Result<bool> {
        unsafe {
            match self.dispositivo.TestCooperativeLevel() {
                Ok(()) => Ok(true),
                Err(e) if e.code() == D3DERR_DEVICELOST => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    Ok(false)
                }
                Err(e) if e.code() == D3DERR_DEVICENOTRESET => {
                    self.dispositivo.Reset(&mut self.parametros)?;
                    Ok(true)
                }
                Err(e) => Err(e),
            }
        }
    }
}

/// D3DCOLOR es un u32 ARGB (no floats como en D3D10+).
fn argb(c: [f32; 4]) -> u32 {
    let b = |x: f32| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
    (b(c[3]) << 24) | (b(c[0]) << 16) | (b(c[1]) << 8) | b(c[2])
}

impl BmoxRender for Bmox9 {
    fn dibujar(&mut self, c: &Constantes) -> Result<()> {
        if !self.device_disponible()? {
            return Ok(());
        }
        unsafe {
            let d = &self.dispositivo;
            d.Clear(0, std::ptr::null(), (D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER) as u32, argb(COLOR_FONDO), 1.0, 0)?;
            d.BeginScene()?;

            d.SetVertexDeclaration(&self.declaracion)?;
            d.SetStreamSource(0, &self.vb, 0, size_of::<Vertice>() as u32)?;
            d.SetIndices(&self.ib)?;
            d.SetVertexShader(&self.vs)?;
            d.SetPixelShader(&self.ps)?;
            // 8 registros float4: c0..c3 = WVP, c4..c7 = World
            d.SetVertexShaderConstantF(0, c as *const Constantes as *const f32, 8)?;
            d.DrawIndexedPrimitive(D3DPT_TRIANGLELIST, 0, 0, self.num_vertices, 0, self.num_triangulos)?;

            d.EndScene()?;
            match d.Present(std::ptr::null(), std::ptr::null(), None, std::ptr::null()) {
                Err(e) if e.code() == D3DERR_DEVICELOST => Ok(()), // se resuelve en el próximo frame
                r => r,
            }
        }
    }
}
