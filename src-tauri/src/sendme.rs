//! Command line arguments.

use std::{
    fmt::{Display, Formatter},
    net::{SocketAddrV4, SocketAddrV6},
    path::{Component, Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use clap::{Parser, Subcommand};
use data_encoding::HEXLOWER;
use futures_buffered::BufferedStreamExt;
use indicatif::{HumanBytes, HumanDuration};
use iroh::{
    address_lookup::{dns::DnsAddressLookup, pkarr::PkarrPublisher},
    endpoint::presets,
    Endpoint, EndpointAddr, RelayMap, RelayMode, RelayUrl, SecretKey, TransportAddr,
};
use iroh_blobs::{
    api::{
        blobs::{AddPathOptions, AddProgressItem, ExportMode, ExportOptions, ImportMode},
        remote::GetProgressItem,
        Store, TempTag,
    },
    format::collection::Collection,
    get::{request::get_hash_seq_and_sizes, GetError, Stats},
    provider::{
        self,
        events::{ConnectMode, EventMask, EventSender, ProviderMessage, RequestUpdate},
    },
    store::fs::FsStore,
    ticket::BlobTicket,
    BlobFormat, BlobsProtocol, Hash,
};
use n0_future::{task::AbortOnDropHandle, FuturesUnordered, StreamExt};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use walkdir::WalkDir;

use crate::{
    config::config,
    logger::{debug, error, info, warn},
};

lazy_static::lazy_static! {
    static ref HOME_DIR: std::path::PathBuf = get_home_dir();
    static ref CWD: std::path::PathBuf = get_temp_dir();
}

// Platform-specific path resolution
fn get_home_dir() -> std::path::PathBuf {
    let _cfg = config();

    #[cfg(target_os = "android")]
    {
        use std::path::PathBuf;
        std::env::var("EXTERNAL_STORAGE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(&_cfg.platform.sdcard_fallback_path))
    }
    #[cfg(target_os = "ios")]
    {
        // On iOS, use the Documents directory
        dirs::home_dir()
            .map(|h| h.join("Documents"))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."))
    }
}

fn get_temp_dir() -> std::path::PathBuf {
    let cfg = config();

    #[cfg(target_os = "android")]
    {
        // App-private internal storage is always writable. The external
        // Android/data/<package> dir does not exist until the Java API
        // creates it, so plain std::fs gets "Permission denied" there.
        std::path::PathBuf::from(&cfg.platform.android_app_data_path)
            .join(format!("{}temp", cfg.storage.temp_dir_prefix))
    }
    #[cfg(target_os = "ios")]
    {
        // On iOS, use the system temp directory with our prefix
        let temp = std::env::temp_dir();
        temp.join(format!("{}temp", cfg.storage.temp_dir_prefix))
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        home.join(&cfg.storage.documents_folder)
            .join(format!("{}temp", cfg.storage.temp_dir_prefix))
    }
}

/// Send a file or directory between two machines, using blake3 verified streaming.
///
/// For all subcommands, you can specify a secret key using the IROH_SECRET
/// environment variable. If you don't, a random one will be generated.
///
/// You can also specify a port for the magicsocket. If you don't, a random one
/// will be chosen.
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    #[default]
    Hex,
    Cid,
}

impl FromStr for Format {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "hex" => Ok(Format::Hex),
            "cid" => Ok(Format::Cid),
            _ => Err(anyhow::anyhow!("invalid format")),
        }
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Hex => write!(f, "hex"),
            Format::Cid => write!(f, "cid"),
        }
    }
}

fn print_hash(hash: &Hash, format: Format) -> String {
    match format {
        Format::Hex => hash.to_hex().to_string(),
        Format::Cid => hash.to_string(),
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Send a file or directory.
    Send(SendArgs),

    /// Receive a file or directory.
    Receive(ReceiveArgs),
}

#[derive(Parser, Debug)]
pub struct CommonArgs {
    /// The IPv4 address that magicsocket will listen on.
    ///
    /// If None, defaults to a random free port, but it can be useful to specify a fixed
    /// port, e.g. to configure a firewall rule.
    #[clap(long, default_value = None)]
    pub magic_ipv4_addr: Option<SocketAddrV4>,

    /// The IPv6 address that magicsocket will listen on.
    ///
    /// If None, defaults to a random free port, but it can be useful to specify a fixed
    /// port, e.g. to configure a firewall rule.
    #[clap(long, default_value = None)]
    pub magic_ipv6_addr: Option<SocketAddrV6>,

    #[clap(long, default_value_t = Format::Hex)]
    pub format: Format,

    #[clap(short = 'v', long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// The relay URL to use as a home relay,
    ///
    /// Can be set to "disable" to disable relay servers and "default"
    /// to configure default servers.
    #[clap(long, default_value_t = RelayModeOption::Default)]
    pub relay: RelayModeOption,
}

/// Available command line options for configuring relays.
#[derive(Clone, Debug)]
pub enum RelayModeOption {
    /// Disables relays altogether.
    Disabled,
    /// Uses the default relay servers.
    Default,
    /// Uses a single, custom relay server by URL.
    Custom(RelayUrl),
}

impl FromStr for RelayModeOption {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disabled" => Ok(Self::Disabled),
            "default" => Ok(Self::Default),
            _ => Ok(Self::Custom(RelayUrl::from_str(s)?)),
        }
    }
}

impl Display for RelayModeOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => f.write_str("disabled"),
            Self::Default => f.write_str("default"),
            Self::Custom(url) => url.fmt(f),
        }
    }
}

/// Resolves the relay mode, replacing `Default` with only the default relays
/// that accept a TCP connection on 443.
///
/// iroh picks its home relay by latency and never switches if that relay
/// turns out to be unreachable (some networks answer pings but drop TCP to a
/// given relay). Probing first keeps both peers on a relay they can use.
/// Falls back to the full default map when none respond.
// ponytail: one-shot probe at endpoint creation; no re-probing if the
// network changes mid-session.
async fn resolve_relay_mode(option: RelayModeOption) -> RelayMode {
    if !matches!(option, RelayModeOption::Default) {
        return option.into();
    }

    let probes = iroh::defaults::prod::default_relay_map()
        .relays::<Vec<_>>()
        .into_iter()
        .map(|relay| async move {
            let host = relay
                .url
                .host_str()
                .unwrap_or_default()
                .trim_end_matches('.')
                .to_string();
            let port = relay.url.port_or_known_default().unwrap_or(443);
            let ok = tokio::time::timeout(
                std::time::Duration::from_secs(3),
                tokio::net::TcpStream::connect((host.as_str(), port)),
            )
            .await
            .map(|r| r.is_ok())
            .unwrap_or(false);
            (relay, ok)
        })
        .collect::<Vec<_>>();

    let reachable: Vec<_> = futures::future::join_all(probes)
        .await
        .into_iter()
        .filter_map(|(relay, ok)| ok.then_some(relay))
        .collect();

    if reachable.is_empty() {
        warn!("No relay passed the reachability probe; using the default relay map");
        return RelayMode::Default;
    }
    let map: RelayMap = reachable.into_iter().collect();
    info!("Using {} reachable relay(s)", map.len());
    RelayMode::Custom(map)
}

impl From<RelayModeOption> for RelayMode {
    fn from(value: RelayModeOption) -> Self {
        match value {
            RelayModeOption::Disabled => RelayMode::Disabled,
            RelayModeOption::Default => RelayMode::Default,
            RelayModeOption::Custom(url) => RelayMode::Custom(url.into()),
        }
    }
}

#[derive(Parser, Debug)]
pub struct SendArgs {
    /// Path to the file or directory to send.
    ///
    /// The last component of the path will be used as the name of the data
    /// being shared.
    pub path: PathBuf,

