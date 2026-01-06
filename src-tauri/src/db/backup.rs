use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

const MAX_BACKUPS: usize = 7;
const BACKUP_INTERVAL_SECS: u64 = 24 * 60 * 60; // 24 hours

/// Get the backup directory path
pub fn get_backup_dir(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join("backups")
}

/// Get timestamp of last backup, returns 0 if no backups exist
fn get_last_backup_time(backup_dir: &PathBuf) -> u64 {
    if !backup_dir.exists() {
        return 0;
    }

    fs::read_dir(backup_dir)
        .ok()
        .and_then(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "db"))
                .filter_map(|e| {
                    e.file_name()
                        .to_str()
                        .and_then(|name| name.strip_prefix("flashnotes_"))
                        .and_then(|name| name.strip_suffix(".db"))
                        .and_then(|ts| ts.parse::<u64>().ok())
                })
                .max()
        })
        .unwrap_or(0)
}

/// Check if backup is needed (more than 24h since last backup)
pub fn needs_backup(app_data_dir: &PathBuf) -> bool {
    let backup_dir = get_backup_dir(app_data_dir);
    let last_backup = get_last_backup_time(&backup_dir);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    now - last_backup >= BACKUP_INTERVAL_SECS
}

/// Create a backup using VACUUM INTO
pub fn create_backup(conn: &Connection, app_data_dir: &PathBuf) -> Result<PathBuf, String> {
    let backup_dir = get_backup_dir(app_data_dir);

    // Ensure backup directory exists
    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let backup_path = backup_dir.join(format!("flashnotes_{}.db", timestamp));
    let backup_path_str = backup_path.to_string_lossy();

    info!("Creating database backup: {}", backup_path_str);

    conn.execute(&format!("VACUUM INTO '{}'", backup_path_str), [])
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    info!("Backup created successfully");

    // Cleanup old backups
    cleanup_old_backups(&backup_dir);

    Ok(backup_path)
}

/// Remove backups older than MAX_BACKUPS
fn cleanup_old_backups(backup_dir: &PathBuf) {
    let mut backups: Vec<_> = fs::read_dir(backup_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "db"))
        .filter_map(|e| {
            let path = e.path();
            let name = e.file_name();
            let ts = name
                .to_str()
                .and_then(|n| n.strip_prefix("flashnotes_"))
                .and_then(|n| n.strip_suffix(".db"))
                .and_then(|ts| ts.parse::<u64>().ok())?;
            Some((path, ts))
        })
        .collect();

    // Sort by timestamp descending (newest first)
    backups.sort_by(|a, b| b.1.cmp(&a.1));

    // Remove backups beyond MAX_BACKUPS
    for (path, _) in backups.into_iter().skip(MAX_BACKUPS) {
        if let Err(e) = fs::remove_file(&path) {
            warn!("Failed to remove old backup {:?}: {}", path, e);
        } else {
            info!("Removed old backup: {:?}", path);
        }
    }
}

/// Create pre-migration backup
#[allow(dead_code)]
pub fn create_migration_backup(conn: &Connection, app_data_dir: &PathBuf) -> Result<PathBuf, String> {
    let backup_dir = get_backup_dir(app_data_dir);

    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let backup_path = backup_dir.join(format!("flashnotes_premigration_{}.db", timestamp));
    let backup_path_str = backup_path.to_string_lossy();

    info!("Creating pre-migration backup: {}", backup_path_str);

    conn.execute(&format!("VACUUM INTO '{}'", backup_path_str), [])
        .map_err(|e| format!("Failed to create migration backup: {}", e))?;

    Ok(backup_path)
}
