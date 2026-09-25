//! # estudio_base (solo Windows)
//!
//! Lo que comparten los cuatro cubos D3D y que es **de Windows**: ventana Win32,
//! bucle de mensajes, compilador HLSL (d3dcompiler_47.dll) y escritura de PNG.
//! La malla, las matrices y la luz NO están aquí: vienen de `cubo-neutro`.
//!
//! Modos (argumentos de línea de comandos, iguales en los cuatro .exe):
//! - sin argumentos:        interactivo con VSync; la rotación avanza un paso por fotograma.
//! - `--sin-vsync`:         interactivo sin VSync (gira tan rápido como dibuja).
//! - `--fotograma N`:       dibuja solo el fotograma N en una ventana oculta, lo guarda como
//!                          PNG (`--salida ruta.png`, por defecto `cubo_fNNNN.png`) y sale.
//! - `--medir N`:           dibuja los fotogramas 0..N sin VSync y muestra los FPS.
//! - `--corregir-medio-pixel`: solo D3D9; desplaza la geometría -0.5 px para usar la
//!                          convención de centros de píxel de D3D10+ (las demás lo ignoran).

use std::path::{Path, PathBuf};
use std::time::Instant;

pub use cubo_neutro::{self, Constantes, Vertice};
use windows::core::{w, Error, Result, HSTRING, PCSTR};
use windows::Win32::Foundation::{E_FAIL, E_INVALIDARG, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

pub const ANCHO: u32 = 1280;
pub const ALTO: u32 = 720;

/// Lo que recibe cada cubo al crearse.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub ancho: u32,
    pub alto: u32,
    pub vsync: bool,
    /// Solo D3D9: compensar que sus centros de píxel están en enteros y no en +0.5.
    pub corregir_medio_pixel: bool,
}

/// Lo único que cada versión D3D implementa.
pub trait Renderizador {
    /// Dibuja un fotograma y lo presenta. Si `captura` es `Some`, antes de presentar copia
    /// el back buffer ahí como `0xAARRGGBB`, fila 0 arriba (el formato de `cubo-neutro`).
    fn dibujar(&mut self, c: &Constantes, captura: Option<&mut [u32]>) -> Result<()>;
}

enum Modo {
    Interactivo,
    Fotograma { n: u32, salida: PathBuf },
    Medir { n: u32 },
}

fn leer_argumentos() -> Result<(Modo, bool, bool)> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let numero = |i: usize| -> Result<u32> {
        args.get(i + 1).and_then(|s| s.parse().ok()).ok_or_else(|| Error::new(E_INVALIDARG, format!("{} espera un número", args[i])))
    };
    let mut modo = Modo::Interactivo;
    let mut vsync = true;
    let mut medio_pixel = false;
    let mut salida = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--fotograma" => {
                modo = Modo::Fotograma { n: numero(i)?, salida: PathBuf::new() };
                i += 1;
            }
            "--medir" => {
                modo = Modo::Medir { n: numero(i)? };
                i += 1;
            }
            "--salida" => {
                salida = args.get(i + 1).map(PathBuf::from);
                i += 1;
            }
            "--sin-vsync" => vsync = false,
            "--corregir-medio-pixel" => medio_pixel = true,
            otro => return Err(Error::new(E_INVALIDARG, format!("argumento desconocido: {otro}"))),
        }
        i += 1;
    }
    if let Modo::Fotograma { n, salida: s } = &mut modo {
        *s = salida.unwrap_or_else(|| PathBuf::from(format!("cubo_f{n:04}.png")));
    }
    Ok((modo, vsync, medio_pixel))
}

fn constantes_de(fotograma: u32) -> Constantes {
    cubo_neutro::constantes(cubo_neutro::angulo_de_fotograma(fotograma), ANCHO as f32 / ALTO as f32)
}

