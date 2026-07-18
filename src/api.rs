#[derive(Debug, Clone)]
pub struct Balance {
    pub total: f64,
    pub topped_up: f64,
    pub granted: f64,
    pub currency: String,
}
