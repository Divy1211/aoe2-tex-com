mod format;
mod encode_decode;

use pyo3::prelude::*;
use crate::format::{BcFormat, BcQuality};
use crate::encode_decode::{encode, decode};

#[pymodule]
#[pyo3(name = "aoe2_tex_com")]
fn aoe2_tex_com(pid: &Bound<PyModule>) -> PyResult<()> {
    pid.add_class::<BcFormat>()?;
    pid.add_class::<BcQuality>()?;

    pid.add_function(wrap_pyfunction!(encode, pid)?)?;
    pid.add_function(wrap_pyfunction!(decode, pid)?)?;
    
    Ok(())
}