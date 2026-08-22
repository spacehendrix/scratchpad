//! The lifecycle policy that makes Scratchpad ephemeral.
//!
//! - Archive: non-pinned docs untouched for 30 days leave the browse list
//!   (reversible — editing un-archives).
//! - Delete by age: non-pinned docs untouched for 3 years are removed.
//! - Delete by space: past the 5 GB cap, evict to 4.5 GB (hysteresis) —
//!   archived docs oldest-first, then active non-pinned oldest-first.
//! - Pinned documents are exempt from every destructive pass.
//! - Clock-skew guard: when system time appears to have regressed more than
//!   a day behind the newest known timestamp, destructive passes are skipped
//!   for the run (archival is reversible and cannot trigger on a regressed
//!   clock anyway, since the age computation goes negative).

use crate::core::error::CoreResult;
use crate::core::model::{DocMeta, RetentionReport, UnixMs};
use crate::core::state::Session;

const DAY_MS: i64 = 24 * 60 * 60 * 1000;
pub const BROWSE_WINDOW_MS: i64 = 30 * DAY_MS;
pub const MAX_AGE_MS: i64 = 3 * 365 * DAY_MS;
pub const STORAGE_LIMIT_BYTES: u64 = 5 * 1024 * 1024 * 1024;
pub const STORAGE_TARGET_BYTES: u64 = STORAGE_LIMIT_BYTES - 512 * 1024 * 1024;
const SKEW_TOLERANCE_MS: i64 = DAY_MS;

pub fn run(session: &mut Session, now: UnixMs) -> CoreResult<RetentionReport> {
    run_with_limits(session, now, STORAGE_LIMIT_BYTES, STORAGE_TARGET_BYTES)
}

pub fn run_with_limits(
    session: &mut Session,
    now: UnixMs,
    limit_bytes: u64,
    target_bytes: u64,
) -> CoreResult<RetentionReport> {
    let mut report = RetentionReport::default();

    let max_known = session
        .catalog
        .iter()
        .map(|m| m.updated_at.max(m.created_at))
        .max()
        .unwrap_or(0);
    let skew_ok = now + SKEW_TOLERANCE_MS >= max_known;
    report.skipped_clock_skew = !skew_ok;

    // Archive pass (reversible).
    for i in 0..session.catalog.len() {
        let m = &session.catalog[i];
        if !m.pinned && m.archived_at.is_none() && now - m.updated_at > BROWSE_WINDOW_MS {
            let mut archived = m.clone();
            archived.archived_at = Some(now);
            session.store.update_meta(&session.key, &archived)?;
            session.catalog[i] = archived;
            report.archived += 1;
        }
    }

    if !skew_ok {
        return Ok(report);
    }

    // Delete by age.
    let doomed: Vec<String> = session
        .catalog
        .iter()
        .filter(|m| !m.pinned && now - m.updated_at > MAX_AGE_MS)
        .map(|m| m.id.clone())
        .collect();
    if !doomed.is_empty() {
        session.store.delete_many(&doomed)?;
        session.catalog.retain(|m| !doomed.contains(&m.id));
        report.deleted_by_age = doomed.len() as u32;
    }

    // Delete by space, in eviction order, until back under target (or only
    // pinned documents remain).
    loop {
        let size = session.store.db_size_bytes();
        if size <= limit_bytes {
            break;
        }
        let order = eviction_order(&session.catalog);
        if order.is_empty() {
            break; // only pinned left — surfaced via StorageStats::over_capacity
        }
        let needed = size.saturating_sub(target_bytes);
        let mut batch: Vec<String> = Vec::new();
        let mut freed_estimate: u64 = 0;
        for meta in &order {
            batch.push(meta.id.clone());
            freed_estimate += u64::from(meta.size_bytes);
            if freed_estimate >= needed {
                break;
            }
        }
        session.store.delete_many(&batch)?;
        session.catalog.retain(|m| !batch.contains(&m.id));
        report.deleted_by_space += batch.len() as u32;
        session.store.vacuum()?;
    }

    Ok(report)
}

