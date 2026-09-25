//! Dibuja un fotograma con `cubo_neutro::dibujar_por_cpu_con` y lo guarda como PNG.
//! Mismos argumentos que los .exe D3D en modo captura:
//!
//!   estudio_cpu --fotograma 30 --salida referencia/cpu/cubo_f0030.png
//!
//! Sin más opciones usa `Reglas::D3D10` (el juez). Cada opción cambia una de las
//! cuatro reglas de `cubo_neutro::Reglas`, para medir qué pasa si no se cumple:
//!
//!   --centro-entero          regla 1: centros de píxel en enteros (como D3D9)
//!   --subpixel-mitad-arriba  regla 2: ajuste a 1/256 con mitades hacia arriba
//!   --viewport-normalizado   regla 3: viewport como (ndc*0.5+0.5)*ancho
//!   --unorm-12bits           regla 4: UNORM8 truncando a 12 bits (lo medido en una RTX 3060)

use std::path::PathBuf;

use cubo_neutro::Reglas;

const ANCHO: u32 = 1280;
const ALTO: u32 = 720;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut fotograma = 0u32;
    let mut salida = None;
    let mut reglas = Reglas::D3D10;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--fotograma" => {
                fotograma = args.get(i + 1).ok_or("--fotograma espera un número")?.parse()?;
                i += 1;
            }
            "--salida" => {
                salida = Some(PathBuf::from(args.get(i + 1).ok_or("--salida espera una ruta")?));
                i += 1;
            }
            "--centro-entero" => reglas.centro_medio_pixel = false,
            "--subpixel-mitad-arriba" => reglas.subpixel_al_par = false,
            "--viewport-normalizado" => reglas.viewport_escalado = false,
            "--unorm-12bits" => reglas.unorm8 = cubo_neutro::a_unorm8_truncar_12,
            otro => return Err(format!("argumento desconocido: {otro}").into()),
        }
        i += 1;
    }
    let salida = salida.unwrap_or_else(|| PathBuf::from(format!("cubo_f{fotograma:04}.png")));

    let mut pixeles = vec![0u32; (ANCHO * ALTO) as usize];
    let inicio = std::time::Instant::now();
    cubo_neutro::dibujar_por_cpu_con(cubo_neutro::angulo_de_fotograma(fotograma), ANCHO, ALTO, &mut pixeles, &reglas);
    let ms = inicio.elapsed().as_secs_f64() * 1000.0;

    if let Some(dir) = salida.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)?;
    }
    let rgb: Vec<u8> = pixeles.iter().flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8]).collect();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(&salida)?), ANCHO, ALTO);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()?.write_image_data(&rgb)?;
    println!("[CPU] fotograma {fotograma} en {ms:.1} ms -> {}", salida.display());
    Ok(())
}
