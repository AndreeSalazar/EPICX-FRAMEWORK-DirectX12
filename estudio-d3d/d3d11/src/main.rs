//! Estudio: cubo 3D con Direct3D 11.
//!
//! cargo run -p estudio_d3d11 --release [-- --fotograma N | --medir N | --sin-vsync]

mod cubo_d3d11;

fn main() -> windows::core::Result<()> {
    estudio_base::ejecutar("Estudio D3D11", cubo_d3d11::CuboD3d11::nuevo)
}
