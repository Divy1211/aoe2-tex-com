mod format;
mod encode_decode;
mod helper;
mod pack;

mod bc1_transparency;
mod preprocess;

use pyo3::prelude::*;
use crate::format::{BcFormat, BcQuality};
use crate::encode_decode::{encode, decode};
use crate::preprocess::{preprocess_frames, ProcessedFrame};

#[pymodule]
#[pyo3(name = "aoe2_tex_com")]
fn aoe2_tex_com(pid: &Bound<PyModule>) -> PyResult<()> {
    pid.add_class::<BcFormat>()?;
    pid.add_class::<BcQuality>()?;
    pid.add_class::<ProcessedFrame>()?;

    pid.add_function(wrap_pyfunction!(encode, pid)?)?;
    pid.add_function(wrap_pyfunction!(decode, pid)?)?;
    pid.add_function(wrap_pyfunction!(preprocess_frames, pid)?)?;
    
    Ok(())
}