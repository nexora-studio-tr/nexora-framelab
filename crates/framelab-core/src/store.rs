use crate::{config, kv, model::*};
use rusqlite::{params, Connection};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Clone)]
pub struct Store {
    pub dir: PathBuf,
}
fn db_error(_: rusqlite::Error) -> String {
    "STORAGE_ERROR".into()
}
impl Store {
    pub fn open(dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&dir).map_err(|_| "STORAGE_ERROR")?;
        let this = Self { dir };
        let c = this.connection()?;
        c.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS changes(id TEXT PRIMARY KEY,path TEXT NOT NULL,before BLOB NOT NULL,after BLOB NOT NULL,patches TEXT NOT NULL,phase TEXT NOT NULL,created INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS owned(id TEXT PRIMARY KEY,path TEXT NOT NULL,hash TEXT NOT NULL,phase TEXT NOT NULL); CREATE TABLE IF NOT EXISTS reports(id TEXT PRIMARY KEY,created INTEGER NOT NULL,body TEXT NOT NULL);").map_err(db_error)?;
        Ok(this)
    }
    fn connection(&self) -> Result<Connection> {
        let c = Connection::open(self.dir.join("framelab.sqlite3")).map_err(db_error)?;
        c.busy_timeout(Duration::from_secs(3)).map_err(db_error)?;
        c.execute_batch("PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;")
            .map_err(db_error)?;
        Ok(c)
    }
    fn phase(&self, id: &str, phase: &str) -> Result<()> {
        self.connection()?
            .execute(
                "UPDATE changes SET phase=?1 WHERE id=?2",
                params![phase, id],
            )
            .map_err(db_error)?;
        Ok(())
    }
    pub fn apply(&self, path: &Path, patches: &[Patch]) -> Result<String> {
        if fs2::available_space(&self.dir).map_err(|_| "STORAGE_ERROR")? < 128 * 1024 * 1024 {
            return Err("DISK_SPACE_LOW".into());
        }
        let path = config::safe_file(path)?;
        let before = config::read_video(&path)?;
        config::validate_patches(&before, patches)?;
        let after = kv::replace(&before, patches, false)?;
        let id = uuid::Uuid::new_v4().to_string();
        let c = self.connection()?;
        let active:i64=c.query_row("SELECT count(*) FROM changes WHERE path=?1 AND phase IN ('prepared','applied','conflict')",[path.to_string_lossy().as_ref()],|r|r.get(0)).map_err(db_error)?;
        if active != 0 {
            return Err("RECOVERY_REQUIRED".into());
        }
        c.execute(
            "INSERT INTO changes VALUES(?1,?2,?3,?4,?5,'prepared',?6)",
            params![
                id,
                path.to_string_lossy(),
                before.as_bytes(),
                after.as_bytes(),
                serde_json::to_string(patches).map_err(|_| "STORAGE_ERROR")?,
                now_ms()
            ],
        )
        .map_err(db_error)?;
        atomic_write(&path, after.as_bytes(), Some(before.as_bytes()))?;
        self.phase(&id, "applied")?;
        Ok(id)
    }
    pub fn commit(&self, id: &str) -> Result<()> {
        let c = self.connection()?;
        let count = c
            .execute(
                "UPDATE changes SET phase='committed' WHERE id=?1 AND phase='applied'",
                [id],
            )
            .map_err(db_error)?;
        if count != 1 {
            return Err("TRANSACTION_STATE_INVALID".into());
        }
        Ok(())
    }
    pub fn commit_report(&self, id: &str, report: &Report) -> Result<()> {
        let mut c = self.connection()?;
        let tx = c.transaction().map_err(db_error)?;
        let count = tx
            .execute(
                "UPDATE changes SET phase='committed' WHERE id=?1 AND phase='applied'",
                [id],
            )
            .map_err(db_error)?;
        if count != 1 {
            return Err("TRANSACTION_STATE_INVALID".into());
        }
        let body = serde_json::to_string(report).map_err(|_| "STORAGE_ERROR")?;
        tx.execute(
            "INSERT OR REPLACE INTO reports VALUES(?1,?2,?3)",
            params![report.id, report.created_at, body],
        )
        .map_err(db_error)?;
        tx.commit().map_err(db_error)?;
        Ok(())
    }
    pub fn revert(&self, id: &str) -> Result<()> {
        let c = self.connection()?;
        let row = c
            .query_row(
                "SELECT path,before,after,patches,phase FROM changes WHERE id=?1",
                [id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, Vec<u8>>(1)?,
                        r.get::<_, Vec<u8>>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                    ))
                },
            )
            .map_err(db_error)?;
        if row.4 == "reverted" {
            return Ok(());
        }
        let path = PathBuf::from(&row.0);
        let patches: Vec<Patch> = serde_json::from_str(&row.3).map_err(|_| "STORAGE_ERROR")?;
        let result = (|| {
            let current = config::read_video(&path)?;
            let restored = if current.as_bytes() == row.2 {
                String::from_utf8(row.1.clone()).map_err(|_| "STORAGE_ERROR")?
            } else if current.as_bytes() == row.1 {
                current.clone()
            } else {
                let version = kv::video(&current)?;
                if kv::text(&version, "Version") != Some(config::SUPPORTED_VERSION) {
                    return Err("CONFIG_CONFLICT".into());
                }
                kv::replace(&current, &patches, true)?
            };
            if restored != current {
                atomic_write(&path, restored.as_bytes(), Some(current.as_bytes()))?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => self.phase(id, "reverted"),
            Err(e) => {
                self.phase(id, "conflict")?;
                Err(e)
            }
        }
    }
    pub fn recover(&self) -> Result<Vec<Recovery>> {
        let pending = self.pending()?;
        for p in &pending {
            if matches!(p.status.as_str(), "prepared" | "applied") {
                let _ = self.revert(&p.id);
            }
        }
        self.clean_owned()?;
        self.pending()
    }
    pub fn pending(&self) -> Result<Vec<Recovery>> {
        let c = self.connection()?;
        let mut q=c.prepare("SELECT id,phase,patches FROM changes WHERE phase IN ('prepared','applied','conflict') ORDER BY created").map_err(db_error)?;
        let rows = q
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(db_error)?;
        let mut out = Vec::new();
        for row in rows {
            let (id, status, raw) = row.map_err(db_error)?;
            let patches: Vec<Patch> = serde_json::from_str(&raw).map_err(|_| "STORAGE_ERROR")?;
            out.push(Recovery {
                id,
                status,
                keys: patches.into_iter().map(|p| p.key).collect(),
            });
        }
        let mut q = c
            .prepare("SELECT id FROM owned WHERE phase='active'")
            .map_err(db_error)?;
        let rows = q
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(db_error)?;
        for row in rows {
            out.push(Recovery {
                id: format!("owned:{}", row.map_err(db_error)?),
                status: "owned".into(),
                keys: vec!["LOCAL_TELEMETRY_FILE".into()],
            });
        }
        Ok(out)
    }
    pub fn own_file(&self, path: &Path, bytes: &[u8]) -> Result<String> {
        if path.exists() {
            return Err("OWNED_FILE_EXISTS".into());
        }
        let parent = path.parent().ok_or("FILE_UNAVAILABLE")?;
        config::safe_file(parent)?;
        let id = uuid::Uuid::new_v4().to_string();
        self.connection()?
            .execute(
                "INSERT INTO owned VALUES(?1,?2,?3,'active')",
                params![id, path.to_string_lossy(), sha256(bytes)],
            )
            .map_err(db_error)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|_| "OWNED_FILE_WRITE_FAILED")?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|_| "OWNED_FILE_WRITE_FAILED")?;
        Ok(id)
    }
    pub fn clean_owned(&self) -> Result<()> {
        let c = self.connection()?;
        let mut q = c
            .prepare("SELECT id,path,hash FROM owned WHERE phase='active'")
            .map_err(db_error)?;
        let rows = q
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(db_error)?;
        for row in rows {
            let (id, path, hash) = row.map_err(db_error)?;
            let path = PathBuf::from(path);
            if path.exists() {
                config::safe_file(&path)?;
                let mut data = Vec::new();
                std::fs::File::open(&path)
                    .map_err(|_| "OWNED_FILE_CONFLICT")?
                    .take(1024 * 1024 + 1)
                    .read_to_end(&mut data)
                    .map_err(|_| "OWNED_FILE_CONFLICT")?;
                if data.len() > 1024 * 1024 {
                    return Err("OWNED_FILE_CONFLICT".into());
                }
                if sha256(&data) != hash {
                    return Err("OWNED_FILE_CONFLICT".into());
                }
                std::fs::remove_file(&path).map_err(|_| "OWNED_FILE_CONFLICT")?;
            }
            c.execute("UPDATE owned SET phase='removed' WHERE id=?1", [id])
                .map_err(db_error)?;
        }
        Ok(())
    }
    pub fn save_report(&self, report: &Report) -> Result<()> {
        let body = serde_json::to_string(report).map_err(|_| "STORAGE_ERROR")?;
        if body.len() > 2 * 1024 * 1024 {
            return Err("REPORT_TOO_LARGE".into());
        }
        self.connection()?
            .execute(
                "INSERT OR REPLACE INTO reports VALUES(?1,?2,?3)",
                params![report.id, report.created_at, body],
            )
            .map_err(db_error)?;
        Ok(())
    }
    pub fn reports(&self) -> Result<Vec<Report>> {
        let c = self.connection()?;
        let mut q = c
            .prepare("SELECT body FROM reports ORDER BY created DESC LIMIT 20")
            .map_err(db_error)?;
        let rows = q
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(db_error)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(serde_json::from_str(&row.map_err(db_error)?).map_err(|_| "STORAGE_ERROR")?);
        }
        Ok(out)
    }
    pub fn report(&self, id: &str) -> Result<Report> {
        let body: String = self
            .connection()?
            .query_row("SELECT body FROM reports WHERE id=?1", [id], |r| r.get(0))
            .map_err(db_error)?;
        serde_json::from_str(&body).map_err(|_| "STORAGE_ERROR".into())
    }
}
pub fn atomic_write(path: &Path, bytes: &[u8], expected: Option<&[u8]>) -> Result<()> {
    let parent = path.parent().ok_or("FILE_UNAVAILABLE")?;
    config::safe_file(parent)?;
    let tmp = parent.join(format!(".nexora-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(|_| "ATOMIC_WRITE_FAILED")?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|_| "ATOMIC_WRITE_FAILED")?;
        drop(file);
        if let Some(expected) = expected {
            if std::fs::read(path).map_err(|_| "CONFIG_CONFLICT")? != expected {
                return Err("CONFIG_CONFLICT".into());
            }
        }
        std::fs::rename(&tmp, path).map_err(|_| "ATOMIC_REPLACE_FAILED")?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(tmp);
    }
    result
}
