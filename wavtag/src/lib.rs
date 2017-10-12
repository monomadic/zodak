extern crate byteorder;

mod riff;
pub use riff::*;

pub mod utils;

mod chunks;
pub use chunks::*;

mod midi;
pub use midi::*;
