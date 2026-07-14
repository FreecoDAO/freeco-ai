//! Encrypted, authenticated local recovery archives.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path, PathBuf};
use zeroize::Zeroizing;

const MAGIC: &[u8; 4] = b"OFB1";
const KEY_NAME: &str = "openfang.backup.archive-key.v1";
const MAX_ARCHIVE_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct BackupFile {
    path: String,
    contents: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct BackupPayload {
    version: u8,
    created_at: String,
    files: Vec<BackupFile>,
    sha256: String,
}

#[derive(Debug, serde::Serialize)]
pub struct BackupResult {
    pub path: String,
    pub files: usize,
    pub bytes: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct RestoreResult {
    pub dry_run: bool,
    pub files: usize,
    pub staging_dir: Option<String>,
}

fn archive_key(home_dir: &Path) -> Result<Zeroizing<[u8; 32]>, String> {
    let vault_path = home_dir.join("vault.enc");
    let mut vault = openfang_extensions::vault::CredentialVault::new(vault_path.clone());
    if vault_path.exists() {
        vault
            .unlock()
            .map_err(|e| format!("unable to unlock vault: {e}"))?;
    } else {
        vault
            .init()
            .map_err(|e| format!("unable to initialize vault: {e}"))?;
    }

    if let Some(encoded) = vault.get(KEY_NAME) {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded.as_bytes())
            .map_err(|_| "invalid backup key in vault".to_string())?;
        return bytes
            .as_slice()
            .try_into()
            .map(Zeroizing::new)
            .map_err(|_| "invalid backup key length in vault".to_string());
    }

    let mut key = Zeroizing::new([0u8; 32]);
    rand::rng().fill_bytes(key.as_mut());
    let encoded = base64::engine::general_purpose::STANDARD.encode(key.as_ref());
    vault
        .set(KEY_NAME.to_string(), Zeroizing::new(encoded))
        .map_err(|e| format!("unable to protect backup key in vault: {e}"))?;
    Ok(key)
}

fn add_file(files: &mut Vec<BackupFile>, root: &Path, path: PathBuf) -> Result<(), String> {
    if !path.is_file() {
        return Ok(());
    }
    let relative = path
        .strip_prefix(root)
        .map_err(|_| "backup path escaped home directory".to_string())?;
    let bytes = fs::read(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    files.push(BackupFile {
        path: relative.to_string_lossy().to_string(),
        contents: base64::engine::general_purpose::STANDARD.encode(bytes),
    });
    Ok(())
}

fn collect_files(
    home_dir: &Path,
    data_dir: &Path,
    db_path: &Path,
) -> Result<Vec<BackupFile>, String> {
    let mut files = Vec::new();
    add_file(&mut files, home_dir, home_dir.join("config.toml"))?;
    add_file(&mut files, home_dir, home_dir.join("vault.enc"))?;
    for suffix in ["", "-wal", "-shm"] {
        add_file(
            &mut files,
            home_dir,
            PathBuf::from(format!("{}{}", db_path.display(), suffix)),
        )?;
    }
    for directory in [
        home_dir.join("workflows"),
        home_dir.join("agents"),
        data_dir.join("charts"),
    ] {
        if let Ok(entries) = fs::read_dir(directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    add_file(&mut files, home_dir, path)?;
                } else if path.is_dir() {
                    add_file(&mut files, home_dir, path.join("agent.toml"))?;
                }
            }
        }
    }
    Ok(files)
}

