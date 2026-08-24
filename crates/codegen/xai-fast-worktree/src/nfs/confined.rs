//! Fd-relative helpers + the single owned deleter used by daemon-down `rm`
//! and `clean-artifacts`. Never a weaker sibling of `grove_git::delete_owned`.
pub fn is_safe_worktree_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('.')
        && !id.contains('/')
        && !id.contains('\\')
        && !id.contains('\0')
}

#[cfg(test)]
pub(crate) mod tests {
    use std::path::Path;

    const DAEMON_DB_FILE: &str = "daemon.db";

    /// Plant an in-flight create journal row (`wt_create_state`) for
    /// `worktree_id` in the daemon DB under `data`, so remove/GC paths see a
    /// create that has not completed. The `backing` path and optional payload
    /// are accepted for symmetry with callers; only the row is written.
    pub(crate) fn plant_journal(
        data: &Path,
        worktree_id: &str,
        _backing: &Path,
        _extra: Option<serde_json::Value>,
    ) {
        std::fs::create_dir_all(data).unwrap();
        let conn = rusqlite::Connection::open(data.join(DAEMON_DB_FILE)).unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS wt_create_state (
                worktree_id TEXT PRIMARY KEY,
                phase TEXT NOT NULL,
                dest TEXT NOT NULL,
                source TEXT NOT NULL,
                orphan_seen_at INTEGER,
                updated_at INTEGER NOT NULL
            );",
        )
        .unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO wt_create_state(worktree_id, phase, dest, source, updated_at)
             VALUES (?1, 'pinned', '', '', 1)",
            rusqlite::params![worktree_id],
        )
        .unwrap();
    }
}
