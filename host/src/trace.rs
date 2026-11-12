#![allow(dead_code)] // fields are consumed by the analysis stage later

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Event {
    pub t: u64,
    pub cpu: u32,
    pub class: String,
    pub name: String,
    pub pid: Option<u32>,
    pub tid: Option<u32>,
    pub priority: Option<u32>,
    pub rcvid: Option<u32>,
    pub label: Option<String>,
}

#[derive(Debug, Default)]
pub struct Trace {
    pub cycles_per_sec: Option<u64>,
    pub events: Vec<Event>,
    pub thread_names: HashMap<(u32, u32), String>,
}

fn parse_num(s: &str) -> Option<u64> {
    let s = s.split('(').next()?;
    match s.strip_prefix("0x") {
        Some(h) => u64::from_str_radix(h, 16).ok(),
        None => s.parse().ok(),
    }
}

fn parse_event_line(line: &str) -> Option<Event> {
    let mut it = line.split_whitespace();
    let t = parse_num(it.next()?.strip_prefix("t:")?)?;
    let cpu = parse_num(it.next()?.strip_prefix("CPU:")?)? as u32;
    let class = it.next()?.to_string();
    let name = it.next()?.strip_prefix(':')?.split('/').next()?.to_string();
    let mut e = Event { t, cpu, class, name, ..Default::default() };
    for tok in it {
        if let Some((k, v)) = tok.split_once(':') {
            let n = parse_num(v).map(|n| n as u32);
            match k {
                "pid" => e.pid = n,
                "tid" => e.tid = n,
                "priority" => e.priority = n,
                "rcvid" => e.rcvid = n,
                _ => {}
            }
        }
    }
    Some(e)
}

fn apply_continuation(e: &mut Event, line: &str) {
    if let Some(v) = line.strip_prefix("name:") {
        e.label = Some(v.to_string());
    } else if let Some(v) = line.strip_prefix("pid:") {
        if e.pid.is_none() {
            e.pid = parse_num(v.trim()).map(|n| n as u32);
        }
    } else if let Some(v) = line.strip_prefix("tid:") {
        if e.tid.is_none() {
            e.tid = parse_num(v.trim()).map(|n| n as u32);
        }
    }
}

pub fn parse(text: &str) -> Trace {
    let mut trace = Trace::default();
    for line in text.lines() {
        if line.starts_with("t:0x") {
            if let Some(e) = parse_event_line(line) {
                trace.events.push(e);
            }
        } else if let Some(v) = line.trim().strip_prefix("TRACE_CYCLES_PER_SEC::") {
            trace.cycles_per_sec = v.trim().parse().ok();
        } else if let Some(last) = trace.events.last_mut() {
            apply_continuation(last, line.trim());
        }
    }
    for e in &trace.events {
        if e.name == "PROCTHREAD_NAME" {
            if let (Some(p), Some(t), Some(l)) = (e.pid, e.tid, &e.label) {
                trace.thread_names.insert((p, t), l.clone());
            }
        }
    }
    trace
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
TRACE_CYCLES_PER_SEC:: 2995127162
t:0x44572a2d3 CPU:01 COMM            :SND_MESSAGE        rcvid:0x00020001 pid:380944
t:0x44572bcf3 CPU:01 THREAD          :THREPLY            pid:380944 tid:3 priority:50 policy:2
t:0x4780e15d7 CPU:00 PROCESS         :PROCTHREAD_NAME
                       pid:282642
                       tid:8
                      name:io-sock 163853 3
t:0x4457279b1 CPU:01 KER_CALL        :MSG_SENDV/11       coid:0x40000001 msg[0]:"hi" (0x00006968)
"#;

    #[test]
    fn reads_cycles_per_sec() {
        assert_eq!(parse(SAMPLE).cycles_per_sec, Some(2995127162));
    }

    #[test]
    fn reads_thread_event_fields() {
        let tr = parse(SAMPLE);
        assert_eq!(tr.events.len(), 4);
        let e = &tr.events[1];
        assert_eq!(e.name, "THREPLY");
        assert_eq!(e.t, 0x44572bcf3);
        assert_eq!((e.pid, e.tid, e.priority), (Some(380944), Some(3), Some(50)));
    }

    #[test]
    fn reads_rcvid_in_hex() {
        assert_eq!(parse(SAMPLE).events[0].rcvid, Some(0x00020001));
    }

    #[test]
    fn joins_multiline_thread_name() {
        let tr = parse(SAMPLE);
        assert_eq!(
            tr.thread_names.get(&(282642, 8)).map(|s| s.as_str()),
            Some("io-sock 163853 3")
        );
    }

    #[test]
    fn strips_kernel_call_number() {
        assert_eq!(parse(SAMPLE).events[3].name, "MSG_SENDV");
    }
}