pub fn create_backup(
    home_dir: &Path,
    data_dir: &Path,
    db_path: &Path,
    retention: usize,
) -> Result<BackupResult, String> {
    let key = archive_key(home_dir)?;
    let files = collect_files(home_dir, data_dir, db_path)?;
    let content = serde_json::to_vec(&files).map_err(|e| e.to_string())?;
    let payload = BackupPayload {
        version: 1,
        created_at: chrono::Utc::now().to_rfc3339(),
        files,
        sha256: hex::encode(Sha256::digest(&content)),
    };
    let plaintext = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    let cipher = Aes256Gcm::new_from_slice(key.as_ref()).map_err(|e| e.to_string())?;
    let mut nonce = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
        .map_err(|_| "backup encryption failed".to_string())?;
    let backups = home_dir.join("backups");
    fs::create_dir_all(&backups).map_err(|e| e.to_string())?;
    let path = backups.join(format!(
        "openfang-{}-{}.ofbak",
        chrono::Utc::now().format("%Y%m%dT%H%M%SZ"),
        uuid::Uuid::new_v4()
    ));
    let mut archive = MAGIC.to_vec();
    archive.extend_from_slice(&nonce);
    archive.extend_from_slice(&ciphertext);
    fs::write(&path, &archive).map_err(|e| e.to_string())?;
    prune_backups(&backups, retention)?;
    Ok(BackupResult {
        path: path.display().to_string(),
        files: payload.files.len(),
        bytes: archive.len() as u64,
    })
}

fn prune_backups(backups: &Path, retention: usize) -> Result<(), String> {
    let mut archives: Vec<_> = fs::read_dir(backups)
        .map_err(|e| e.to_string())?
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "ofbak"))
        .collect();
    archives.sort_by_key(|entry| entry.metadata().and_then(|m| m.modified()).ok());
    while archives.len() > retention.max(1) {
        let old = archives.remove(0);
        fs::remove_file(old.path()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn read_payload(home_dir: &Path, archive: &Path) -> Result<BackupPayload, String> {
    let metadata = fs::metadata(archive).map_err(|e| e.to_string())?;
    if metadata.len() > MAX_ARCHIVE_BYTES {
        return Err("backup archive exceeds maximum size".to_string());
    }
    let encrypted = fs::read(archive).map_err(|e| e.to_string())?;
    if encrypted.len() < MAGIC.len() + 12 || &encrypted[..4] != MAGIC {
        return Err("invalid backup archive".to_string());
    }
    let key = archive_key(home_dir)?;
    let cipher = Aes256Gcm::new_from_slice(key.as_ref()).map_err(|e| e.to_string())?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&encrypted[4..16]), &encrypted[16..])
        .map_err(|_| "backup authentication failed".to_string())?;
    let payload: BackupPayload = serde_json::from_slice(&plaintext).map_err(|e| e.to_string())?;
    let content = serde_json::to_vec(&payload.files).map_err(|e| e.to_string())?;
    if hex::encode(Sha256::digest(&content)) != payload.sha256 {
        return Err("backup integrity verification failed".to_string());
    }
    Ok(payload)
}

fn safe_relative_path(path: &str) -> bool {
    !Path::new(path).is_absolute()
        && Path::new(path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

/// Verify an archive, or safely materialize it in a staging directory. A
/// daemon must be stopped before an operator promotes staged database files.
pub fn restore_backup(
    home_dir: &Path,
    archive: &Path,
    dry_run: bool,
) -> Result<RestoreResult, String> {
    let payload = read_payload(home_dir, archive)?;
    if payload
        .files
        .iter()
        .any(|file| !safe_relative_path(&file.path))
    {
        return Err("backup contains an unsafe path".to_string());
    }
    if dry_run {
        return Ok(RestoreResult {
            dry_run: true,
            files: payload.files.len(),
            staging_dir: None,
        });
    }
    let staging = home_dir
        .join("restore-staging")
        .join(uuid::Uuid::new_v4().to_string());
    let files = payload.files.len();
    for file in payload.files {
        let destination = staging.join(&file.path);
        let parent = destination.parent().ok_or("invalid backup path")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(file.contents)
            .map_err(|_| "backup contains invalid file encoding".to_string())?;
        fs::write(destination, bytes).map_err(|e| e.to_string())?;
    }
    Ok(RestoreResult {
        dry_run: false,
        files,
        staging_dir: Some(staging.display().to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal() {
        assert!(!safe_relative_path("../config.toml"));
        assert!(safe_relative_path("workflows/a.json"));
    }
}