    /// What type of ticket to use.
    ///
    /// Use "id" for the shortest type only including the node ID,
    /// "addresses" to only add IP addresses without a relay url,
    /// "relay" to only add a relay address, and leave the option out
    /// to use the biggest type of ticket that includes both relay and
    /// address information.
    ///
    /// Generally, the more information the higher the likelyhood of
    /// a successful connection, but also the bigger a ticket to connect.
    ///
    /// This is most useful for debugging which methods of connection
    /// establishment work well.
    #[clap(long, default_value_t = AddrInfoOptions::RelayAndAddresses)]
    pub ticket_type: AddrInfoOptions,

    #[clap(flatten)]
    pub common: CommonArgs,
}

#[derive(Parser, Debug)]
pub struct ReceiveArgs {
    /// The ticket to use to connect to the sender.
    pub ticket: BlobTicket,

    #[clap(flatten)]
    pub common: CommonArgs,
}

/// Options to configure what is included in a [`NodeAddr`]
#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    Default,
    Debug,
    derive_more::Display,
    derive_more::FromStr,
    Serialize,
    Deserialize,
)]
pub enum AddrInfoOptions {
    /// Only the Node ID is added.
    ///
    /// This usually means that iroh-dns discovery is used to find address information.
    #[default]
    Id,
    /// Includes the Node ID and both the relay URL, and the direct addresses.
    RelayAndAddresses,
    /// Includes the Node ID and the relay URL.
    Relay,
    /// Includes the Node ID and the direct addresses.
    Addresses,
}

fn apply_options(addr: &mut EndpointAddr, opts: AddrInfoOptions) {
    match opts {
        AddrInfoOptions::Id => {
            addr.addrs = Default::default();
        }
        AddrInfoOptions::RelayAndAddresses => {
            // nothing to do
        }
        AddrInfoOptions::Relay => {
            addr.addrs = addr
                .addrs
                .iter()
                .filter(|addr| matches!(addr, TransportAddr::Relay(_)))
                .cloned()
                .collect();
        }
        AddrInfoOptions::Addresses => {
            addr.addrs = addr
                .addrs
                .iter()
                .filter(|addr| matches!(addr, TransportAddr::Ip(_)))
                .cloned()
                .collect();
        }
    }
}

/// Get the secret key or generate a new one.
///
/// Print the secret key to stderr if it was generated, so the user can save it.
fn get_or_create_secret(print: bool) -> anyhow::Result<SecretKey> {
    match std::env::var("IROH_SECRET") {
        Ok(secret) => SecretKey::from_str(&secret).context("invalid secret"),
        Err(_) => {
            let key = SecretKey::generate();
            if print {
                info!("Generated a new secret key");
            }
            Ok(key)
        }
    }
}

fn validate_path_component(component: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        !component.contains('/'),
        "path components must not contain the only correct path separator, /"
    );
    Ok(())
}

/// This function converts an already canonicalized path to a string.
///
/// If `must_be_relative` is true, the function will fail if any component of the path is
/// `Component::RootDir`
///
/// This function will also fail if the path is non canonical, i.e. contains
/// `..` or `.`, or if the path components contain any windows or unix path
/// separators.
pub fn canonicalized_path_to_string(
    path: impl AsRef<Path>,
    must_be_relative: bool,
) -> anyhow::Result<String> {
    let mut path_str = String::new();
    let parts = path
        .as_ref()
        .components()
        .filter_map(|c| match c {
            Component::Normal(x) => {
                let c = match x.to_str() {
                    Some(c) => c,
                    None => return Some(Err(anyhow::anyhow!("invalid character in path"))),
                };

                if !c.contains('/') && !c.contains('\\') {
                    Some(Ok(c))
                } else {
                    Some(Err(anyhow::anyhow!("invalid path component {:?}", c)))
                }
            }
            Component::RootDir => {
                if must_be_relative {
                    Some(Err(anyhow::anyhow!("invalid path component {:?}", c)))
                } else {
                    path_str.push('/');
                    None
                }
            }
            _ => Some(Err(anyhow::anyhow!("invalid path component {:?}", c))),
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let parts = parts.join("/");
    path_str.push_str(&parts);
    Ok(path_str)
}

/// Import from a file or directory into the store.
///
/// The returned tag always refers to a collection. If the input is a file, this
/// is a collection with a single blob, named like the file.
///
/// If the input is a directory, the collection contains all the files in the
/// directory.
#[allow(dead_code)]
async fn import(path: PathBuf, db: &Store) -> anyhow::Result<(TempTag, u64, Collection)> {
    import_with_window(path, db, None).await
}

/// Import with optional window for emitting progress events
async fn import_with_window(
    path: PathBuf,
    db: &Store,
    window: Option<tauri::Window>,
) -> anyhow::Result<(TempTag, u64, Collection)> {
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

    let path = path.canonicalize()?;
    anyhow::ensure!(path.exists(), "path {} does not exist", path.display());
    let root = path.parent().context("context get parent")?;
    // walkdir also works for files, so we don't need to special case them
    let files = WalkDir::new(path.clone()).into_iter();
    // flatten the directory structure into a list of (name, path) pairs.
    // ignore symlinks.
    let data_sources: Vec<(String, PathBuf)> = files
        .map(|entry| {
            let entry = entry?;
            if !entry.file_type().is_file() {
                // Skip symlinks. Directories are handled by WalkDir.
                return Ok(None);
            }
            let path = entry.into_path();
            let relative = path.strip_prefix(root)?;
            let name = canonicalized_path_to_string(relative, true)?;
            anyhow::Ok(Some((name, path)))
        })
        .filter_map(Result::transpose)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let total_files = data_sources.len();
    // Known up front so progress can follow bytes hashed, which matters for one huge file.
    let total_bytes: u64 = data_sources
        .iter()
        .filter_map(|(_, p)| std::fs::metadata(p).ok())
        .map(|m| m.len())
        .sum();
    let bytes_done = Arc::new(AtomicU64::new(0));
    let files_done = Arc::new(AtomicUsize::new(0));
    let last_percent = Arc::new(AtomicU64::new(u64::MAX));

    let emit = {
        let window = window.clone();
        let bytes_done = bytes_done.clone();
        let files_done = files_done.clone();
        let last_percent = last_percent.clone();
        move |current_file: &str, force: bool| {
            let Some(win) = &window else { return };
            let percentage = if total_bytes > 0 {
                bytes_done.load(Ordering::Relaxed) as f64 / total_bytes as f64 * 100.0
            } else {
                files_done.load(Ordering::Relaxed) as f64 / total_files.max(1) as f64 * 100.0
            }
            .min(100.0);
            // only when the whole percent changes, to keep the event stream light
            if !force
                && last_percent.swap(percentage as u64, Ordering::Relaxed) == percentage as u64
            {
                return;
            }
            let _ = win.emit(
                "import_progress",
                crate::events::ImportProgressEvent {
                    percentage,
                    current_file: current_file.to_string(),
                    files_processed: files_done.load(Ordering::Relaxed),
                    total_files,
                },
            );
        }
    };

    // import all the files, using num_cpus workers, return names and temp tags
    let mut names_and_tags = n0_future::stream::iter(data_sources)
        .map(|(name, path)| {
            let db = db.clone();
            let emit = emit.clone();
            let bytes_done = bytes_done.clone();
            let files_done = files_done.clone();
            async move {
                emit(&name, true);
                // iOS hands out a temporary copy of the picked file that the
                // system deletes soon after, so referencing it in place would
                // leave nothing to serve; copy into the store there instead.
                #[cfg(target_os = "ios")]
                let mode = ImportMode::Copy;
                #[cfg(not(target_os = "ios"))]
                let mode = ImportMode::TryReference;
                let import = db.add_path_with_opts(AddPathOptions {
                    path,
                    mode,
                    format: BlobFormat::Raw,
                });
                let mut stream = import.stream().await;
                let mut item_size = 0;
                let mut hashed = 0u64;
                let temp_tag = loop {
                    let item = stream
                        .next()
                        .await
                        .context("import stream ended without a tag")?;
                    match item {
                        AddProgressItem::Size(size) => item_size = size,
                        AddProgressItem::OutboardProgress(offset) => {
                            bytes_done.fetch_add(offset.saturating_sub(hashed), Ordering::Relaxed);
                            hashed = offset;
                            emit(&name, false);
                        }
                        AddProgressItem::CopyProgress(_) | AddProgressItem::CopyDone => {}
                        AddProgressItem::Error(cause) => {
                            anyhow::bail!("error importing {}: {}", name, cause);
                        }
                        AddProgressItem::Done(tt) => break tt,
                    }
                };
                // the hashing progress may not report the very end
                bytes_done.fetch_add(item_size.saturating_sub(hashed), Ordering::Relaxed);
                files_done.fetch_add(1, Ordering::Relaxed);
                emit(&name, true);
                anyhow::Ok((name, temp_tag, item_size))
            }
        })
        // capped: each import holds a file open, and macOS allows only 256 by default
        .buffered_unordered(num_cpus::get().clamp(1, 16))
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;
    names_and_tags.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));
    // total size of all files
    let size = names_and_tags.iter().map(|(_, _, size)| *size).sum::<u64>();
    // collect the (name, hash) tuples into a collection
    // we must also keep the tags around so the data does not get gced.
    let (collection, tags) = names_and_tags
        .into_iter()
        .map(|(name, tag, _)| ((name, tag.hash()), tag))
        .unzip::<_, _, Collection, Vec<_>>();
    let temp_tag = collection.clone().store(db).await?;
    // now that the collection is stored, we can drop the tags
    // data is protected by the collection
    drop(tags);
    Ok((temp_tag, size, collection))
}

