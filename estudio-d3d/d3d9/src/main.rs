//! Estudio: cubo 3D con Direct3D 9.
//!
//! cargo run -p estudio_d3d9 --release [-- --fotograma N | --medir N | --sin-vsync]

mod cubo_d3d9;

fn main() -> windows::core::Result<()> {
    estudio_base::ejecutar("Estudio D3D9", cubo_d3d9::CuboD3d9::nuevo)
}
