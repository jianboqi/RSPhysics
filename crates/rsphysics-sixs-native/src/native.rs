use libloading::Library;
#[cfg(not(target_os = "macos"))]
use sha2::{Digest, Sha256};
#[cfg(not(target_os = "macos"))]
use std::sync::{Mutex, OnceLock};
use std::{
    collections::BTreeMap,
    ffi::CString,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
include!(concat!(env!("OUT_DIR"), "/runtime-assets.rs"));
#[cfg(not(target_os = "macos"))]
static LIBRARIES: OnceLock<Mutex<BTreeMap<String, Library>>> = OnceLock::new();
static COUNTER: AtomicU64 = AtomicU64::new(0);
struct Workspace(PathBuf);
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
pub fn run(
    program: &str,
    cards: &str,
    files: &BTreeMap<String, String>,
) -> Result<(String, BTreeMap<String, String>), String> {
    let bytes: &[u8] = match program {
        "sixsV2.1" => include_bytes!(env!("RSP_SIXS_LIBRARY")),
        "sixsV2.1-lutaero" => include_bytes!(env!("RSP_SIXS_LUT_LIBRARY")),
        _ => return Err("Unknown 6SV2.1 entry point".into()),
    };
    #[cfg(not(target_os = "macos"))]
    let hash = {
        let mut digest = Sha256::new();
        digest.update(bytes);
        for (name, content) in RUNTIME_ASSETS {
            digest.update(name.as_bytes());
            digest.update(content);
        }
        format!("{:x}", digest.finalize())
    };
    #[cfg(not(target_os = "macos"))]
    let cache = std::env::var_os("RSPHYSICS_KERNEL_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("rsphysics-kernels"))
        .join(hash);
    #[cfg(not(target_os = "macos"))]
    fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    let extension = std::env::consts::DLL_EXTENSION;
    #[cfg(not(target_os = "macos"))]
    let path = cache.join(format!("{program}.{extension}"));
    #[cfg(not(target_os = "macos"))]
    let libraries = LIBRARIES.get_or_init(|| Mutex::new(BTreeMap::new()));
    #[cfg(not(target_os = "macos"))]
    let mut libraries = libraries.lock().map_err(|e| e.to_string())?;
    #[cfg(not(target_os = "macos"))]
    if !libraries.contains_key(program) {
        for (name, content) in RUNTIME_ASSETS {
            materialize(&cache.join(name), content)?;
        }
        if fs::read(&path).ok().as_deref() != Some(bytes) {
            let temporary = cache.join(format!(
                "kernel-{}-{}.tmp",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
            if let Err(error) = fs::rename(&temporary, &path) {
                let _ = fs::remove_file(&temporary);
                if fs::read(&path).ok().as_deref() != Some(bytes) {
                    return Err(error.to_string());
                }
            }
        }
        // SAFETY: bytes are the library compiled from the pinned source during this
        // build; the stored Library outlives its symbol and every call.
        let library = unsafe { Library::new(&path) }.map_err(|e| e.to_string())?;
        libraries.insert(program.into(), library);
    }
    let temp = std::env::temp_dir().join(format!(
        "rsphysics-sixs-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&temp).map_err(|e| e.to_string())?;
    let work = Workspace(temp);
    #[cfg(target_os = "macos")]
    let library = {
        for (name, content) in RUNTIME_ASSETS {
            fs::write(work.0.join(name), content).map_err(|e| e.to_string())?;
        }
        let image = work.0.join(format!("{program}.{extension}"));
        fs::write(&image, bytes).map_err(|e| e.to_string())?;
        // SAFETY: a private image of the build-controlled library has its own
        // COMMON/SAVE storage. It is unloaded after this call, before cleanup.
        unsafe { Library::new(&image) }.map_err(|e| e.to_string())?
    };
    for (name, content) in files {
        if name.is_empty()
            || name.contains(['/', '\\', ':'])
            || name.contains("..")
            || matches!(name.as_str(), "input.cards" | "report.txt" | "error.txt")
        {
            return Err("Invalid auxiliary filename".into());
        }
        fs::write(work.0.join(name), content).map_err(|e| e.to_string())?;
    }
    fs::write(work.0.join("input.cards"), cards).map_err(|e| e.to_string())?;
    let directory = CString::new(work.0.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?;
    // SAFETY: the build-controlled C function takes one NUL-terminated directory
    // string, retains no caller pointer, and serializes/restores model state.
    let status = unsafe {
        #[cfg(not(target_os = "macos"))]
        let library = &libraries[program];
        let function = library
            .get::<unsafe extern "C" fn(*const std::ffi::c_char) -> i32>(b"rsp_sixs_run\0")
            .map_err(|e| e.to_string())?;
        function(directory.as_ptr())
    };
    let report = fs::read_to_string(work.0.join("report.txt")).unwrap_or_default();
    if status != 0 {
        return Err(format!(
            "6SV2.1 status {status}: {} {}",
            fs::read_to_string(work.0.join("error.txt")).unwrap_or_default(),
            report.chars().take(300).collect::<String>()
        ));
    }
    let mut outputs = BTreeMap::new();
    for file in fs::read_dir(&work.0).map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;
        let name = file.file_name().to_string_lossy().to_string();
        if file.file_type().map_err(|e| e.to_string())?.is_file()
            && !matches!(name.as_str(), "input.cards" | "report.txt" | "error.txt")
        {
            outputs.insert(
                name,
                fs::read_to_string(file.path()).map_err(|e| e.to_string())?,
            );
        }
    }
    Ok((report, outputs))
}

#[cfg(not(target_os = "macos"))]
fn materialize(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    if fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(());
    }
    let temporary = path.with_extension(format!(
        "{}-{}.tmp",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        if fs::read(path).ok().as_deref() != Some(bytes) {
            return Err(error.to_string());
        }
    }
    Ok(())
}
