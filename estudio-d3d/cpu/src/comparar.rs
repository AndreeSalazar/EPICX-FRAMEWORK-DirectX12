//! Compara dos PNG del mismo tamaño píxel a píxel.
//!
//!   comparar a.png b.png [--mapa diferencias.png] [--listar N]
//!
//! Imprime cuántos píxeles difieren y cuánto (máxima diferencia por canal, 0..255).
//! Con `--mapa` guarda una imagen: negro = igual, blanco = distinto.
//! Con `--listar N` imprime las coordenadas y colores de los primeros N píxeles distintos.

use std::fs::File;
use std::io::BufWriter;

fn leer(ruta: &str) -> Result<(u32, u32, Vec<[u8; 3]>), Box<dyn std::error::Error>> {
    let dec = png::Decoder::new(File::open(ruta)?);
    let mut lector = dec.read_info()?;
    let mut buf = vec![0; lector.output_buffer_size()];
    let info = lector.next_frame(&mut buf)?;
    let canales = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        otro => return Err(format!("{ruta}: formato {otro:?} no soportado").into()),
    };
    let px = buf[..info.buffer_size()].chunks(canales).map(|c| [c[0], c[1], c[2]]).collect();
    Ok((info.width, info.height, px))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        return Err("uso: comparar a.png b.png [--mapa diferencias.png]".into());
    }
    let mapa = args.iter().position(|a| a == "--mapa").and_then(|i| args.get(i + 1));
    let (wa, ha, a) = leer(&args[0])?;
    let (wb, hb, b) = leer(&args[1])?;
    if (wa, ha) != (wb, hb) {
        return Err(format!("tamaños distintos: {wa}x{ha} vs {wb}x{hb}").into());
    }

    // histograma[d] = píxeles cuya mayor diferencia de canal es d
    let mut histograma = [0u64; 256];
    for (p, q) in a.iter().zip(&b) {
        let d = (0..3).map(|k| p[k].abs_diff(q[k])).max().unwrap();
        histograma[d as usize] += 1;
    }
    let total = a.len() as u64;
    let distintos = total - histograma[0];
    println!("{} vs {}", args[0], args[1]);
    println!("  píxeles: {total}, distintos: {distintos} ({:.4} %)", distintos as f64 * 100.0 / total as f64);
    for (d, n) in histograma.iter().enumerate().skip(1).filter(|(_, n)| **n > 0) {
        println!("    diferencia máx. de canal {d:>3}: {n} píxeles");
    }

    let listar: usize = args.iter().position(|a| a == "--listar").and_then(|i| args.get(i + 1)).and_then(|n| n.parse().ok()).unwrap_or(0);
    for (i, (p, q)) in a.iter().zip(&b).enumerate().filter(|(_, (p, q))| p != q).take(listar) {
        let (x, y) = (i as u32 % wa, i as u32 / wa);
        println!("    ({x}, {y}): {:02X}{:02X}{:02X} vs {:02X}{:02X}{:02X}", p[0], p[1], p[2], q[0], q[1], q[2]);
    }

    if let Some(ruta) = mapa {
        let datos: Vec<u8> = a.iter().zip(&b).map(|(p, q)| if p == q { 0 } else { 255 }).collect();
        let mut enc = png::Encoder::new(BufWriter::new(File::create(ruta)?), wa, ha);
        enc.set_color(png::ColorType::Grayscale);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header()?.write_image_data(&datos)?;
    }
    Ok(())
}
