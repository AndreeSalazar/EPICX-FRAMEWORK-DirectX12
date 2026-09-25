//! BMOX-10: cubo 3D con Direct3D 10.
//!
//! cargo run -p bmox10 --release

mod bmox10;

fn main() -> windows::core::Result<()> {
    bmox_base::ejecutar("BMOX-10 Cube (Direct3D 10)", bmox10::Bmox10::nuevo)
}
