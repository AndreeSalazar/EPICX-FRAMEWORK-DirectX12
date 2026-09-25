//! # BMOX base
//!
//! Todo lo que es IGUAL en las cuatro versiones, para que cada carpeta
//! (`BMOX-9` … `BMOX-12`) muestre solo lo que cambia de la API gráfica:
//!
//! - Ventana Win32 + bucle de mensajes (`ejecutar`)
//! - Geometría del cubo: 24 vértices (4 por cara) + 36 índices (`cubo`)
//! - Matrices World / View / Projection con glam (`constantes`)
//! - Compilación HLSL en tiempo de ejecución con d3dcompiler_47 (`compilar_hlsl`)
//!
//! Convenciones compartidas por todas las versiones:
//! - Sistema de mano izquierda (el de Direct3D), profundidad 0..1.
//! - Caras frontales en sentido horario → se descartan las traseras (cull back).
//! - Matrices subidas en orden de columnas (`to_cols_array`) y en HLSL `mul(M, v)`.

use std::time::Instant;

use glam::{Mat4, Vec3};
use windows::core::{w, Error, Result, HSTRING, PCSTR};
use windows::Win32::Foundation::{E_FAIL, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

pub const ANCHO: u32 = 1280;
pub const ALTO: u32 = 720;

/// `false` = sin VSync (FPS máximos, como el ejemplo de EPICX) para comparar
/// el rendimiento entre versiones. Ponlo en `true` para limitar a la frecuencia del monitor.
pub const VSYNC: bool = false;

/// Mismo fondo que `examples/cube_dx12.rs` (0xFF101018).
pub const COLOR_FONDO: [f32; 4] = [0x10 as f32 / 255.0, 0x10 as f32 / 255.0, 0x18 as f32 / 255.0, 1.0];

/// Formato de vértice común: posición + normal + color (40 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertice {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

/// Constantes por frame (128 bytes, múltiplo de 16 como exige un constant buffer).
/// - D3D9: registros c0..c3 (WVP) y c4..c7 (World)
/// - D3D10/11/12: `cbuffer` en el registro b0
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Constantes {
    pub wvp: [f32; 16],
    pub world: [f32; 16],
}

/// Lo único que cada versión BMOX tiene que implementar: dibujar un frame.
pub trait BmoxRender {
    fn dibujar(&mut self, c: &Constantes) -> Result<()>;
}

/// Cubo de lado 2 centrado en el origen, con un color por cara.
/// Se usan 24 vértices (y no 8) porque cada cara necesita su propia normal.
pub fn cubo() -> (Vec<Vertice>, Vec<u16>) {
    // (normal, u, v, color) con u × v = -normal: así los vértices
    // (-u+v, u+v, u-v, -u-v) quedan en sentido horario vistos desde fuera.
    let caras: [([f32; 3], [f32; 3], [f32; 3], [f32; 4]); 6] = [
        ([0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.19, 0.88, 0.19, 1.0]), // frente: verde
        ([0.0, 0.0, 1.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.88, 0.19, 0.19, 1.0]), // atrás: rojo
        ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [0.88, 0.88, 0.19, 1.0]),  // derecha: amarillo
        ([-1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [0.19, 0.19, 0.88, 1.0]), // izquierda: azul
        ([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.88, 0.19, 0.88, 1.0]),  // arriba: magenta
        ([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.19, 0.88, 0.88, 1.0]), // abajo: cian
    ];

    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    for (n, u, v, color) in caras {
        let (n, u, v) = (Vec3::from(n), Vec3::from(u), Vec3::from(v));
        let base = vertices.len() as u16;
        for (a, b) in [(-1.0, 1.0), (1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)] {
            vertices.push(Vertice { pos: (n + u * a + v * b).into(), normal: n.into(), color });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    (vertices, indices)
}

/// World (rotación) · View (cámara) · Projection (perspectiva) para el tiempo `t` en segundos.
pub fn constantes(t: f32, aspecto: f32) -> Constantes {
    let world = Mat4::from_rotation_y(t) * Mat4::from_rotation_x(t * 0.7);
    let view = Mat4::look_at_lh(Vec3::new(0.0, 1.5, -5.0), Vec3::ZERO, Vec3::Y);
    let proj = Mat4::perspective_lh(60f32.to_radians(), aspecto, 0.1, 100.0);
    Constantes { wvp: (proj * view * world).to_cols_array(), world: world.to_cols_array() }
}

/// Compila HLSL con `D3DCompile` (d3dcompiler_47.dll, incluido en Windows).
/// Sirve para todos los perfiles: vs_3_0 (D3D9), vs_4_0 (D3D10), vs_5_0 (D3D11/12).
pub fn compilar_hlsl(fuente: &str, entrada: PCSTR, perfil: PCSTR) -> Result<Vec<u8>> {
    let mut codigo: Option<ID3DBlob> = None;
    let mut errores: Option<ID3DBlob> = None;
    let r = unsafe {
        D3DCompile(
            fuente.as_ptr() as _,
            fuente.len(),
            None,
            None,
            None,
            entrada,
            perfil,
            0,
            0,
            &mut codigo,
            Some(&mut errores),
        )
    };
    if let Err(e) = r {
        let msg = errores.map(|b| String::from_utf8_lossy(bytes_blob(&b)).into_owned()).unwrap_or_default();
        return Err(Error::new(e.code(), format!("HLSL: {msg}")));
    }
    Ok(bytes_blob(codigo.as_ref().unwrap()).to_vec())
}

/// Vista en bytes de un `ID3DBlob` (bytecode de shader, firma raíz serializada, etc).
pub fn bytes_blob(blob: &ID3DBlob) -> &[u8] {
    unsafe { std::slice::from_raw_parts(blob.GetBufferPointer() as *const u8, blob.GetBufferSize()) }
}

/// Error con mensaje propio (para fallos que no devuelven HRESULT).
pub fn error(msg: &str) -> Error {
    Error::new(E_FAIL, msg)
}

/// Crea la ventana, construye el renderizador con `crear` y corre el bucle hasta ESC o cerrar.
/// El título muestra los FPS cada medio segundo.
pub fn ejecutar<R: BmoxRender>(titulo: &str, crear: impl FnOnce(HWND, u32, u32) -> Result<R>) -> Result<()> {
    let hwnd = crear_ventana(titulo)?;
    let mut render = crear(hwnd, ANCHO, ALTO)?;
    println!("[{titulo}] listo. ESC para salir.");

    let inicio = Instant::now();
    let mut t_fps = Instant::now();
    let mut frames = 0u32;
    let mut total = 0u64;
    loop {
        let mut msg = MSG::default();
        while unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.as_bool() {
            if msg.message == WM_QUIT {
                let seg = inicio.elapsed().as_secs_f32();
                println!("[{titulo}] {total} frames en {seg:.2}s -> promedio {:.0} FPS", total as f32 / seg);
                return Ok(());
            }
            unsafe {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        render.dibujar(&constantes(inicio.elapsed().as_secs_f32(), ANCHO as f32 / ALTO as f32))?;

        frames += 1;
        total += 1;
        let dt = t_fps.elapsed().as_secs_f32();
        if dt >= 0.5 {
            let texto = HSTRING::from(format!("{titulo} | FPS: {:.0}", frames as f32 / dt));
            unsafe { SetWindowTextW(hwnd, &texto)? };
            frames = 0;
            t_fps = Instant::now();
        }
    }
}

fn crear_ventana(titulo: &str) -> Result<HWND> {
    unsafe {
        let instancia = GetModuleHandleW(None)?;
        let clase = w!("BMOX");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(proc_ventana),
            hInstance: instancia.into(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            lpszClassName: clase,
            ..Default::default()
        };
        if RegisterClassExW(&wc) == 0 {
            return Err(Error::from_win32());
        }

        // Tamaño fijo: sin bordes redimensionables ni maximizar, así ningún ejemplo
        // necesita recrear el swap chain (eso se estudia aparte).
        let estilo = WINDOW_STYLE(WS_OVERLAPPEDWINDOW.0 & !(WS_THICKFRAME.0 | WS_MAXIMIZEBOX.0));
        let mut rect = RECT { left: 0, top: 0, right: ANCHO as i32, bottom: ALTO as i32 };
        AdjustWindowRect(&mut rect, estilo, false)?;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            clase,
            &HSTRING::from(titulo),
            estilo | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            rect.right - rect.left,
            rect.bottom - rect.top,
            None,
            None,
            instancia,
            None,
        )?;
        Ok(hwnd)
    }
}

extern "system" fn proc_ventana(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    const VK_ESCAPE: usize = 0x1B;
    unsafe {
        match msg {
            WM_KEYDOWN if wp.0 == VK_ESCAPE => {
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }
}
