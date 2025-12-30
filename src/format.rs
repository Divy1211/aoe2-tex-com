use pyo3::pyclass;

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