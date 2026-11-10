//! Direct Fortran C ABI boundary. No executable is launched.
#[allow(unsafe_code)]
mod native;
pub use native::run;
