use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub priority: u32, // higher number = higher priority, like QNX
    pub wcet_us: f64,
    pub period_us: f64,
    pub deadline_us: f64,
    #[serde(default)]
    pub blocking_us: f64,
}

pub fn load_tasks(json: &str) -> Result<Vec<Task>, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_tasks_and_defaults_blocking() {
        let json = r#"[
            {"name":"arm","priority":50,"wcet_us":200,"period_us":1000,"deadline_us":1000},
            {"name":"log","priority":10,"wcet_us":500,"period_us":5000,"deadline_us":5000,"blocking_us":100}
        ]"#;
        let tasks = load_tasks(json).unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].blocking_us, 0.0);
        assert_eq!(tasks[1].blocking_us, 100.0);
    }

    #[test]
    fn rejects_bad_json() {
        assert!(load_tasks("not json").is_err());
    }
}
