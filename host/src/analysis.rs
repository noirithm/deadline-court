use crate::model::Task;

/// Worst-case response time of task `i` under fixed-priority preemptive
/// scheduling: R = C + B + sum(ceil(R / Tj) * Cj) over higher-priority tasks.
/// Equal-priority tasks are counted as interference (conservative).
/// Returns None if R exceeds the deadline.
pub fn response_time(tasks: &[Task], i: usize) -> Option<f64> {
    let t = &tasks[i];
    let mut r = t.wcet_us + t.blocking_us;
    for _ in 0..1000 {
        let interference: f64 = tasks
            .iter()
            .enumerate()
            .filter(|(j, o)| *j != i && o.priority >= t.priority)
            .map(|(_, o)| (r / o.period_us).ceil() * o.wcet_us)
            .sum();
        let next = t.wcet_us + t.blocking_us + interference;
        if next > t.deadline_us {
            return None;
        }
        if (next - r).abs() < 1e-9 {
            return Some(next);
        }
        r = next;
    }
    None
}

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct Verdict {
    pub name: String,
    pub holds: bool,
    pub response_us: Option<f64>,
    pub slack_us: Option<f64>,
}

pub fn analyze_all(tasks: &[Task]) -> Vec<Verdict> {
    (0..tasks.len())
        .map(|i| {
            let r = response_time(tasks, i);
            Verdict {
                name: tasks[i].name.clone(),
                holds: r.is_some(),
                response_us: r,
                slack_us: r.map(|r| tasks[i].deadline_us - r),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(name: &str, prio: u32, c: f64, t: f64, d: f64) -> Task {
        Task {
            name: name.to_string(),
            priority: prio,
            wcet_us: c,
            period_us: t,
            deadline_us: d,
            blocking_us: 0.0,
        }
    }

    fn set() -> Vec<Task> {
        vec![
            task("a", 3, 1.0, 4.0, 4.0),
            task("b", 2, 2.0, 6.0, 6.0),
            task("c", 1, 3.0, 13.0, 13.0),
        ]
    }

    #[test]
    fn textbook_example() {
        let tasks = set();
        assert_eq!(response_time(&tasks, 0), Some(1.0));
        assert_eq!(response_time(&tasks, 1), Some(3.0));
        assert_eq!(response_time(&tasks, 2), Some(10.0));
    }

    #[test]
    fn tight_deadline_is_violated() {
        let mut tasks = set();
        tasks[2].deadline_us = 9.0;
        assert_eq!(response_time(&tasks, 2), None);
    }

    #[test]
    fn blocking_reduces_slack() {
        let mut tasks = set();
        tasks[0].blocking_us = 2.0;
        assert_eq!(analyze_all(&tasks)[0].slack_us, Some(1.0));
    }

    #[test]
    fn analyze_all_reports_each_task() {
        let tasks = set();
        let v = analyze_all(&tasks);
        assert_eq!(v.len(), 3);
        assert!(v.iter().all(|x| x.holds));
        assert_eq!(v[2].response_us, Some(10.0));
        assert_eq!(v[2].slack_us, Some(3.0));
    }
}