fn get_export_path(root: &Path, name: &str) -> anyhow::Result<PathBuf> {
    let parts = name.split('/');
    let mut path = root.to_path_buf();
    for part in parts {
        validate_path_component(part)?;
        path.push(part);
    }
    Ok(path)
}

/// What to do when a file being received already exists.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ConflictChoice {
    Overwrite,
    Rename,
    Cancel,
}

lazy_static::lazy_static! {
    /// Pending answer to the "file already exists" prompt, if one is open.
    static ref CONFLICT_ANSWER: std::sync::Mutex<Option<tokio::sync::oneshot::Sender<ConflictChoice>>> =
        std::sync::Mutex::new(None);
}

/// Called by the UI with "overwrite", "rename" or "cancel".
pub fn resolve_conflict(choice: &str) {
    let choice = match choice {
        "overwrite" => ConflictChoice::Overwrite,
        "rename" => ConflictChoice::Rename,
        _ => ConflictChoice::Cancel,
    };
    if let Some(tx) = CONFLICT_ANSWER.lock().ok().and_then(|mut g| g.take()) {
        let _ = tx.send(choice);
    }
}

/// Asks the user (via the window) what to do with `existing` files.
/// Without a window there is nobody to ask, so keep both.
async fn ask_conflict(window: Option<&tauri::Window>, existing: &[String]) -> ConflictChoice {
    let Some(window) = window else {
        return ConflictChoice::Rename;
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    if let Ok(mut slot) = CONFLICT_ANSWER.lock() {
        *slot = Some(tx);
    }
    let _ = window.emit(
        "file_conflict",
        crate::events::FileConflictEvent {
            count: existing.len(),
            names: existing.iter().take(5).cloned().collect(),
        },
    );
    // A prompt nobody answers must not hang the transfer forever.
    match tokio::time::timeout(Duration::from_secs(300), rx).await {
        Ok(Ok(choice)) => choice,
        _ => ConflictChoice::Cancel,
    }
}

/// `name (n).ext` next to `target`, used when `name.ext` is already taken.
fn numbered_path(target: &Path, n: u32) -> PathBuf {
    let stem = target
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name = match target.extension() {
        Some(ext) => format!("{stem} ({n}).{}", ext.to_string_lossy()),
        None => format!("{stem} ({n})"),
    };
    target.with_file_name(name)
}

async fn export(
    db: &Store,
    collection: Collection,
    storage_file_path: String,
    window: Option<&tauri::Window>,
) -> anyhow::Result<()> {
    // On Android the user can pick a save folder; it is a content URI that
    // plain file APIs cannot write to, so those files are delivered through
    // the Storage Access Framework (see export_to_saf).
    #[cfg(target_os = "android")]
    if let Some(json) = storage_file_path.strip_prefix("saf:") {
        let window = window.context("saving to a picked folder needs the app window")?;
        return export_to_saf(db, collection, json, window).await;
    }
    // Same idea on iOS: the user can pick a folder outside the app sandbox,
    // stored as a bookmark (see export_to_ios_folder).
    #[cfg(target_os = "ios")]
    if let Some(json) = storage_file_path.strip_prefix("ios:") {
        let window = window.context("saving to a picked folder needs the app window")?;
        return export_to_ios_folder(db, collection, json, window).await;
    }
    export_to_path(db, collection, PathBuf::from(storage_file_path), window).await
}

/// Receives into app-private staging with the normal export, then copies the
/// files into the folder the user picked (via its bookmark), asking about
/// clashes once.
#[cfg(target_os = "ios")]
async fn export_to_ios_folder(
    db: &Store,
    collection: Collection,
    location_json: &str,
    window: &tauri::Window,
) -> anyhow::Result<()> {
    use tauri::Manager;
    use tauri_plugin_ios_folder::IosFolderExt;

    #[derive(Deserialize)]
    struct Location {
        bookmark: String,
    }
    let bookmark = serde_json::from_str::<Location>(location_json)
        .context("the saved save-location is not valid")?
        .bookmark;
    let app = window.app_handle();

    let staging = CWD
        .join("ios-staging")
        .join(HEXLOWER.encode(&rand::rng().random::<[u8; 8]>()));
    export_to_path(db, collection, staging.clone(), None).await?;

    let result = async {
        let files: Vec<String> = WalkDir::new(&staging)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                e.path()
                    .strip_prefix(&staging)
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned())
            })
            .collect();

        let existing = app
            .ios_folder()
            .existing(&bookmark, &files)
            .await
            .map_err(anyhow::Error::msg)?;
        let choice = if existing.is_empty() {
            ConflictChoice::Rename
        } else {
            ask_conflict(Some(window), &existing).await
        };
        if choice == ConflictChoice::Cancel {
            anyhow::bail!(
                "Download cancelled: {} file(s) already exist",
                existing.len()
            );
        }

        app.ios_folder()
            .copy_in(
                &bookmark,
                &staging.to_string_lossy(),
                choice == ConflictChoice::Overwrite,
            )
            .await
            .map_err(anyhow::Error::msg)
    }
    .await;

    let _ = tokio::fs::remove_dir_all(&staging).await;
    result
}

/// Receives into app-private staging with the normal export, then copies the
/// files into the folder the user picked, asking about clashes once.
#[cfg(target_os = "android")]
async fn export_to_saf(
    db: &Store,
    collection: Collection,
    tree_json: &str,
    window: &tauri::Window,
) -> anyhow::Result<()> {
    use tauri::Manager;
    use tauri_plugin_android_fs::{AndroidFsExt, FsUri};

    let tree = FsUri::from_json_str(tree_json).context("the saved save-location is not valid")?;
    let api = window.app_handle().android_fs_async();

    let staging = CWD
        .join("saf-staging")
        .join(HEXLOWER.encode(&rand::rng().random::<[u8; 8]>()));
    export_to_path(db, collection, staging.clone(), None).await?;

    let result = async {
        let files: Vec<String> = WalkDir::new(&staging)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                e.path()
                    .strip_prefix(&staging)
                    .ok()
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
            })
            .collect();

        let mut existing = Vec::new();
        for rel in &files {
            if api.resolve_file_uri(&tree, rel).await.is_ok() {
                existing.push(rel.clone());
            }
        }
        let choice = if existing.is_empty() {
            ConflictChoice::Rename
        } else {
            ask_conflict(Some(window), &existing).await
        };
        if choice == ConflictChoice::Cancel {
            anyhow::bail!(
                "Download cancelled: {} file(s) already exist",
                existing.len()
            );
        }

        for rel in &files {
            // Overwrite reuses the existing file; otherwise create_new_file
            // makes a new one and numbers the name if it is taken.
            let dest = match api.resolve_file_uri(&tree, rel).await {
                Ok(uri) if choice == ConflictChoice::Overwrite => uri,
                _ => api
                    .create_new_file(&tree, rel, None)
                    .await
                    .with_context(|| format!("create {rel} in the chosen folder"))?,
            };
            let mut out = api
                .open_file_writable(&dest)
                .await
                .with_context(|| format!("open {rel} for writing"))?;
            let from = staging.join(rel);
            tokio::task::spawn_blocking(move || {
                std::io::copy(&mut std::fs::File::open(from)?, &mut out)
            })
            .await??;
        }
        anyhow::Ok(())
    }
    .await;

    let _ = tokio::fs::remove_dir_all(&staging).await;
    result
}

