//! Command line arguments.

use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
    net::{SocketAddrV4, SocketAddrV6},
    path::{Component, Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use clap::{Parser, Subcommand};
use console::style;
use data_encoding::HEXLOWER;
use futures_buffered::BufferedStreamExt;
use indicatif::{
    HumanBytes, HumanDuration, MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle,
};
use iroh::{
    discovery::{dns::DnsDiscovery, pkarr::PkarrPublisher},
    Endpoint, NodeAddr, RelayMap, RelayMode, RelayUrl, SecretKey,
};
use iroh_blobs::{
    format::collection::Collection,
    get::{
        db::DownloadProgress,
        fsm::{AtBlobHeaderNextError, DecodeError},
        request::get_hash_seq_and_sizes,
    },
    net_protocol::Blobs,
    provider::{self, CustomEventSender},
    store::{ExportMode, ImportMode, ImportProgress},
    ticket::BlobTicket,
    BlobFormat, Hash, HashAndFormat, TempTag,
};
use n0_future::{future::Boxed, StreamExt};
use rand::Rng;
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
        use std::path::PathBuf;
        let mut path = if let Ok(external_storage) = std::env::var("EXTERNAL_STORAGE") {
            let mut p = PathBuf::from(external_storage);
            p.push("Android/data");
            p.push(&cfg.platform.android_package_name);
            p.push("files");
            p
        } else {
            let mut p = PathBuf::from(&cfg.platform.sdcard_fallback_path);
            p.push("Android/data");
            p.push(&cfg.platform.android_package_name);
            p.push("files");
            p
        };
        path.push(format!("{}temp", cfg.storage.temp_dir_prefix));
        path
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

impl From<RelayModeOption> for RelayMode {
    fn from(value: RelayModeOption) -> Self {
        match value {
            RelayModeOption::Disabled => RelayMode::Disabled,
            RelayModeOption::Default => RelayMode::Default,
            RelayModeOption::Custom(url) => RelayMode::Custom(RelayMap::from_url(url)),
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

fn apply_options(addr: &mut NodeAddr, opts: AddrInfoOptions) {
    match opts {
        AddrInfoOptions::Id => {
            addr.direct_addresses.clear();
            addr.relay_url = None;
        }
        AddrInfoOptions::RelayAndAddresses => {
            // nothing to do
        }
        AddrInfoOptions::Relay => {
            addr.direct_addresses.clear();
        }
        AddrInfoOptions::Addresses => {
            addr.relay_url = None;
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
            let key = SecretKey::generate(rand::rngs::OsRng);
            if print {
                info!("Using secret key: {}", key);
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

pub async fn show_ingest_progress(
    recv: async_channel::Receiver<ImportProgress>,
) -> anyhow::Result<()> {
    let mp = MultiProgress::new();
    mp.set_draw_target(ProgressDrawTarget::stderr());
    let op = mp.add(ProgressBar::hidden());
    op.set_style(
        ProgressStyle::default_spinner().template("{spinner:.green} [{elapsed_precise}] {msg}")?,
    );
    // op.set_message(format!("{} Ingesting ...\n", style("[1/2]").bold().dim()));
    // op.set_length(total_files);
    let mut names = BTreeMap::new();
    let mut sizes = BTreeMap::new();
    let mut pbs = BTreeMap::new();
    loop {
        let event = recv.recv().await;
        match event {
            Ok(ImportProgress::Found { id, name }) => {
                names.insert(id, name);
            }
            Ok(ImportProgress::Size { id, size }) => {
                sizes.insert(id, size);
                let total_size = sizes.values().sum::<u64>();
                op.set_message(format!(
                    "{} Ingesting {} files, {}\n",
                    style("[1/2]").bold().dim(),
                    sizes.len(),
                    HumanBytes(total_size)
                ));
                let name = names.get(&id).cloned().unwrap_or_default();
                let pb = mp.add(ProgressBar::hidden());
                pb.set_style(ProgressStyle::with_template(
                    "{msg}{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}",
                )?.progress_chars("#>-"));
                pb.set_message(format!("{} {}", style("[2/2]").bold().dim(), name));
                pb.set_length(size);
                pbs.insert(id, pb);
            }
            Ok(ImportProgress::OutboardProgress { id, offset }) => {
                if let Some(pb) = pbs.get(&id) {
                    pb.set_position(offset);
                }
            }
            Ok(ImportProgress::OutboardDone { id, .. }) => {
                // you are not guaranteed to get any OutboardProgress
                if let Some(pb) = pbs.remove(&id) {
                    pb.finish_and_clear();
                }
            }
            Ok(ImportProgress::CopyProgress { .. }) => {
                // we are not copying anything
            }
            Err(e) => {
                op.set_message(format!("Error receiving progress: {e}"));
                break;
            }
        }
    }
    op.finish_and_clear();
    Ok(())
}

/// Show ingest progress with Tauri window events
pub async fn show_ingest_progress_with_window(
    recv: async_channel::Receiver<ImportProgress>,
    window: Option<tauri::Window>,
    total_files: usize,
) -> anyhow::Result<()> {
    let mp = MultiProgress::new();
    mp.set_draw_target(ProgressDrawTarget::stderr());
    let op = mp.add(ProgressBar::hidden());
    op.set_style(
        ProgressStyle::default_spinner().template("{spinner:.green} [{elapsed_precise}] {msg}")?,
    );

    let mut names = BTreeMap::new();
    let mut sizes = BTreeMap::new();
    let mut pbs = BTreeMap::new();
    let mut files_processed = 0;

    loop {
        let event = recv.recv().await;
        match event {
            Ok(ImportProgress::Found { id, name }) => {
                names.insert(id, name.clone());

                // Emit event to window
                if let Some(ref win) = window {
                    let _ = win.emit(
                        "import_progress",
                        crate::events::ImportProgressEvent {
                            percentage: (files_processed as f64 / total_files as f64 * 100.0)
                                .min(100.0),
                            current_file: name,
                            files_processed,
                            total_files,
                        },
                    );
                }
            }
            Ok(ImportProgress::Size { id, size }) => {
                sizes.insert(id, size);
                let total_size = sizes.values().sum::<u64>();
                op.set_message(format!(
                    "{} Ingesting {} files, {}\n",
                    style("[1/2]").bold().dim(),
                    sizes.len(),
                    HumanBytes(total_size)
                ));
                let name = names.get(&id).cloned().unwrap_or_default();
                let pb = mp.add(ProgressBar::hidden());
                pb.set_style(ProgressStyle::with_template(
                    "{msg}{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}",
                )?.progress_chars("#>-"));
                pb.set_message(format!("{} {}", style("[2/2]").bold().dim(), name));
                pb.set_length(size);
                pbs.insert(id, pb);
            }
            Ok(ImportProgress::OutboardProgress { id, offset }) => {
                if let Some(pb) = pbs.get(&id) {
                    pb.set_position(offset);
                }
            }
            Ok(ImportProgress::OutboardDone { id, .. }) => {
                // you are not guaranteed to get any OutboardProgress
                if let Some(pb) = pbs.remove(&id) {
                    pb.finish_and_clear();
                }

                files_processed += 1;

                // Emit updated progress
                if let Some(ref win) = window {
                    let current_name = names.get(&id).cloned().unwrap_or_default();
                    let _ = win.emit(
                        "import_progress",
                        crate::events::ImportProgressEvent {
                            percentage: (files_processed as f64 / total_files as f64 * 100.0)
                                .min(100.0),
                            current_file: current_name,
                            files_processed,
                            total_files,
                        },
                    );
                }
            }
            Ok(ImportProgress::CopyProgress { .. }) => {
                // we are not copying anything
            }
            Err(e) => {
                op.set_message(format!("Error receiving progress: {e}"));
                break;
            }
        }
    }
    op.finish_and_clear();
    Ok(())
}

/// Import from a file or directory into the database.
///
/// The returned tag always refers to a collection. If the input is a file, this
/// is a collection with a single blob, named like the file.
///
/// If the input is a directory, the collection contains all the files in the
/// directory.
#[allow(dead_code)]
async fn import(
    path: PathBuf,
    db: impl iroh_blobs::store::Store,
) -> anyhow::Result<(TempTag, u64, Collection)> {
    import_with_window(path, db, None).await
}

/// Import with optional window for emitting progress events
async fn import_with_window(
    path: PathBuf,
    db: impl iroh_blobs::store::Store,
    window: Option<tauri::Window>,
) -> anyhow::Result<(TempTag, u64, Collection)> {
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
    let (send, recv) = async_channel::bounded(32);
    let progress = iroh_blobs::util::progress::AsyncChannelProgressSender::new(send);

    // Clone window for progress task if available
    let window_clone = window.clone();
    let show_progress = if window_clone.is_some() {
        tokio::spawn(show_ingest_progress_with_window(
            recv,
            window_clone,
            total_files,
        ))
    } else {
        tokio::spawn(show_ingest_progress(recv))
    };

    // import all the files, using num_cpus workers, return names and temp tags
    let mut names_and_tags = futures_lite::stream::iter(data_sources)
        .map(|(name, path)| {
            let db = db.clone();
            let progress = progress.clone();
            async move {
                let (temp_tag, file_size) = db
                    .import_file(path, ImportMode::TryReference, BlobFormat::Raw, progress)
                    .await?;
                anyhow::Ok((name, temp_tag, file_size))
            }
        })
        .buffered_unordered(num_cpus::get())
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;
    drop(progress);
    names_and_tags.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));
    // total size of all files
    let size = names_and_tags.iter().map(|(_, _, size)| *size).sum::<u64>();
    // collect the (name, hash) tuples into a collection
    // we must also keep the tags around so the data does not get gced.
    let (collection, tags) = names_and_tags
        .into_iter()
        .map(|(name, tag, _)| ((name, *tag.hash()), tag))
        .unzip::<_, _, Collection, Vec<_>>();
    let temp_tag = collection.clone().store(&db).await?;
    // now that the collection is stored, we can drop the tags
    // data is protected by the collection
    drop(tags);
    show_progress.await??;
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

async fn export(
    db: impl iroh_blobs::store::Store,
    collection: Collection,
    storage_file_path: String,
) -> anyhow::Result<()> {
    let root = PathBuf::from(storage_file_path);
    for (name, hash) in collection.iter() {
        let target = get_export_path(&root, name)?;
        if target.exists() {
            error!("Target {} already exists. Export stopped", target.display());
            warn!("You can remove the file or directory and try again. The download will not be repeated");
            anyhow::bail!("target {} already exists", target.display());
        }
        db.export(
            *hash,
            target,
            ExportMode::TryReference,
            Box::new(move |_position| Ok(())),
        )
        .await?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct SendStatus {
    /// the multiprogress bar
    mp: MultiProgress,
}

impl SendStatus {
    fn new() -> Self {
        let mp = MultiProgress::new();
        mp.set_draw_target(ProgressDrawTarget::stderr());
        Self { mp }
    }

    fn new_client(&self) -> ClientStatus {
        let current = self.mp.add(ProgressBar::hidden());
        current.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap(),
        );
        current.enable_steady_tick(Duration::from_millis(100));
        current.set_message("waiting for requests");
        ClientStatus {
            current: current.into(),
        }
    }
}

#[derive(Debug, Clone)]
struct ClientStatus {
    current: Arc<ProgressBar>,
}

impl Drop for ClientStatus {
    fn drop(&mut self) {
        if Arc::strong_count(&self.current) == 1 {
            self.current.finish_and_clear();
        }
    }
}

impl CustomEventSender for ClientStatus {
    fn send(&self, event: iroh_blobs::provider::Event) -> Boxed<()> {
        self.try_send(event);
        Box::pin(std::future::ready(()))
    }

    fn try_send(&self, event: provider::Event) {
        tracing::info!("{:?}", event);
        let msg = match event {
            provider::Event::ClientConnected { connection_id } => {
                Some(format!("{} got connection", connection_id))
            }
            provider::Event::TransferBlobCompleted {
                connection_id,
                hash,
                index,
                size,
                ..
            } => Some(format!(
                "{} transfer blob completed {} {} {}",
                connection_id,
                hash,
                index,
                HumanBytes(size)
            )),
            provider::Event::TransferCompleted {
                connection_id,
                stats,
                ..
            } => Some(format!(
                "{} transfer completed {} {}",
                connection_id,
                stats.send.write_bytes.size,
                HumanDuration(stats.send.write_bytes.stats.duration)
            )),
            provider::Event::TransferAborted { connection_id, .. } => {
                Some(format!("{} transfer completed", connection_id))
            }
            _ => None,
        };
        if let Some(msg) = msg {
            self.current.set_message(msg);
        }
    }
}

// Extended ClientStatus that can emit Tauri events
#[derive(Clone)]
struct TauriClientStatus {
    current: Arc<ProgressBar>,
    window: Option<tauri::Window>,
    total_size: Arc<std::sync::Mutex<u64>>,
    start_time: Arc<std::sync::Mutex<Option<Instant>>>,
}

// Manual Debug implementation since tauri::Window doesn't implement Debug
impl std::fmt::Debug for TauriClientStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TauriClientStatus")
            .field("window", &self.window.is_some())
            .field("total_size", &self.total_size)
            .field("start_time", &self.start_time)
            .finish()
    }
}

impl TauriClientStatus {
    fn new(mp: &MultiProgress, window: Option<tauri::Window>, total_size: u64) -> Self {
        let current = mp.add(ProgressBar::hidden());
        current.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap(),
        );
        current.enable_steady_tick(Duration::from_millis(100));
        current.set_message("waiting for requests");

        Self {
            current: current.into(),
            window,
            total_size: Arc::new(std::sync::Mutex::new(total_size)),
            start_time: Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

impl Drop for TauriClientStatus {
    fn drop(&mut self) {
        if Arc::strong_count(&self.current) == 1 {
            self.current.finish_and_clear();
        }
    }
}

impl CustomEventSender for TauriClientStatus {
    fn send(&self, event: iroh_blobs::provider::Event) -> Boxed<()> {
        self.try_send(event);
        Box::pin(std::future::ready(()))
    }

    fn try_send(&self, event: provider::Event) {
        tracing::info!("TauriClientStatus event: {:?}", event);

        match &event {
            provider::Event::ClientConnected { connection_id } => {
                let msg = format!("{} got connection", connection_id);
                self.current.set_message(msg.clone());

                // Initialize start time
                if let Ok(mut start) = self.start_time.lock() {
                    *start = Some(Instant::now());
                }

                if let Some(ref window) = self.window {
                    let _ = window.emit("send_transfer_started", ());
                }
            }
            provider::Event::TransferBlobCompleted {
                connection_id,
                hash,
                index,
                size,
                ..
            } => {
                let msg = format!(
                    "{} transfer blob completed {} {} {}",
                    connection_id,
                    hash,
                    index,
                    HumanBytes(*size)
                );
                self.current.set_message(msg);

                // Emit progress event
                if let Some(ref window) = self.window {
                    if let Ok(start) = self.start_time.lock() {
                        if let Some(start_instant) = *start {
                            let elapsed = start_instant.elapsed();
                            let elapsed_ms = elapsed.as_millis() as u64;

                            if let Ok(total_size) = self.total_size.lock() {
                                // We can't track exact bytes transferred easily, but we can estimate
                                // based on blob completion. For now, emit an event on each blob completion.
                                let speed = if elapsed.as_secs_f64() > 0.0 {
                                    (*size as f64 / elapsed.as_secs_f64()) as u64
                                } else {
                                    0
                                };

                                let _ = window.emit(
                                    "send_transfer_progress",
                                    crate::events::TransferProgressEvent {
                                        bytes_transferred: *size,
                                        total_bytes: *total_size,
                                        percentage: (*size as f64 / *total_size as f64 * 100.0)
                                            .min(100.0),
                                        speed_bytes_per_sec: speed,
                                        elapsed_ms,
                                        eta_ms: 0, // Hard to estimate for sender
                                    },
                                );
                            }
                        }
                    }
                }
            }
            provider::Event::TransferCompleted {
                connection_id,
                stats,
                ..
            } => {
                let msg = format!(
                    "{} transfer completed {} {}",
                    connection_id,
                    stats.send.write_bytes.size,
                    HumanDuration(stats.send.write_bytes.stats.duration)
                );
                self.current.set_message(msg);

                if let Some(ref window) = self.window {
                    if let Ok(total_size) = self.total_size.lock() {
                        let _ = window.emit(
                            "send_transfer_progress",
                            crate::events::TransferProgressEvent {
                                bytes_transferred: stats.send.write_bytes.size,
                                total_bytes: *total_size,
                                percentage: 100.0,
                                speed_bytes_per_sec: 0,
                                elapsed_ms: stats.send.write_bytes.stats.duration.as_millis()
                                    as u64,
                                eta_ms: 0,
                            },
                        );
                    }
                }
            }
            provider::Event::TransferAborted { connection_id, .. } => {
                let msg = format!("{} transfer aborted", connection_id);
                self.current.set_message(msg);
            }
            _ => {}
        }
    }
}

#[allow(dead_code)]
fn make_download_progress() -> ProgressBar {
    let pb = ProgressBar::hidden();
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb.set_style(
        ProgressStyle::with_template(
            "{msg}{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} {binary_bytes_per_sec}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    pb
}

// use serde::{Deserialize, Serialize};
use std::error::Error;

use reqwest::Client;

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

    let client = Client::new();
    let payload = BlobRequest { blobs };

    info!(
        "Creating {} blobs using BASE_URL: {}",
        payload.blobs.len(),
        cfg.api.base_url
    );

    let response = client
        .post(format!("{}/api/code/", cfg.api.base_url))
        .json(&payload)
        .send()
        .await?
        .json::<BlobResponse>()
        .await?;

    info!("Blobs created successfully with code: {}", response.code);
    Ok(response.code)
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

    let client = Client::new();

    debug!("Fetching blob with code: {}", code);

    let url: String = format!("{}/api/code/{}", cfg.api.base_url, code);

    let response = client
        .get(&url)
        .send()
        .await?
        .json::<GetBlobResponse>()
        .await?;

    info!("Successfully fetched {} blob(s)", response.blobs.len());
    Ok(response.blobs)
}

#[allow(dead_code)]
pub async fn show_download_progress(
    recv: async_channel::Receiver<DownloadProgress>,
    total_size: u64,
) -> anyhow::Result<()> {
    let mp = MultiProgress::new();
    mp.set_draw_target(ProgressDrawTarget::stderr());
    let op = mp.add(make_download_progress());
    op.set_message(format!("{} Connecting ...\n", style("[1/3]").bold().dim()));
    let mut total_done = 0;
    let mut sizes = BTreeMap::new();
    loop {
        let x = recv.recv().await;
        match x {
            Ok(DownloadProgress::Connected) => {
                op.set_message(format!("{} Requesting ...\n", style("[2/3]").bold().dim()));
            }
            Ok(DownloadProgress::FoundHashSeq { children, .. }) => {
                op.set_message(format!(
                    "{} Downloading {} blob(s)\n",
                    style("[3/3]").bold().dim(),
                    children + 1,
                ));
                op.set_length(total_size);
                op.reset();
            }
            Ok(DownloadProgress::Found { id, size, .. }) => {
                sizes.insert(id, size);
            }
            Ok(DownloadProgress::Progress { offset, .. }) => {
                op.set_position(total_done + offset);
            }
            Ok(DownloadProgress::Done { id }) => {
                total_done += sizes.remove(&id).unwrap_or_default();
            }
            Ok(DownloadProgress::AllDone(stats)) => {
                op.finish_and_clear();
                eprintln!(
                    "Transferred {} in {}, {}/s",
                    HumanBytes(stats.bytes_read),
                    HumanDuration(stats.elapsed),
                    HumanBytes((stats.bytes_read as f64 / stats.elapsed.as_secs_f64()) as u64)
                );
                break;
            }
            Ok(DownloadProgress::Abort(e)) => {
                anyhow::bail!("download aborted: {e:?}");
            }
            Err(e) => {
                anyhow::bail!("error reading progress: {e:?}");
            }
            _ => {}
        }
    }
    Ok(())
}

fn show_get_error(e: anyhow::Error) -> anyhow::Error {
    if let Some(err) = e.downcast_ref::<DecodeError>() {
        match err {
            DecodeError::NotFound => {
                eprintln!("{}", style("send side no longer has a file").yellow())
            }
            DecodeError::LeafNotFound(_) | DecodeError::ParentNotFound(_) => eprintln!(
                "{}",
                style("send side no longer has part of a file").yellow()
            ),
            DecodeError::Io(err) => eprintln!(
                "{}",
                style(format!("generic network error: {}", err)).yellow()
            ),
            DecodeError::Read(err) => eprintln!(
                "{}",
                style(format!("error reading data from quinn: {}", err)).yellow()
            ),
            DecodeError::LeafHashMismatch(_) | DecodeError::ParentHashMismatch(_) => {
                eprintln!("{}", style("send side sent wrong data").red())
            }
        };
    } else if let Some(header_error) = e.downcast_ref::<AtBlobHeaderNextError>() {
        // TODO(iroh-bytes): get_to_db should have a concrete error type so you don't have to guess
        match header_error {
            AtBlobHeaderNextError::Io(err) => eprintln!(
                "{}",
                style(format!("generic network error: {}", err)).yellow()
            ),
            AtBlobHeaderNextError::Read(err) => eprintln!(
                "{}",
                style(format!("error reading data from quinn: {}", err)).yellow()
            ),
            AtBlobHeaderNextError::NotFound => {
                eprintln!("{}", style("send side no longer has a file").yellow())
            }
        };
    } else {
        eprintln!(
            "{}",
            style(format!("generic error: {:?}", e.root_cause())).red()
        );
    }
    e
}

#[allow(dead_code)]
pub async fn receive(args: ReceiveArgs, storage_file_path: String) -> anyhow::Result<()> {
    let cfg = config();
    let ticket = args.ticket;
    let addr = ticket.node_addr().clone();

    info!("Starting file receive from node: {}", addr.node_id);

    let secret_key = get_or_create_secret(args.common.verbose > 0)?;
    let mut builder = Endpoint::builder()
        .alpns(vec![])
        .secret_key(secret_key)
        .relay_mode(args.common.relay.into());

    if ticket.node_addr().relay_url.is_none() && ticket.node_addr().direct_addresses.is_empty() {
        builder = builder.add_discovery(|_| Some(DnsDiscovery::n0_dns()));
    }
    if let Some(addr) = args.common.magic_ipv4_addr {
        builder = builder.bind_addr_v4(addr);
    }
    if let Some(addr) = args.common.magic_ipv6_addr {
        builder = builder.bind_addr_v6(addr);
    }
    let endpoint = builder.bind().await?;
    let dir_name = format!(
        "{}get-{}",
        cfg.storage.temp_dir_prefix,
        ticket.hash().to_hex()
    );
    let iroh_data_dir = CWD.join(dir_name);
    let db = iroh_blobs::store::fs::Store::load(&iroh_data_dir).await?;
    let mp = MultiProgress::new();
    let connect_progress = mp.add(ProgressBar::hidden());
    connect_progress.set_draw_target(ProgressDrawTarget::stderr());
    connect_progress.set_style(ProgressStyle::default_spinner());
    connect_progress.set_message(format!("connecting to {}", addr.node_id));
    let connection = endpoint.connect(addr, iroh_blobs::protocol::ALPN).await?;
    let hash_and_format = HashAndFormat {
        hash: ticket.hash(),
        format: ticket.format(),
    };
    connect_progress.finish_and_clear();
    let (send, recv) = async_channel::bounded(32);
    let progress = iroh_blobs::util::progress::AsyncChannelProgressSender::new(send);
    let (_hash_seq, sizes) =
        get_hash_seq_and_sizes(&connection, &hash_and_format.hash, 1024 * 1024 * 32)
            .await
            .map_err(show_get_error)?;
    let total_size = sizes.iter().sum::<u64>();
    let total_files = sizes.len().saturating_sub(1);
    let payload_size = sizes.iter().skip(1).sum::<u64>();
    info!(
        "Getting collection {} {} files, {}",
        print_hash(&ticket.hash(), args.common.format),
        total_files,
        HumanBytes(payload_size)
    );
    // print the details of the collection only in verbose mode
    if args.common.verbose > 0 {
        debug!(
            "Getting {} blobs in total, {}",
            sizes.len(),
            HumanBytes(total_size)
        );
    }
    let _task = tokio::spawn(show_download_progress(recv, total_size));
    let get_conn = || async move { Ok(connection) };
    let stats = iroh_blobs::get::db::get_to_db(&db, get_conn, &hash_and_format, progress)
        .await
        .map_err(|e| show_get_error(anyhow::anyhow!(e)))?;
    let collection = Collection::load_db(&db, &hash_and_format.hash).await?;
    if args.common.verbose > 0 {
        for (name, hash) in collection.iter() {
            debug!("    {} {name}", print_hash(hash, args.common.format));
        }
    }
    if let Some((name, _)) = collection.iter().next() {
        if let Some(first) = name.split('/').next() {
            info!("Downloading to: {}", first);
        }
    }
    export(db, collection, storage_file_path.clone()).await?;
    tokio::fs::remove_dir_all(iroh_data_dir).await?;
    if args.common.verbose > 0 {
        info!(
            "Downloaded {} files, {}. Took {} ({}/s)",
            total_files,
            HumanBytes(payload_size),
            HumanDuration(stats.elapsed),
            HumanBytes((stats.bytes_read as f64 / stats.elapsed.as_secs_f64()) as u64),
        );
    }
    Ok(())
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
    let mut builder = Endpoint::builder()
        .alpns(vec![iroh_blobs::protocol::ALPN.to_vec()])
        .secret_key(secret_key)
        .relay_mode(args.common.relay.into());

    if args.ticket_type == AddrInfoOptions::Id {
        builder =
            builder.add_discovery(|secret_key| Some(PkarrPublisher::n0_dns(secret_key.clone())));
    }
    if let Some(addr) = args.common.magic_ipv4_addr {
        builder = builder.bind_addr_v4(addr);
    }
    if let Some(addr) = args.common.magic_ipv6_addr {
        builder = builder.bind_addr_v6(addr);
    }

    // Create a unique directory for this sending session.
    debug!("Creating blobs data directory");
    let suffix = rand::thread_rng().gen::<[u8; 16]>();
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

    info!("Importing file: {:?}", args.path);
    let path = args.path;
    // Import first to get the size, which we'll need for progress tracking
    // Use import_with_window to emit progress events
    let db_store = iroh_blobs::store::fs::Store::load(&blobs_data_dir).await?;
    let (temp_tag, size, collection) =
        import_with_window(path.clone(), db_store, window.clone()).await?;
    info!("Import complete, size: {}", HumanBytes(size));
    let hash = *temp_tag.hash();

    debug!("Creating blobs store");
    let blobs = if let Some(ref win) = window {
        // Use TauriClientStatus for progress tracking
        let mp = MultiProgress::new();
        mp.set_draw_target(ProgressDrawTarget::stderr());
        let client_status = TauriClientStatus::new(&mp, Some(win.clone()), size);

        Blobs::persistent(&blobs_data_dir)
            .await?
            .events(client_status.into())
            .build(&endpoint)
    } else {
        // Use regular SendStatus without window
        let ps = SendStatus::new();
        Blobs::persistent(&blobs_data_dir)
            .await?
            .events(ps.new_client().into())
            .build(&endpoint)
    };

    debug!("Building router");
    let router = iroh::protocol::Router::builder(endpoint)
        .accept(iroh_blobs::ALPN, blobs.clone())
        .spawn()
        .await?;

    // Wait for the endpoint to be ready.
    debug!("Waiting for relay initialization");
    let _ = router.endpoint().home_relay().initialized().await?;
    debug!("Relay initialized");

    debug!("Getting node address");
    let mut addr = router.endpoint().node_addr().await?;
    apply_options(&mut addr, args.ticket_type);
    let ticket = BlobTicket::new(addr, hash, BlobFormat::HashSeq)?;
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
pub async fn receive_file_minimal(
    ticket: String,
    file_storage_path: String,
    verbose: bool,
) -> anyhow::Result<String> {
    debug!("Receiving file from ticket: {}", ticket);

    let blob = match get_blob(ticket).await {
        Ok(blob) => blob,
        Err(err) => {
            error!("Failed to get blob: {}", err);
            return Err(anyhow::anyhow!("Failed to get blob: {}", err));
        }
    };

    // Parse the ticket string into a BlobTicket.
    // let ticket = BlobTicket::from_str(&ticket)?;
    let ticket = BlobTicket::from_str(&blob[0])?;

    // Construct a minimal ReceiveArgs using the ticket and verbose flag.
    let receive_args = ReceiveArgs {
        ticket,
        common: CommonArgs {
            magic_ipv4_addr: None,
            magic_ipv6_addr: None,
            format: Format::Hex,
            verbose: if verbose { 1 } else { 0 },
            relay: RelayModeOption::Default,
        },
    };

    let res = receive(receive_args, file_storage_path).await;

    match res {
        Ok(()) => Ok("success".to_string()),
        Err(e) => Err(e),
    }
}

pub async fn receive_file_with_progress(
    ticket: String,
    file_storage_path: String,
    verbose: bool,
    window: tauri::Window,
) -> anyhow::Result<String> {
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
    for (idx, blob_str) in blobs.iter().enumerate() {
        info!("Downloading blob {}/{}", idx + 1, blobs.len());
        let _ = window.emit(
            "download_status",
            format!("Downloading file {}/{}...", idx + 1, blobs.len()),
        );

        // Download this blob
        receive_single_blob(blob_str, &file_storage_path, verbose, window.clone()).await?;
    }

    info!("Successfully downloaded all {} file(s)", blobs.len());
    Ok("success".to_string())
}

async fn receive_single_blob(
    blob_str: &str,
    file_storage_path: &str,
    verbose: bool,
    window: tauri::Window,
) -> anyhow::Result<()> {
    let ticket = BlobTicket::from_str(blob_str)?;
    let addr = ticket.node_addr().clone();

    info!("Starting file receive from node: {}", addr.node_id);
    let _ = window.emit("download_status", "Initializing connection...");

    let secret_key = get_or_create_secret(verbose)?;
    let mut builder = Endpoint::builder()
        .alpns(vec![])
        .secret_key(secret_key)
        .relay_mode(RelayModeOption::Default.into());

    if ticket.node_addr().relay_url.is_none() && ticket.node_addr().direct_addresses.is_empty() {
        builder = builder.add_discovery(|_| Some(DnsDiscovery::n0_dns()));
    }

    let endpoint = builder.bind().await?;
    let cfg = config();
    let dir_name = format!(
        "{}get-{}",
        cfg.storage.temp_dir_prefix,
        ticket.hash().to_hex()
    );
    let iroh_data_dir = CWD.join(dir_name);
    let db = iroh_blobs::store::fs::Store::load(&iroh_data_dir).await?;

    let _ = window.emit("download_status", "Connecting to sender...");
    let connection = endpoint.connect(addr, iroh_blobs::protocol::ALPN).await?;
    let hash_and_format = HashAndFormat {
        hash: ticket.hash(),
        format: ticket.format(),
    };

    let (send, recv) = async_channel::bounded(32);
    let progress = iroh_blobs::util::progress::AsyncChannelProgressSender::new(send);
    let (_hash_seq, sizes) =
        get_hash_seq_and_sizes(&connection, &hash_and_format.hash, 1024 * 1024 * 32)
            .await
            .map_err(show_get_error)?;

    let total_size = sizes.iter().sum::<u64>();
    let total_files = sizes.len().saturating_sub(1);
    let payload_size = sizes.iter().skip(1).sum::<u64>();

    info!(
        "Getting collection {} {} files, {}",
        print_hash(&ticket.hash(), Format::Hex),
        total_files,
        HumanBytes(payload_size)
    );

    let _ = window.emit(
        "download_status",
        format!("Downloading {} files...", total_files),
    );

    // Spawn task to monitor progress and emit events
    let window_clone = window.clone();
    let start_time = Instant::now();
    let _task = tokio::spawn(async move {
        let mut total_done = 0u64;
        let mut sizes_map = BTreeMap::new();
        let mut last_update = Instant::now();

        loop {
            let x = recv.recv().await;
            match x {
                Ok(DownloadProgress::Connected) => {
                    let _ = window_clone.emit("download_status", "Connected, requesting data...");
                }
                Ok(DownloadProgress::FoundHashSeq { .. }) => {
                    let _ = window_clone.emit("download_status", "Downloading...");
                }
                Ok(DownloadProgress::Found { id, size, .. }) => {
                    sizes_map.insert(id, size);
                }
                Ok(DownloadProgress::Progress { offset, .. }) => {
                    let current_bytes = total_done + offset;
                    let elapsed = start_time.elapsed();
                    let elapsed_ms = elapsed.as_millis() as u64;

                    // Update every 100ms to avoid too many events
                    if last_update.elapsed().as_millis() > 100 {
                        let percentage = (current_bytes as f64 / total_size as f64) * 100.0;
                        let speed = if elapsed.as_secs_f64() > 0.0 {
                            (current_bytes as f64 / elapsed.as_secs_f64()) as u64
                        } else {
                            0
                        };

                        let eta_ms = if speed > 0 {
                            let remaining_bytes = total_size.saturating_sub(current_bytes);
                            ((remaining_bytes as f64 / speed as f64) * 1000.0) as u64
                        } else {
                            0
                        };

                        let _ = window_clone.emit(
                            "transfer_progress",
                            crate::events::TransferProgressEvent {
                                bytes_transferred: current_bytes,
                                total_bytes: total_size,
                                percentage,
                                speed_bytes_per_sec: speed,
                                elapsed_ms,
                                eta_ms,
                            },
                        );

                        last_update = Instant::now();
                    }
                }
                Ok(DownloadProgress::Done { id }) => {
                    total_done += sizes_map.remove(&id).unwrap_or_default();
                }
                Ok(DownloadProgress::AllDone(_stats)) => {
                    break;
                }
                Ok(DownloadProgress::Abort(e)) => {
                    let _ =
                        window_clone.emit("download_error", format!("Download aborted: {:?}", e));
                    break;
                }
                Err(_) => {
                    break;
                }
                _ => {}
            }
        }
    });

    let get_conn = || async move { Ok(connection) };
    let stats = iroh_blobs::get::db::get_to_db(&db, get_conn, &hash_and_format, progress)
        .await
        .map_err(|e| show_get_error(anyhow::anyhow!(e)))?;

    let collection = Collection::load_db(&db, &hash_and_format.hash).await?;

    if let Some((name, _)) = collection.iter().next() {
        if let Some(first) = name.split('/').next() {
            info!("Downloading to: {}", first);
        }
    }

    export(db, collection, file_storage_path.to_string()).await?;
    tokio::fs::remove_dir_all(iroh_data_dir).await?;

    info!(
        "Downloaded {} files, {}. Took {} ({}/s)",
        total_files,
        HumanBytes(payload_size),
        HumanDuration(stats.elapsed),
        HumanBytes((stats.bytes_read as f64 / stats.elapsed.as_secs_f64()) as u64),
    );

    Ok(())
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