/// Space-eviction order: archived first (oldest activity first), then active
/// non-pinned (oldest first). Pinned documents are never candidates.
pub fn eviction_order(catalog: &[DocMeta]) -> Vec<&DocMeta> {
    let mut candidates: Vec<&DocMeta> = catalog.iter().filter(|m| !m.pinned).collect();
    candidates.sort_by_key(|m| (m.archived_at.is_none(), m.updated_at));
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::keychain::InMemoryKeySource;
    use crate::core::state::AppState;

    const DAY: i64 = DAY_MS;

    fn unlocked(dir: &std::path::Path) -> AppState {
        let mut state = AppState::default();
        state.unlock(&InMemoryKeySource([1u8; 32]), dir).unwrap();
        state
    }

    #[test]
    fn archives_at_day_31_not_day_29() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = unlocked(dir.path());
        let s = state.session_mut().unwrap();
        let m = s.save(None, None, "note".into(), 0).unwrap();

        let r = run(s, 29 * DAY).unwrap();
        assert_eq!(r.archived, 0);
        assert_eq!(s.list().len(), 1);

        let r = run(s, 31 * DAY).unwrap();
        assert_eq!(r.archived, 1);
        assert!(s.list().is_empty());
        // Still in the store, just archived.
        assert_eq!(s.get(&m.id).unwrap().meta.archived_at, Some(31 * DAY));
    }

    #[test]
    fn pinned_never_archives_or_deletes() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = unlocked(dir.path());
        let s = state.session_mut().unwrap();
        let m = s.save(None, None, "keep me".into(), 0).unwrap();
        s.toggle_pin(&m.id).unwrap();

        let r = run(s, 10 * 365 * DAY).unwrap();
        assert_eq!(r.archived, 0);
        assert_eq!(r.deleted_by_age, 0);
        assert_eq!(s.list().len(), 1);
    }

    #[test]
    fn deletes_after_three_years() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = unlocked(dir.path());
        let s = state.session_mut().unwrap();
        let m = s.save(None, None, "old".into(), 0).unwrap();

        let r = run(s, 3 * 365 * DAY + DAY).unwrap();
        assert_eq!(r.deleted_by_age, 1);
        assert!(s.get(&m.id).is_err());
        assert!(s.catalog.is_empty());
    }

    #[test]
    fn edit_after_archive_revives_and_resets_clock() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = unlocked(dir.path());
        let s = state.session_mut().unwrap();
        let m = s.save(None, None, "note".into(), 0).unwrap();
        run(s, 40 * DAY).unwrap();
        assert!(s.list().is_empty());

        s.save(Some(m.id.clone()), None, "revived".into(), 41 * DAY).unwrap();
        assert_eq!(s.list().len(), 1);
        // Not re-archived until 30 further days pass.
        run(s, 60 * DAY).unwrap();
        assert_eq!(s.list().len(), 1);
        run(s, 72 * DAY).unwrap();
        assert!(s.list().is_empty());
    }

    #[test]
    fn clock_regression_skips_deletes_but_not_report() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = unlocked(dir.path());
        let s = state.session_mut().unwrap();
        s.save(None, None, "ancient".into(), 0).unwrap();
        s.save(None, None, "recent".into(), 10 * 365 * DAY).unwrap();

        // now = 5y, but newest timestamp is 10y: time regressed.
        let r = run(s, 5 * 365 * DAY).unwrap();
        assert!(r.skipped_clock_skew);
        assert_eq!(r.deleted_by_age, 0);
        // The "ancient" doc would otherwise be 5y past its window.
        assert_eq!(s.catalog.len(), 2);
    }

    #[test]
    fn eviction_order_archived_oldest_first_then_active_oldest() {
        let mk = |id: &str, updated: i64, pinned: bool, archived: Option<i64>| DocMeta {
            id: id.into(),
            title: None,
            preview: String::new(),
            created_at: 0,
            updated_at: updated,
            pinned,
            archived_at: archived,
            size_bytes: 1,
            checklist: None,
        };
        let catalog = vec![
            mk("active-old", 10, false, None),
            mk("pinned", 5, true, None),
            mk("archived-new", 20, false, Some(100)),
            mk("archived-old", 15, false, Some(90)),
            mk("active-new", 30, false, None),
        ];
        let order: Vec<&str> = eviction_order(&catalog).iter().map(|m| m.id.as_str()).collect();
        assert_eq!(
            order,
            vec!["archived-old", "archived-new", "active-old", "active-new"]
        );
    }

    #[test]
    fn space_eviction_deletes_non_pinned_until_under_limit() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = unlocked(dir.path());
        let s = state.session_mut().unwrap();
        let big = "x".repeat(50_000);
        let keep = s.save(None, None, format!("keep {big}"), 100).unwrap();
        s.toggle_pin(&keep.id).unwrap();
        for i in 0..5 {
            s.save(None, None, format!("bulk {i} {big}"), 200 + i).unwrap();
        }

        // Limit of 1 byte: everything evictable must go; pinned survives.
        let r = run_with_limits(s, 300, 1, 0).unwrap();
        assert_eq!(r.deleted_by_space, 5);
        assert_eq!(s.catalog.len(), 1);
        assert_eq!(s.catalog[0].id, keep.id);
        // Report knows nothing was age-deleted.
        assert_eq!(r.deleted_by_age, 0);
    }
}