/// Punto de entrada común de los cuatro .exe.
pub fn ejecutar<R: Renderizador>(titulo: &str, crear: impl FnOnce(HWND, &Config) -> Result<R>) -> Result<()> {
    let (modo, vsync, corregir_medio_pixel) = leer_argumentos()?;
    let config = |vsync| Config { ancho: ANCHO, alto: ALTO, vsync, corregir_medio_pixel };
    match modo {
        Modo::Fotograma { n, salida } => {
            let hwnd = crear_ventana(titulo, false)?;
            let mut render = crear(hwnd, &config(false))?;
            let mut pixeles = vec![0u32; (ANCHO * ALTO) as usize];
            render.dibujar(&constantes_de(n), Some(&mut pixeles))?;
            guardar_png(&salida, ANCHO, ALTO, &pixeles).map_err(|e| Error::new(E_FAIL, format!("{}: {e}", salida.display())))?;
            println!("[{titulo}] fotograma {n} -> {}", salida.display());
            Ok(())
        }
        Modo::Medir { n } => {
            let hwnd = crear_ventana(titulo, true)?;
            let mut render = crear(hwnd, &config(false))?;
            let inicio = Instant::now();
            for f in 0..n {
                if !atender_mensajes() {
                    break;
                }
                render.dibujar(&constantes_de(f), None)?;
            }
            let seg = inicio.elapsed().as_secs_f64();
            println!("[{titulo}] {n} fotogramas en {seg:.3} s -> {:.0} FPS", n as f64 / seg);
            Ok(())
        }
        Modo::Interactivo => {
            let hwnd = crear_ventana(titulo, true)?;
            let mut render = crear(hwnd, &config(vsync))?;
            println!("[{titulo}] VSync {}. ESC para salir.", if vsync { "sí" } else { "no" });
            let mut fotograma = 0u32;
            let mut t_fps = Instant::now();
            let mut cuenta = 0u32;
            while atender_mensajes() {
                render.dibujar(&constantes_de(fotograma), None)?;
                fotograma = fotograma.wrapping_add(1);
                cuenta += 1;
                let dt = t_fps.elapsed().as_secs_f32();
                if dt >= 0.5 {
                    let texto = HSTRING::from(format!("{titulo} | fotograma {fotograma} | FPS: {:.0}", cuenta as f32 / dt));
                    unsafe { SetWindowTextW(hwnd, &texto)? };
                    cuenta = 0;
                    t_fps = Instant::now();
                }
            }
            Ok(())
        }
    }
}

/// Procesa los mensajes pendientes. Devuelve `false` cuando llega WM_QUIT.
fn atender_mensajes() -> bool {
    let mut msg = MSG::default();
    unsafe {
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            if msg.message == WM_QUIT {
                return false;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    true
}

/// PNG RGB de 8 bits a partir de píxeles `0xAARRGGBB` (se descarta el alfa).
pub fn guardar_png(ruta: &Path, ancho: u32, alto: u32, pixeles: &[u32]) -> std::io::Result<()> {
    if let Some(dir) = ruta.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)?;
    }
    let rgb: Vec<u8> = pixeles.iter().flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8]).collect();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(ruta)?), ancho, alto);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()?.write_image_data(&rgb)?;
    Ok(())
}

/// Compila HLSL con `D3DCompile` (d3dcompiler_47.dll, incluido en Windows).
/// Sirve para todos los perfiles: vs_3_0 (D3D9), vs_4_0 (D3D10), vs_5_0 (D3D11/12).
pub fn compilar_hlsl(fuente: &str, entrada: PCSTR, perfil: PCSTR) -> Result<Vec<u8>> {
    let mut codigo: Option<ID3DBlob> = None;
    let mut errores: Option<ID3DBlob> = None;
    let r = unsafe {
        D3DCompile(fuente.as_ptr() as _, fuente.len(), None, None, None, entrada, perfil, 0, 0, &mut codigo, Some(&mut errores))
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

/// Copia filas de `alto` × `ancho` píxeles desde memoria con `pitch` bytes por fila,
/// convirtiendo cada píxel con `convertir`.
///
/// # Safety
/// `origen` debe apuntar a `alto * pitch` bytes legibles.
pub unsafe fn copiar_filas(origen: *const u8, pitch: usize, ancho: u32, alto: u32, destino: &mut [u32], convertir: impl Fn(u32) -> u32) {
    for y in 0..alto as usize {
        let fila = std::slice::from_raw_parts(origen.add(y * pitch) as *const u32, ancho as usize);
        for (x, p) in fila.iter().enumerate() {
            destino[y * ancho as usize + x] = convertir(*p);
        }
    }
}

/// R8G8B8A8_UNORM leído como u32 little-endian (0xAABBGGRR) → 0xFFRRGGBB.
pub fn rgba_a_argb(p: u32) -> u32 {
    0xFF00_0000 | ((p & 0xFF) << 16) | (p & 0xFF00) | ((p >> 16) & 0xFF)
}

fn crear_ventana(titulo: &str, visible: bool) -> Result<HWND> {
    unsafe {
        let instancia = GetModuleHandleW(None)?;
        let clase = w!("EstudioD3D");
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

        // Tamaño fijo: ningún ejemplo necesita recrear el swap chain.
        let mut estilo = WINDOW_STYLE(WS_OVERLAPPEDWINDOW.0 & !(WS_THICKFRAME.0 | WS_MAXIMIZEBOX.0));
        let mut rect = RECT { left: 0, top: 0, right: ANCHO as i32, bottom: ALTO as i32 };
        AdjustWindowRect(&mut rect, estilo, false)?;
        if visible {
            estilo |= WS_VISIBLE;
        }

        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            clase,
            &HSTRING::from(titulo),
            estilo,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            rect.right - rect.left,
            rect.bottom - rect.top,
            None,
            None,
            instancia,
            None,
        )
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
