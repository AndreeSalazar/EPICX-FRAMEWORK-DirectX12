//! BMOX-11: cubo 3D con Direct3D 11.
//!
//! cargo run -p bmox11 --release

mod bmox11;

fn main() -> windows::core::Result<()> {
    bmox_base::ejecutar("BMOX-11 Cube (Direct3D 11)", bmox11::Bmox11::nuevo)
}
