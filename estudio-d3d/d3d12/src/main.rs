//! Estudio: cubo 3D con Direct3D 12.
//!
//! cargo run -p estudio_d3d12 --release [-- --fotograma N | --medir N | --sin-vsync]

mod cubo_d3d12;

fn main() -> windows::core::Result<()> {
    estudio_base::ejecutar("Estudio D3D12", cubo_d3d12::CuboD3d12::nuevo)
}
