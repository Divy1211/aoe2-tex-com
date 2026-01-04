use pyo3::prelude::*;

#[pyclass]
#[derive(Clone, Debug)]
pub enum BcFormat {
    Bc1,
    Bc4,
    Bc7,
}

#[pyclass]
#[derive(Clone, Debug)]
pub enum BcQuality {
    Fast,
    Normal,
    Slow
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct DrawCall {
    #[pyo3(get)]
    pub skip: u8,
    #[pyo3(get)]
    pub draw: u8,
}

#[pymethods]
impl DrawCall {
    #[new]
    pub fn new(skip: u8, draw: u8) -> Self {
        Self { skip, draw }
    }
}