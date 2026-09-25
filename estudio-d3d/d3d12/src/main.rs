//! BMOX-12: cubo 3D con Direct3D 12.
//!
//! cargo run -p bmox12 --release

mod bmox12;

fn main() -> windows::core::Result<()> {
    bmox_base::ejecutar("BMOX-12 Cube (Direct3D 12)", bmox12::Bmox12::nuevo)
}
