use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Message {
    pub info: f64,
}
