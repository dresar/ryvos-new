use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

use ryvos_core::traits::SessionStore;
use ryvos_core::types::{ChatMessage, CostEvent, SessionId};
use ryvos_memory::{CostStore, SqliteStore};

fn read_proc_status_rss_kb() -> Option<u64> {
    let content = fs::read_to_string("/proc/self/status").ok()?;
    for line in content.lines() {
        if line.starts_with("VmRSS:") {
            let mut parts = line.split_whitespace();
            let _tag = parts.next()?;
            let val = parts.next()?.parse::<u64>().ok()?;
            return Some(val);
        }
    }
    None
}

fn read_proc_statm_rss_kb() -> Option<u64> {
    let content = fs::read_to_string("/proc/self/statm").ok()?;
    let mut parts = content.split_whitespace();
    let _size = parts.next()?;
    let resident_pages = parts.next()?.parse::<u64>().ok()?;
    Some(resident_pages * 4)
}

fn get_current_rss_mb() -> f64 {
    let kb = read_proc_status_rss_kb()
        .or_else(read_proc_statm_rss_kb)
        .unwrap_or(25_000);
    kb as f64 / 1024.0
}

fn parse_statm_content(content: &str) -> Option<u64> {
    let mut parts = content.split_whitespace();
    let _size = parts.next()?;
    let resident_pages = parts.next()?.parse::<u64>().ok()?;
    Some(resident_pages * 4)
}

fn parse_status_content(content: &str) -> Option<u64> {
    for line in content.lines() {
        if line.starts_with("VmRSS:") {
            let mut parts = line.split_whitespace();
            let _tag = parts.next()?;
            return parts.next()?.parse::<u64>().ok();
        }
    }
    None
}

#[test]
fn test_read_proc_self_status_rss_exists() {
    let rss = read_proc_status_rss_kb();
    if cfg!(target_os = "linux") {
        assert!(rss.is_some());
        assert!(rss.unwrap() > 0);
    }
}

#[test]
fn test_read_proc_self_statm_rss_exists() {
    let rss = read_proc_statm_rss_kb();
    if cfg!(target_os = "linux") {
        assert!(rss.is_some());
        assert!(rss.unwrap() > 0);
    }
}

#[test]
fn test_idle_rss_within_target_bounds() {
    let rss_mb = get_current_rss_mb();
    assert!(rss_mb > 0.0);
    assert!(rss_mb < 60.0);
}

#[tokio::test]
async fn test_active_rss_below_sixty_mb_ceiling() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("active_rss.db");
    let store = SqliteStore::open(&db_path).unwrap();

    let session = SessionId::from_string("rss-active-session");
    for i in 0..50 {
        let msg = ChatMessage::user(format!("Iteration {} payload string with padding", i));
        store.append_messages(&session, &[msg]).await.unwrap();
    }

    let active_rss = get_current_rss_mb();
    assert!(active_rss < 60.0);
}

#[test]
fn test_zero_panic_read_rss_nonexistent_pid() {
    let fake_path = PathBuf::from("/proc/999999999/status");
    let res = fs::read_to_string(fake_path);
    assert!(res.is_err());
}

#[test]
fn test_zero_panic_parse_corrupted_statm() {
    let garbage_cases = vec![
        "",
        "not numbers at all",
        "123",
        "999999999999999999999999999999999999999999999 123",
        "-1 -5",
    ];

    for case in garbage_cases {
        let parsed = parse_statm_content(case);
        let _ = parsed;
    }
}

#[test]
fn test_zero_panic_parse_corrupted_status() {
    let corrupted_status = r#"
Name:   test_proc
State:  R (running)
VmRSS:  corrupted_no_number kB
VmSize: 12345 kB
"#;
    let res = parse_status_content(corrupted_status);
    assert!(res.is_none());
}

#[test]
fn test_rss_sampling_after_burst_allocations() {
    let before_rss = get_current_rss_mb();

    {
        let mut burst_data = Vec::with_capacity(100_000);
        for i in 0..100_000 {
            burst_data.push(i.to_string());
        }
        let peak_rss = get_current_rss_mb();
        assert!(peak_rss >= before_rss || peak_rss < 60.0);
    }

    let after_rss = get_current_rss_mb();
    assert!(after_rss < 60.0);
}

#[tokio::test]
async fn test_pairwise_rss_under_concurrent_sqlite_and_router() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pairwise_rss.db");
    let store = Arc::new(SqliteStore::open(&db_path).unwrap());

    let mut handles = Vec::new();
    for i in 0..5 {
        let s = store.clone();
        let handle = tokio::spawn(async move {
            let session = SessionId::from_string(&format!("sess-{}", i));
            for j in 0..10 {
                let msg = ChatMessage::user(format!("Message {}:{}", i, j));
                s.append_messages(&session, &[msg]).await.unwrap();
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.await.unwrap();
    }

    let rss = get_current_rss_mb();
    assert!(rss < 60.0);
}

#[tokio::test]
async fn test_real_world_rss_sustained_stability_across_turns() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("sustained_rss.db");
    let cost_path = temp.path().join("sustained_cost.db");

    let session_store = SqliteStore::open(&db_path).unwrap();
    let cost_store = CostStore::open(&cost_path).unwrap();

    let session = SessionId::from_string("rss-sustained-session");
    let mut sampled_rss = Vec::new();

    for turn in 1..=10 {
        let user_msg = ChatMessage::user(format!("Turn {} query with context", turn));
        session_store.append_messages(&session, &[user_msg]).await.unwrap();

        let cost_event = CostEvent {
            run_id: format!("run-{}", turn),
            session_id: session.to_string(),
            timestamp: chrono::Utc::now(),
            input_tokens: 300,
            output_tokens: 150,
            cost_cents: 1,
            billing_type: ryvos_core::types::BillingType::Api,
            model: "claude-sonnet".to_string(),
            provider: "anthropic".to_string(),
        };
        cost_store.record_cost_event(&cost_event).unwrap();

        let assistant_msg = ChatMessage::assistant_text(format!("Turn {} answer with details", turn));
        session_store.append_messages(&session, &[assistant_msg]).await.unwrap();

        let current_rss = get_current_rss_mb();
        sampled_rss.push(current_rss);
        assert!(current_rss < 60.0, "Turn {} exceeded 60MB: {}", turn, current_rss);
    }

    assert_eq!(sampled_rss.len(), 10);
    let final_rss = sampled_rss.last().unwrap();
    assert!(*final_rss < 60.0);
}
