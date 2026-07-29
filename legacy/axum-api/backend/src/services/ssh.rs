//! Managed SSH public keys for CaseFlow tunnel access.
//!
//! Keys are stored one-per-line as: `<public_key> # caseflow:<username>`
//! Default file: `~/projects/cms/deploy/ssh/caseflow_authorized_keys`
//! Override with env `SSH_AUTHORIZED_KEYS_PATH`.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use parking_lot::Mutex;

use crate::error::{AppError, AppResult};

static KEYS_LOCK: Mutex<()> = Mutex::new(());

const COMMENT_PREFIX: &str = "caseflow:";

#[derive(Debug, Clone, serde::Serialize)]
pub struct SshKeyEntry {
    pub username: String,
    pub public_key: String,
    pub comment: String,
}

pub struct SshKeyService;

impl SshKeyService {
    pub fn keys_path() -> PathBuf {
        if let Ok(p) = std::env::var("SSH_AUTHORIZED_KEYS_PATH") {
            if !p.trim().is_empty() {
                return PathBuf::from(p);
            }
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join("deploy/ssh/caseflow_authorized_keys")
    }

    fn ensure_parent(path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Internal(format!("create ssh keys dir: {e}")))?;
        }
        Ok(())
    }

    fn validate_public_key(key: &str) -> AppResult<()> {
        let key = key.trim();
        if key.is_empty() || key.contains('\n') || key.contains('\r') {
            return Err(AppError::BadRequest("invalid public key".into()));
        }
        let ok = key.starts_with("ssh-rsa ")
            || key.starts_with("ssh-ed25519 ")
            || key.starts_with("ecdsa-sha2-")
            || key.starts_with("sk-ssh-ed25519@")
            || key.starts_with("sk-ecdsa-sha2-");
        if !ok {
            return Err(AppError::BadRequest(
                "public key must be ssh-ed25519, ssh-rsa, or ecdsa".into(),
            ));
        }
        let parts: Vec<&str> = key.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(AppError::BadRequest("malformed public key".into()));
        }
        Ok(())
    }

    fn normalize_key(key: &str) -> String {
        let parts: Vec<&str> = key.trim().split_whitespace().collect();
        format!("{} {}", parts[0], parts[1])
    }

    fn parse_line(line: &str) -> Option<SshKeyEntry> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        let (key_part, comment) = match line.rsplit_once(" # ") {
            Some((k, c)) => (k.trim(), c.trim()),
            None => (line, ""),
        };
        let username = comment
            .strip_prefix(COMMENT_PREFIX)
            .unwrap_or(comment)
            .trim()
            .to_string();
        if username.is_empty() {
            return None;
        }
        Some(SshKeyEntry {
            username,
            public_key: key_part.to_string(),
            comment: comment.to_string(),
        })
    }

    pub fn list() -> AppResult<Vec<SshKeyEntry>> {
        let _g = KEYS_LOCK.lock();
        let path = Self::keys_path();
        if !path.exists() {
            return Ok(vec![]);
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| AppError::Internal(format!("read ssh keys: {e}")))?;
        Ok(content.lines().filter_map(Self::parse_line).collect())
    }

    pub fn add(username: &str, public_key: &str) -> AppResult<SshKeyEntry> {
        let username = username.trim();
        if username.is_empty() || username.chars().any(|c| c.is_whitespace() || c == '#') {
            return Err(AppError::BadRequest("invalid username".into()));
        }
        Self::validate_public_key(public_key)?;
        let normalized = Self::normalize_key(public_key);

        let _g = KEYS_LOCK.lock();
        let path = Self::keys_path();
        Self::ensure_parent(&path)?;

        let entries = if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| AppError::Internal(format!("read ssh keys: {e}")))?;
            content.lines().filter_map(Self::parse_line).collect::<Vec<_>>()
        } else {
            vec![]
        };

        if entries.iter().any(|e| e.username == username) {
            return Err(AppError::Conflict(format!(
                "SSH key already registered for {username}"
            )));
        }
        if entries.iter().any(|e| e.public_key == normalized) {
            return Err(AppError::Conflict(
                "this public key is already registered".into(),
            ));
        }

        let entry = SshKeyEntry {
            username: username.to_string(),
            public_key: normalized,
            comment: format!("{COMMENT_PREFIX}{username}"),
        };
        let line = format!("{} # {}\n", entry.public_key, entry.comment);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| AppError::Internal(format!("open ssh keys: {e}")))?;
        file.write_all(line.as_bytes())
            .map_err(|e| AppError::Internal(format!("write ssh keys: {e}")))?;

        Ok(entry)
    }

    pub fn remove(username: &str) -> AppResult<()> {
        let username = username.trim();
        let _g = KEYS_LOCK.lock();
        let path = Self::keys_path();
        if !path.exists() {
            return Err(AppError::NotFound("no SSH keys file".into()));
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| AppError::Internal(format!("read ssh keys: {e}")))?;
        let marker = format!("# {COMMENT_PREFIX}{username}");
        let mut kept = Vec::new();
        let mut removed = false;
        for line in content.lines() {
            if line.contains(&marker) {
                removed = true;
                continue;
            }
            kept.push(line);
        }
        if !removed {
            return Err(AppError::NotFound(format!(
                "no SSH key for user {username}"
            )));
        }
        let mut out = kept.join("\n");
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        fs::write(&path, out).map_err(|e| AppError::Internal(format!("write ssh keys: {e}")))?;
        Ok(())
    }
}
