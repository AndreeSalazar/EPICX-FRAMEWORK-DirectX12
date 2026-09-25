//! BMOX-9: cubo 3D con Direct3D 9.
//!
//! cargo run -p bmox9 --release

mod bmox9;

fn main() -> windows::core::Result<()> {
    bmox_base::ejecutar("BMOX-9 Cube (Direct3D 9)", bmox9::Bmox9::nuevo)
}
