#[derive(Debug, Clone)]
pub struct Task {
    pub name: String,
    pub priority: u32, // higher number = higher priority, like QNX
    pub wcet_us: f64,
    pub period_us: f64,
    pub deadline_us: f64,
    pub blocking_us: f64,
}