/// True if an error (or anything it wraps) is an "already exists" I/O error.
fn is_already_exists(e: &(dyn std::error::Error + 'static)) -> bool {
    let mut cur = Some(e);
    while let Some(err) = cur {
        if let Some(io) = err.downcast_ref::<std::io::Error>() {
            return io.kind() == std::io::ErrorKind::AlreadyExists || io.raw_os_error() == Some(17);
        }
        cur = err.source();
    }
    false
}

async fn export_to_path(
    db: &Store,
    collection: Collection,
    root: PathBuf,
    window: Option<&tauri::Window>,
) -> anyhow::Result<()> {
    let mut targets = Vec::new();
    for (name, hash) in collection.iter() {
        targets.push((name.clone(), *hash, get_export_path(&root, name)?));
    }

    // Ask once for the whole transfer if anything is already there.
    let existing: Vec<String> = targets
        .iter()
        .filter(|(_, _, target)| target.exists())
        .map(|(name, _, _)| name.clone())
        .collect();
    let choice = if existing.is_empty() {
        ConflictChoice::Rename
    } else {
        ask_conflict(window, &existing).await
    };
    if choice == ConflictChoice::Cancel {
        anyhow::bail!(
            "Download cancelled: {} file(s) already exist",
            existing.len()
        );
    }

    for (_, hash, target) in targets {
        if choice == ConflictChoice::Overwrite && target.is_file() {
            tokio::fs::remove_file(&target).await?;
        }
        // Otherwise never overwrite: take the next free "name (n).ext". The
        // create can also fail with EEXIST when the name is invisibly taken
        // (e.g. reserved by Samsung's recycle bin on Android); that cannot
        // be overwritten, so it falls through to the next free name too.
        let mut candidate = target.clone();
        let mut n = 0;
        loop {
            if candidate.exists() {
                n += 1;
            } else {
                match db
                    .export_with_opts(ExportOptions {
                        hash,
                        target: candidate.clone(),
                        mode: ExportMode::TryReference,
                    })
                    .await
                {
                    Ok(_) => break,
                    Err(e) if is_already_exists(&e) => {
                        warn!("{} is taken, trying another name", candidate.display());
                        n += 1;
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            if n > 100 {
                anyhow::bail!("could not find a free name for {}", target.display());
            }
            candidate = numbered_path(&target, n);
        }
    }
    Ok(())
}

/// Watches downloads of what this device is sharing and tells the UI how much
/// has been sent (`send_transfer_started`, `send_transfer_progress`).
async fn show_provide_progress(
    mut recv: tokio::sync::mpsc::Receiver<ProviderMessage>,
    window: Option<tauri::Window>,
    total_size: Arc<std::sync::atomic::AtomicU64>,
) {
    let mut tasks = FuturesUnordered::new();
    loop {
        tokio::select! {
            biased;
            item = recv.recv() => {
                let Some(item) = item else { break };
                match item {
                    ProviderMessage::ClientConnectedNotify(_) => {
                        if let Some(win) = &window {
                            let _ = win.emit("send_transfer_started", ());
                        }
                    }
                    ProviderMessage::GetRequestReceivedNotify(msg) => {
                        tasks.push(track_request(msg.rx, window.clone(), total_size.clone()));
                    }
                    _ => {}
                }
            }
            Some(_) = tasks.next(), if !tasks.is_empty() => {}
        }
    }
    while tasks.next().await.is_some() {}
}

/// Follows one download request from start to finish.
async fn track_request(
    mut rx: irpc::channel::mpsc::Receiver<RequestUpdate>,
    window: Option<tauri::Window>,
    total_size: Arc<std::sync::atomic::AtomicU64>,
) {
    use std::sync::atomic::Ordering;

    let started = Instant::now();
    let mut last_emit = Instant::now();
    // A request covers every file in the share; offsets restart for each file.
    let mut sent_in_finished_files = 0u64;
    let mut current_file_size = 0u64;
    let mut sent = 0u64;

    let emit = |sent: u64, done: bool| {
        let Some(win) = &window else { return };
        let total = total_size.load(Ordering::Relaxed);
        let elapsed = started.elapsed();
        let speed = if elapsed.as_secs_f64() > 0.0 {
            (sent as f64 / elapsed.as_secs_f64()) as u64
        } else {
            0
        };
        let percentage = if done || total == 0 {
            100.0
        } else {
            (sent as f64 / total as f64 * 100.0).min(100.0)
        };
        let eta_ms = if speed > 0 && !done {
            (total.saturating_sub(sent) as f64 / speed as f64 * 1000.0) as u64
        } else {
            0
        };
        let _ = win.emit(
            "send_transfer_progress",
            crate::events::TransferProgressEvent {
                bytes_transferred: sent,
                total_bytes: total,
                percentage,
                speed_bytes_per_sec: speed,
                elapsed_ms: elapsed.as_millis() as u64,
                eta_ms,
            },
        );
    };

    while let Ok(Some(update)) = rx.recv().await {
        match update {
            RequestUpdate::Started(msg) => {
                sent_in_finished_files += current_file_size;
                current_file_size = msg.size;
            }
            RequestUpdate::Progress(msg) => {
                sent = sent_in_finished_files + msg.end_offset;
                // at most ten updates a second
                if last_emit.elapsed() >= Duration::from_millis(100) {
                    emit(sent, false);
                    last_emit = Instant::now();
                }
            }
            RequestUpdate::Completed(_) => emit(sent, true),
            RequestUpdate::Aborted(_) => {}
        }
    }
}

// use serde::{Deserialize, Serialize};
use std::error::Error;

use reqwest::Client;

/// An HTTP client for the code server. Installs the TLS provider on first use
/// (reqwest is built without one, see Cargo.toml) and applies the configured
/// timeout, so a hung server cannot hang a send.
fn http_client(timeout: Duration) -> Client {
    static TLS: std::sync::Once = std::sync::Once::new();
    TLS.call_once(|| {
        // Fails only if a provider is already installed, which is fine.
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
    Client::builder()
        .timeout(timeout)
        .build()
        .unwrap_or_else(|_| Client::new())
}

#[derive(Serialize)]
struct BlobRequest {
    blobs: Vec<String>,
}

#[derive(Deserialize)]
struct BlobResponse {
    code: String,
}

#[derive(Deserialize)]
struct GetBlobResponse {
    blobs: Vec<String>,
}

pub async fn create_blobs(blobs: Vec<String>) -> Result<String, Box<dyn Error>> {
    let cfg = config();

    // No code-exchange server configured: hand back the raw ticket(s)
    // directly. The caller displays this as text + QR instead of a short code.
    if cfg.api.base_url.is_empty() {
        info!(
            "No BASE_URL configured, returning {} raw ticket(s)",
            blobs.len()
        );
        return Ok(blobs.join(","));
    }

    let client = http_client(Duration::from_secs(cfg.api.timeout_secs));
    let payload = BlobRequest { blobs };

    info!(
        "Creating {} blobs using BASE_URL: {}",
        payload.blobs.len(),
        cfg.api.base_url
    );

    let request = async {
        client
            .post(format!("{}/api/code/", cfg.api.base_url))
            .json(&payload)
            .send()
            .await?
            .json::<BlobResponse>()
            .await
    };
    match request.await {
        Ok(response) => {
            info!("Blobs created successfully with code: {}", response.code);
            Ok(response.code)
        }
        // Raw tickets work without the server (see `get_blob`), so an
        // unreachable server shouldn't block sending.
        Err(e) => {
            warn!("Code server unavailable ({e}), returning raw ticket(s)");
            Ok(payload.blobs.join(","))
        }
    }
}

/// True if a code server is configured and answers. The UI only offers
/// short codes when this holds; tickets and QR always work.
pub async fn code_server_available() -> bool {
    let base = &config().api.base_url;
    if base.is_empty() {
        return false;
    }
    // Any HTTP response (even a 404) means the server is up.
    http_client(Duration::from_secs(3))
        .get(base)
        .send()
        .await
        .is_ok()
}

/// True if every comma-separated segment of `code` parses as a raw blob
/// ticket, meaning it was produced without a code-exchange server.
fn looks_like_raw_tickets(code: &str) -> bool {
    code.split(',')
        .all(|segment| BlobTicket::from_str(segment).is_ok())
}

pub async fn get_blob(code: String) -> Result<Vec<String>, Box<dyn Error>> {
    // Self-describing: a value made only of valid tickets was shared
    // without a server, regardless of whether one is configured locally.
    if looks_like_raw_tickets(&code) {
        debug!("Code looks like raw ticket(s), skipping server lookup");
        return Ok(code.split(',').map(|s| s.to_string()).collect());
    }

    let cfg = config();
    if cfg.api.base_url.is_empty() {
        return Err("This code requires a server, but none is configured".into());
    }

    let client = http_client(Duration::from_secs(cfg.api.timeout_secs));

    debug!("Fetching blob with code: {}", code);

    let url: String = format!("{}/api/code/{}", cfg.api.base_url, code);

    // One retry: a flaky connection shouldn't make a valid code look broken.
    let mut attempt = 1;
    let response = loop {
        match client.get(&url).send().await {
            Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => {
                return Err("code not found".into());
            }
            Ok(resp) if !resp.status().is_success() => {
                return Err(format!("code server error ({})", resp.status()).into());
            }
            Ok(resp) => match resp.json::<GetBlobResponse>().await {
                Ok(parsed) => break parsed,
                Err(e) => return Err(format!("code server sent an unreadable reply: {e}").into()),
            },
            Err(e) if attempt < 2 => {
                warn!("Code lookup failed ({e}), retrying");
                attempt += 1;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(e) => return Err(format!("code server unreachable: {e}").into()),
        }
    };

    info!("Successfully fetched {} blob(s)", response.blobs.len());
    Ok(response.blobs)
}

/// Logs why a download failed, and passes the error on.
fn log_get_error(e: GetError) -> GetError {
    warn!("Download failed: {e}");
    e
}

// In your existing module (or in a new one), add the following:
pub async fn start_send(
    args: SendArgs,
    window: Option<tauri::Window>,
) -> anyhow::Result<(BlobTicket, iroh::protocol::Router, PathBuf)> {
    let cfg = config();

    debug!("Getting secret key");
    let secret_key = get_or_create_secret(args.common.verbose > 0)?;

    debug!("Building endpoint");
    let relay_mode = resolve_relay_mode(args.common.relay).await;
    let mut builder = Endpoint::builder(presets::N0)
        .alpns(vec![iroh_blobs::protocol::ALPN.to_vec()])
        .secret_key(secret_key)
        .relay_mode(relay_mode.clone());

    if args.ticket_type == AddrInfoOptions::Id {
        builder = builder.address_lookup(PkarrPublisher::n0_dns());
    }
    if let Some(addr) = args.common.magic_ipv4_addr {
        builder = builder.bind_addr(addr)?;
    }
    if let Some(addr) = args.common.magic_ipv6_addr {
        builder = builder.bind_addr(addr)?;
    }

    // Create a unique directory for this sending session.
    debug!("Creating blobs data directory");
    let suffix = rand::rng().random::<[u8; 16]>();
    let blobs_data_dir = CWD.join(format!(
        "{}send-{}",
        cfg.storage.temp_dir_prefix,
        HEXLOWER.encode(&suffix)
    ));

    if blobs_data_dir.exists() {
        error!(
            "Cannot share twice from the same directory: {}",
            CWD.display()
        );
        anyhow::bail!(
            "Cannot share twice from the same directory: {}",
            CWD.display()
        );
    }

    tokio::fs::create_dir_all(&blobs_data_dir).await?;
    debug!("Blobs directory created: {:?}", blobs_data_dir);

    debug!("Binding endpoint");
    let endpoint = builder.bind().await?;

    debug!("Creating blobs store");
    let store = FsStore::load(&blobs_data_dir).await?;
    // The share's total size is only known after the import, so the progress
    // tracker reads it from here.
    let total_size = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let (progress_tx, progress_rx) = tokio::sync::mpsc::channel(32);
    tokio::spawn(show_provide_progress(
        progress_rx,
        window.clone(),
        total_size.clone(),
    ));
    let blobs = BlobsProtocol::new(
        &store,
        Some(EventSender::new(
            progress_tx,
            EventMask {
                connected: ConnectMode::Notify,
                get: provider::events::RequestMode::NotifyLog,
                ..EventMask::DEFAULT
            },
        )),
    );

    info!("Importing file: {:?}", args.path);
    let path = args.path;
    // Import first to get the size, which we'll need for progress tracking
    let (temp_tag, size, collection) =
        import_with_window(path.clone(), blobs.store(), window.clone()).await?;
    total_size.store(size, std::sync::atomic::Ordering::Relaxed);
    info!("Import complete, size: {}", HumanBytes(size));
    let hash = temp_tag.hash();

    debug!("Building router");
    let router = iroh::protocol::Router::builder(endpoint)
        .accept(iroh_blobs::ALPN, blobs.clone())
        .spawn();

    // Wait for the endpoint to find its address before making a ticket.
    debug!("Waiting for the endpoint to come online");
    let ep = router.endpoint().clone();
    tokio::time::timeout(Duration::from_secs(30), async move {
        if !matches!(relay_mode, RelayMode::Disabled) {
            let _ = ep.online().await;
        }
    })
    .await?;
    debug!("Endpoint online");

    let mut addr = router.endpoint().addr();
    apply_options(&mut addr, args.ticket_type);
    let ticket = BlobTicket::new(addr, hash, BlobFormat::HashSeq);
    info!("Ticket created successfully");

    let entry_type = if path.is_file() { "file" } else { "directory" };
    info!(
        "Imported {} {}, {}, hash {}",
        entry_type,
        path.display(),
        HumanBytes(size),
        print_hash(&hash, args.common.format)
    );

    if args.common.verbose > 0 {
        for (name, hash) in collection.iter() {
            debug!("    {} {name}", print_hash(hash, args.common.format));
        }
    }

    drop(temp_tag);

    // Return the ticket along with the router and data directory
    Ok((ticket, router, blobs_data_dir))
}

use tauri::State;

// SharedSenderState SenderState from src/sender_state.rs
use crate::sender_state::SharedSenderState;

pub async fn send_file_minimal(
    file_path: String,
    verbose: bool,
    state: State<'_, SharedSenderState>,
    window: tauri::Window,
) -> anyhow::Result<String> {
    // Construct a minimal SendArgs using the file_path and verbose flag.
    let send_args = SendArgs {
        path: PathBuf::from(file_path),
        ticket_type: AddrInfoOptions::RelayAndAddresses,
        common: CommonArgs {
            magic_ipv4_addr: None,
            magic_ipv6_addr: None,
            format: Format::Hex,
            verbose: if verbose { 1 } else { 0 },
            relay: RelayModeOption::Default,
        },
    };

    // Call our new start_send() which sets up sharing with progress tracking.
    let (ticket, router, blobs_data_dir) = start_send(send_args, Some(window)).await?;

    info!("Sharing file with ticket: {}", ticket);

    // Store the router and blobs_data_dir in global state so we can later shut down the sender.
    {
        let mut sender_state = state.lock().unwrap();
        sender_state.router = Some(router);
        sender_state.blobs_data_dir = Some(blobs_data_dir);
    }
    // Save the blob to the database using create_blobs (single blob in a vector)
    let blob = ticket.to_string();
    match create_blobs(vec![blob]).await {
        Ok(code) => Ok(code),
        Err(err) => Err(anyhow::anyhow!("Failed to create blob: {}", err)),
    }
}

pub async fn send_files_minimal(
    file_paths: Vec<String>,
    verbose: bool,
    state: State<'_, SharedSenderState>,
    window: tauri::Window,
) -> anyhow::Result<String> {
    // If only one file, use the single file method
    if file_paths.len() == 1 {
        return send_file_minimal(file_paths[0].clone(), verbose, state, window).await;
    }

    info!("Starting multi-file send for {} files", file_paths.len());

    // Create individual blob tickets for each file
    let mut blob_tickets = Vec::new();

    for (idx, file_path) in file_paths.iter().enumerate() {
        let source = PathBuf::from(file_path);
        if !source.exists() {
            anyhow::bail!("File not found: {}", file_path);
        }

        info!(
            "Creating blob for file {}/{}: {}",
            idx + 1,
            file_paths.len(),
            file_path
        );

        // Create send args for this individual file
        let send_args = SendArgs {
            path: source.clone(),
            ticket_type: AddrInfoOptions::RelayAndAddresses,
            common: CommonArgs {
                magic_ipv4_addr: None,
                magic_ipv6_addr: None,
                format: Format::Hex,
                verbose: if verbose { 1 } else { 0 },
                relay: RelayModeOption::Default,
            },
        };

        // Start send for this file
        let (ticket, router, blobs_data_dir) = start_send(send_args, Some(window.clone())).await?;

        info!("Created blob ticket for {}: {}", file_path, ticket);

        // Store ALL routers and blobs_data_dirs in global state
        // This keeps all file endpoints alive for download
        {
            let mut sender_state = state.lock().unwrap();
            sender_state.routers.push(router);
            sender_state.blobs_data_dirs.push(blobs_data_dir);
        }

        // Add the blob ticket to our collection
        blob_tickets.push(ticket.to_string());
    }

    info!(
        "Created {} blob tickets, now creating code",
        blob_tickets.len()
    );

    // Create a single code that contains all blob tickets
    match create_blobs(blob_tickets).await {
        Ok(code) => {
            info!(
                "Successfully created code {} for {} files",
                code,
                file_paths.len()
            );
            Ok(code)
        }
        Err(err) => Err(anyhow::anyhow!("Failed to create code for blobs: {}", err)),
    }
}

pub async fn stop_sharing(state: State<'_, SharedSenderState>) -> anyhow::Result<()> {
    let cfg = config();
    let (router, blobs_data_dir, routers, blobs_data_dirs) = {
        let mut sender_state = state.lock().unwrap();
        (
            sender_state.router.take(),
            sender_state.blobs_data_dir.take(),
            std::mem::take(&mut sender_state.routers),
            std::mem::take(&mut sender_state.blobs_data_dirs),
        )
    };

    // Shut down the single router if present (for single file transfers)
    if let Some(router) = router {
        debug!("Shutting down router");
        tokio::time::timeout(
            Duration::from_secs(cfg.network.router_shutdown_timeout_secs),
            router.shutdown(),
        )
        .await??;
        info!("Router shutdown complete");
    }

    // Shut down all routers (for multi-file transfers)
    for (idx, router) in routers.into_iter().enumerate() {
        debug!("Shutting down router {}", idx + 1);
        if let Err(e) = tokio::time::timeout(
            Duration::from_secs(cfg.network.router_shutdown_timeout_secs),
            router.shutdown(),
        )
        .await
        {
            warn!("Failed to shutdown router {}: {:?}", idx + 1, e);
        } else {
            info!("Router {} shutdown complete", idx + 1);
        }
    }

    // Clean up the single blobs directory if present
    if let Some(dir) = blobs_data_dir {
        debug!("Cleaning up blobs directory: {:?}", dir);
        tokio::fs::remove_dir_all(&dir).await?;
        info!("Blobs directory cleaned up");
    }

    // Clean up all blobs directories (for multi-file transfers)
    for (idx, dir) in blobs_data_dirs.into_iter().enumerate() {
        debug!("Cleaning up blobs directory {}: {:?}", idx + 1, dir);
        if let Err(e) = tokio::fs::remove_dir_all(&dir).await {
            warn!("Failed to cleanup directory {}: {:?}", idx + 1, e);
        } else {
            info!("Blobs directory {} cleaned up", idx + 1);
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn receive_file_with_progress(
    ticket: String,
    file_storage_path: String,
    verbose: bool,
    window: Events,
) -> anyhow::Result<ReceiveSummary> {
    debug!("Receiving file from ticket: {}", ticket);

    let blobs = match get_blob(ticket).await {
        Ok(blobs) => blobs,
        Err(err) => {
            error!("Failed to get blob: {}", err);
            return Err(anyhow::anyhow!("Failed to get blob: {}", err));
        }
    };

    info!("Retrieved {} blob(s) to download", blobs.len());

    // Download all blobs one by one
    let mut parts = Vec::new();
    for (idx, blob_str) in blobs.iter().enumerate() {
        info!("Downloading blob {}/{}", idx + 1, blobs.len());
        let _ = window.emit(
            "download_status",
            format!("Downloading file {}/{}...", idx + 1, blobs.len()),
        );

        // Download this blob
        parts.push(
            receive_single_blob(blob_str, &file_storage_path, verbose, window.clone()).await?,
        );
    }

    info!("Successfully downloaded all {} file(s)", blobs.len());
    Ok(ReceiveSummary::combine(parts))
}

/// Where a transfer reports its progress: the app window, or nowhere (tests).
#[derive(Clone)]
pub struct Events(Option<tauri::Window>);

impl Events {
    pub fn new(window: tauri::Window) -> Self {
        Self(Some(window))
    }

    #[cfg(test)]
    fn none() -> Self {
        Self(None)
    }

    fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> tauri::Result<()> {
        match &self.0 {
            Some(window) => window.emit(event, payload),
            None => Ok(()),
        }
    }

    fn window(&self) -> Option<&tauri::Window> {
        self.0.as_ref()
    }
}

/// What a finished receive tells the UI.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReceiveSummary {
    /// The file or folder that was received (or "N items" for several).
    pub name: String,
    pub files: u32,
    pub bytes: u64,
    /// Time spent transferring, not counting waits such as the clash prompt.
    pub download_ms: u64,
}

impl ReceiveSummary {
    /// Merges the summaries of several tickets received in one go.
    fn combine(parts: Vec<ReceiveSummary>) -> ReceiveSummary {
        match parts.len() {
            0 => ReceiveSummary::default(),
            1 => parts.into_iter().next().unwrap_or_default(),
            n => ReceiveSummary {
                name: format!("{n} items"),
                files: parts.iter().map(|p| p.files).sum(),
                bytes: parts.iter().map(|p| p.bytes).sum(),
                download_ms: parts.iter().map(|p| p.download_ms).sum(),
            },
        }
    }
}

/// Display name for a received collection: the file or folder name if there
/// is exactly one top-level entry, otherwise "N items".
fn summary_name<'a>(names: impl Iterator<Item = &'a str>) -> String {
    let tops: std::collections::BTreeSet<&str> = names
        .filter_map(|n| n.split('/').next())
        .filter(|n| !n.is_empty())
        .collect();
    match tops.len() {
        0 => String::new(),
        1 => tops.into_iter().next().unwrap_or_default().to_string(),
        n => format!("{n} items"),
    }
}

/// Attempts at the network part of a receive (connect + download) before giving up.
const RECEIVE_ATTEMPTS: u32 = 3;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);

/// What a successful download reports, for the summary.
struct Fetched {
    stats: Stats,
    /// Size of the files themselves, without the collection's bookkeeping.
    payload_size: u64,
}

/// Retries `fetch_once` on failure. The store keeps what was already
/// downloaded, so a retry resumes rather than starting over.
async fn fetch_with_retry(
    ticket: &BlobTicket,
    db: &Store,
    verbose: bool,
    window: &Events,
) -> anyhow::Result<Fetched> {
    let mut attempt = 1;
    loop {
        match fetch_once(ticket, db, verbose, window, attempt).await {
            Ok(fetched) => return Ok(fetched),
            Err(e) if attempt >= RECEIVE_ATTEMPTS => return Err(e),
            Err(e) => {
                warn!("Receive attempt {attempt}/{RECEIVE_ATTEMPTS} failed: {e:#}");
                attempt += 1;
                let _ = window.emit(
                    "download_status",
                    format!("Connection failed, retrying ({attempt}/{RECEIVE_ATTEMPTS})..."),
                );
                tokio::time::sleep(Duration::from_secs(2 * (u64::from(attempt) - 1))).await;
            }
        }
    }
}

/// One attempt: fresh endpoint, connect to the sender and download into `db`.
/// Only what is still missing is requested, which is what makes a download resumable.
async fn fetch_once(
    ticket: &BlobTicket,
    db: &Store,
    verbose: bool,
    window: &Events,
    attempt: u32,
) -> anyhow::Result<Fetched> {
    let window = window.clone();
    // Keep the attempt visible, or the retry message is overwritten within seconds.
    let attempt_note = if attempt > 1 {
        format!(" (attempt {attempt}/{RECEIVE_ATTEMPTS})")
    } else {
        String::new()
    };
    let hash_and_format = ticket.hash_and_format();

    // Part or all of it may already be here from an earlier attempt or session.
    let local = db.remote().local(hash_and_format).await?;
    if local.is_complete() {
        info!("Everything is already downloaded");
        return Ok(Fetched {
            stats: Stats::default(),
            payload_size: local.local_bytes(),
        });
    }
    let local_size = local.local_bytes();

    let _ = window.emit(
        "download_status",
        format!("Initializing connection{attempt_note}..."),
    );
    let secret_key = get_or_create_secret(verbose)?;
    let mut builder = Endpoint::builder(presets::N0)
        .alpns(vec![])
        .secret_key(secret_key)
        .relay_mode(resolve_relay_mode(RelayModeOption::Default).await);

    if ticket.addr().relay_urls().next().is_none() && ticket.addr().ip_addrs().next().is_none() {
        builder = builder.address_lookup(DnsAddressLookup::n0_dns());
    }
    let endpoint = builder.bind().await?;

    let result = async {
        let _ = window.emit(
            "download_status",
            format!("Connecting to sender{attempt_note}..."),
        );
        let connection = tokio::time::timeout(
            CONNECT_TIMEOUT,
            endpoint.connect(ticket.addr().clone(), iroh_blobs::protocol::ALPN),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Timed out connecting to the sender"))??;

        // Tell the UI whether the transfer goes straight to the sender or through a
        // relay (slower), and when that changes (e.g. after hole punching succeeds).
        let _path_watcher = AbortOnDropHandle::new(tokio::spawn({
            let connection = connection.clone();
            let window = window.clone();
            async move {
                let mut paths = connection.paths_stream();
                let mut last = None;
                while let Some(list) = paths.next().await {
                    let relay = (&list)
                        .into_iter()
                        .find(|p| p.is_selected())
                        .map(|p| p.is_relay());
                    if relay.is_some() && relay != last {
                        let _ = window.emit(
                            "connection_path",
                            if relay == Some(true) {
                                "relay"
                            } else {
                                "direct"
                            },
                        );
                        last = relay;
                    }
                }
            }
        }));

        let (_hash_seq, sizes) =
            get_hash_seq_and_sizes(&connection, &hash_and_format.hash, 1024 * 1024 * 32, None)
                .await
                .map_err(log_get_error)?;
        let total_size = sizes.iter().copied().sum::<u64>();
        // the first entry is the collection's own bookkeeping, then come the files
        let payload_size = sizes.iter().skip(1).copied().sum::<u64>();
        info!(
            "Getting collection {} {} files, {} ({} already here)",
            print_hash(&ticket.hash(), Format::Hex),
            sizes.len().saturating_sub(1),
            HumanBytes(payload_size),
            HumanBytes(local_size),
        );
        let _ = window.emit("download_status", "Downloading...");

        let mut stream = db
            .remote()
            .execute_get(connection, local.missing())
            .stream();
        let mut last_update = Instant::now();
        let started = Instant::now();
        let mut stats = Stats::default();
        while let Some(item) = stream.next().await {
            match item {
                GetProgressItem::Progress(offset) => {
                    // Update every 100ms to avoid too many events
                    if last_update.elapsed() > Duration::from_millis(100) {
                        // `offset` counts this session only, so the speed is not
                        // inflated by what a resumed download already had.
                        let current_bytes = local_size + offset;
                        let elapsed = started.elapsed();
                        let speed = if elapsed.as_secs_f64() > 0.0 {
                            (offset as f64 / elapsed.as_secs_f64()) as u64
                        } else {
                            0
                        };
                        let eta_ms = if speed > 0 {
                            let remaining = total_size.saturating_sub(current_bytes);
                            ((remaining as f64 / speed as f64) * 1000.0) as u64
                        } else {
                            0
                        };
                        let _ = window.emit(
                            "transfer_progress",
                            crate::events::TransferProgressEvent {
                                bytes_transferred: current_bytes,
                                total_bytes: total_size,
                                percentage: (current_bytes as f64 / total_size.max(1) as f64
                                    * 100.0)
                                    .min(100.0),
                                speed_bytes_per_sec: speed,
                                elapsed_ms: elapsed.as_millis() as u64,
                                eta_ms,
                            },
                        );
                        last_update = Instant::now();
                    }
                }
                GetProgressItem::Done(value) => {
                    stats = value;
                    break;
                }
                GetProgressItem::Error(cause) => anyhow::bail!(log_get_error(cause)),
            }
        }
        anyhow::Ok(Fetched {
            stats,
            payload_size,
        })
    }
    .await;
    endpoint.close().await;
    result
}

async fn receive_single_blob(
    blob_str: &str,
    file_storage_path: &str,
    verbose: bool,
    window: Events,
) -> anyhow::Result<ReceiveSummary> {
    let ticket = BlobTicket::from_str(blob_str)?;
    info!(
        "Starting file receive from endpoint: {}",
        ticket.addr().id.fmt_short()
    );

    let cfg = config();
    let dir_name = format!(
        "{}get-{}",
        cfg.storage.temp_dir_prefix,
        ticket.hash().to_hex()
    );
    let iroh_data_dir = CWD.join(dir_name);
    let db = FsStore::load(&iroh_data_dir).await?;

    // On failure the store (and its partial download) is kept, so the next
    // try resumes where this one stopped.
    let Fetched {
        stats,
        payload_size,
    } = match fetch_with_retry(&ticket, &db, verbose, &window).await {
        Ok(fetched) => fetched,
        Err(e) => {
            let _ = db.shutdown().await;
            return Err(e);
        }
    };

    let collection = match Collection::load(ticket.hash(), db.as_ref()).await {
        Ok(collection) => collection,
        Err(e) => {
            let _ = db.shutdown().await;
            return Err(e.into());
        }
    };
    let total_files = collection.len();
    let received_name = summary_name(collection.iter().map(|(name, _)| name.as_str()));

    if let Some((name, _)) = collection.iter().next() {
        if let Some(first) = name.split('/').next() {
            info!("Downloading to: {}", first);
        }
    }

    let exported = export(
        &db,
        collection,
        file_storage_path.to_string(),
        window.window(),
    )
    .await;
    // Always drop the work dir: it is named after the content, so a leftover
    // one (e.g. after a cancelled export) would poison the next receive.
    let _ = db.shutdown().await;
    let _ = tokio::fs::remove_dir_all(iroh_data_dir).await;
    exported?;

    info!(
        "Downloaded {} files, {}. Took {}",
        total_files,
        HumanBytes(payload_size),
        HumanDuration(stats.elapsed),
    );

    Ok(ReceiveSummary {
        name: received_name,
        files: total_files as u32,
        bytes: payload_size,
        download_ms: stats.elapsed.as_millis() as u64,
    })
}

#[cfg(test)]
mod code_format_tests {
    use super::looks_like_raw_tickets;

    #[test]
    fn short_numeric_code_is_not_a_raw_ticket() {
        assert!(!looks_like_raw_tickets("123456"));
    }

    #[test]
    fn empty_string_is_not_a_raw_ticket() {
        assert!(!looks_like_raw_tickets(""));
    }

    #[test]
    fn garbage_text_is_not_a_raw_ticket() {
        assert!(!looks_like_raw_tickets("not,a,ticket"));
    }
}

#[cfg(test)]
mod export_name_tests {
    use super::numbered_path;
    use std::path::Path;

    #[test]
    fn numbers_before_the_extension() {
        assert_eq!(
            numbered_path(Path::new("/d/gemini/ai.py"), 1),
            Path::new("/d/gemini/ai (1).py")
        );
        assert_eq!(
            numbered_path(Path::new("/d/Makefile"), 2),
            Path::new("/d/Makefile (2)")
        );
        assert_eq!(
            numbered_path(Path::new("/d/a.tar.gz"), 1),
            Path::new("/d/a.tar (1).gz")
        );
    }
}

#[cfg(test)]
mod receive_summary_tests {
    use super::{summary_name, ReceiveSummary};

    #[test]
    fn names_a_single_file_or_folder() {
        assert_eq!(summary_name(["report.pdf"].into_iter()), "report.pdf");
        assert_eq!(
            summary_name(["gemini/ai.py", "gemini/sub/x.txt"].into_iter()),
            "gemini"
        );
    }

    #[test]
    fn counts_several_top_level_items() {
        assert_eq!(
            summary_name(["a.txt", "b.txt", "dir/c"].into_iter()),
            "3 items"
        );
        assert_eq!(summary_name(std::iter::empty()), "");
    }

    #[test]
    fn combines_tickets() {
        let one = ReceiveSummary {
            name: "a".into(),
            files: 2,
            bytes: 10,
            download_ms: 5,
        };
        assert_eq!(ReceiveSummary::combine(vec![one.clone()]), one);
        let two = ReceiveSummary::combine(vec![one.clone(), one]);
        assert_eq!(
            (two.name.as_str(), two.files, two.bytes, two.download_ms),
            ("2 items", 4, 20, 10)
        );
    }
}

/// Runs the real send and receive code against each other over loopback: no
/// window, no relay, no internet.
#[cfg(test)]
mod roundtrip_tests {
    use super::*;

    /// Points HOME at a temp dir before the app's lazy statics read it, so the
    /// tests never touch the real Documents folder.
    fn isolated_home() -> &'static std::path::Path {
        static HOME: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
        HOME.get_or_init(|| {
            let dir = tempfile::tempdir().expect("temp dir");
            #[cfg(unix)]
            std::env::set_var("HOME", dir.path());
            crate::config::AppConfig::init();
            dir
        })
        .path()
    }

    async fn share(path: &Path) -> (BlobTicket, iroh::protocol::Router, PathBuf) {
        let args = SendArgs {
            path: path.to_path_buf(),
            ticket_type: AddrInfoOptions::RelayAndAddresses,
            common: CommonArgs {
                magic_ipv4_addr: None,
                magic_ipv6_addr: None,
                format: Format::Hex,
                verbose: 0,
                relay: RelayModeOption::Disabled,
            },
        };
        start_send(args, None).await.expect("start sharing")
    }

    /// Manual: shares SHARE_PATH with the default relays and prints the ticket, for
    /// testing a real device against this machine. Run with
    /// `SHARE_PATH=... cargo test manual_share -- --ignored --nocapture`.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "manual device test"]
    async fn manual_share() {
        let _ = isolated_home();
        let path = std::env::var("SHARE_PATH").expect("set SHARE_PATH");
        let args = SendArgs {
            path: PathBuf::from(path),
            ticket_type: AddrInfoOptions::RelayAndAddresses,
            common: CommonArgs {
                magic_ipv4_addr: None,
                magic_ipv6_addr: None,
                format: Format::Hex,
                verbose: 0,
                relay: RelayModeOption::Default,
            },
        };
        let (ticket, _router, _dir) = start_send(args, None).await.expect("share");
        println!("TICKET={ticket}");
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }

    /// Manual: receives TICKET into DEST (counterpart of `manual_share`).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "manual device test"]
    async fn manual_receive() {
        let _ = isolated_home();
        let ticket = std::env::var("TICKET").expect("set TICKET");
        let dest = std::env::var("DEST").expect("set DEST");
        let summary = receive_file_with_progress(ticket, dest, false, Events::none())
            .await
            .expect("receive");
        println!("RECEIVED={summary:?}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn a_folder_arrives_intact_and_a_second_copy_is_numbered() {
        let home = isolated_home();
        let src = home.join("folder-to-share");
        std::fs::create_dir_all(src.join("sub/deep")).unwrap();
        std::fs::write(src.join("a.txt"), b"hello").unwrap();
        let big: Vec<u8> = (0..3_000_000u32).map(|i| (i % 251) as u8).collect();
        std::fs::write(src.join("sub/big.bin"), &big).unwrap();
        std::fs::write(src.join("sub/deep/c.txt"), b"deep").unwrap();

        let (ticket, router, work_dir) = share(&src).await;
        let dest = home.join("received");
        let receive = || {
            receive_file_with_progress(
                ticket.to_string(),
                dest.to_string_lossy().into_owned(),
                false,
                Events::none(),
            )
        };

        let summary = receive().await.expect("first receive");
        assert_eq!(summary.name, "folder-to-share");
        assert_eq!(summary.files, 3);
        assert_eq!(summary.bytes, 5 + 3_000_000 + 4);
        let root = dest.join("folder-to-share");
        assert_eq!(std::fs::read(root.join("a.txt")).unwrap(), b"hello");
        assert_eq!(std::fs::read(root.join("sub/big.bin")).unwrap(), big);
        assert_eq!(std::fs::read(root.join("sub/deep/c.txt")).unwrap(), b"deep");

        // Receiving the same thing again must not overwrite: with no window to
        // ask, the files are kept side by side with a number.
        receive().await.expect("second receive");
        assert_eq!(std::fs::read(root.join("a (1).txt")).unwrap(), b"hello");
        assert_eq!(std::fs::read(root.join("a.txt")).unwrap(), b"hello");

        router.shutdown().await.ok();
        let _ = std::fs::remove_dir_all(work_dir);
    }
}

#[cfg(test)]
mod http_tests {
    use super::*;

    /// Run with `cargo test -- --ignored`. HTTPS needs the TLS provider installed in
    /// `http_client`; without it reqwest panics, which would break short codes.
    #[tokio::test]
    #[ignore = "needs internet"]
    async fn https_to_the_code_server_works() {
        let reply = http_client(Duration::from_secs(10))
            .get("https://sendme-codes.sendme-share.workers.dev/")
            .send()
            .await
            .expect("HTTPS request")
            .text()
            .await
            .expect("body");
        assert_eq!(reply, "ok");
    }

    #[test]
    fn building_the_client_twice_does_not_panic() {
        let _ = http_client(Duration::from_secs(1));
        let _ = http_client(Duration::from_secs(1));
    }
}
