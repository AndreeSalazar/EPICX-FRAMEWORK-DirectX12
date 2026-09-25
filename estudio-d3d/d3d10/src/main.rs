//! Estudio: cubo 3D con Direct3D 10.
//!
//! cargo run -p estudio_d3d10 --release [-- --fotograma N | --medir N | --sin-vsync]

mod cubo_d3d10;

fn main() -> windows::core::Result<()> {
    estudio_base::ejecutar("Estudio D3D10", cubo_d3d10::CuboD3d10::nuevo)
}
