//! Neural voices that live on this machine.
//!
//! Windows' own voices are what the podcast reads with by default. They are installed per language
//! and many machines have only English, which is why an English voice reading a German script is
//! the failure this app reports rather than hides. A Kokoro pack fills that gap: one file set per
//! voice, unpacked under this app's own data folder and run through sherpa-onnx here.
//!
//! Fetching a pack is the second thing in this app that opens a network connection, after the
//! assistant. It happens only when the user asks for a pack by name, to an address from the
//! catalogue below — never to one the webview supplies.

use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::hub;

/// A Kokoro export on Hugging Face, and which of its files hold what.
#[derive(Debug, Clone)]
pub struct KokoroExport {
    pub repo: String,
    pub model: String,
    pub voices: String,
    /// A file in the repository holding the model's own vocabulary, where its table is not the
    /// mirror's.
    ///
    /// The two Kokoro generations differ by eleven entries — `A` sits at a different id and ten
    /// phoneme symbols are missing from the older table — which is enough to mispronounce a word
    /// and to drop the symbols silently. A model that states its own vocabulary is believed over
    /// the mirror.
    pub tokens: Option<String>,
}

/// Where a pack's bytes come from.
#[derive(Debug, Clone)]
pub enum Origin {
    /// One `.tar.bz2` from the sherpa-onnx release page, already laid out the way sherpa reads it.
    Archive { url: String },
    /// An export that has to be assembled before sherpa can speak it — see [`crate::hub`].
    KokoroExport(KokoroExport),
}

/// A voice pack that can be fetched, and where it comes from.
#[derive(Debug, Clone)]
pub struct Pack {
    pub id: String,
    /// The language the pack speaks, matched against the language the episode is scripted in.
    pub language: String,
    pub label: String,
    /// Roughly, so the size is on the button rather than a surprise after it is pressed.
    pub megabytes: u32,
    pub origin: Origin,
    /// The folder the pack lives in, which is not always the id.
    pub dir: String,
    /// What the pack's speakers are called, in the order their ids run.
    ///
    /// The names are not in the download: sherpa's own documentation is where the mapping is
    /// written down, so it is kept here rather than guessed from the file. A pack whose names are
    /// unknown leaves this empty and its speakers are numbered.
    pub speakers: Vec<String>,
}

/// The packs this app offers by name.
///
/// German twice and English once, because Windows ships an English voice on most machines and
/// nothing German on many. The German ones exist on Hugging Face only as community exports and are
/// assembled on the way in; the English one ships from the sherpa release page ready to use.
pub fn catalogue() -> Vec<Pack> {
    vec![
        Pack {
            id: "hub:Godelaune/Kokoro-82M-ONNX-German-Martin".to_owned(),
            language: "de".to_owned(),
            label: "Kokoro German (Martin)".to_owned(),
            megabytes: 330,
            origin: Origin::KokoroExport(KokoroExport {
                repo: "Godelaune/Kokoro-82M-ONNX-German-Martin".to_owned(),
                model: "kokoro-martin.onnx".to_owned(),
                voices: "voices-martin.npz".to_owned(),
                tokens: None,
            }),
            dir: "Godelaune--Kokoro-82M-ONNX-German-Martin".to_owned(),
            speakers: Vec::new(),
        },
        Pack {
            id: "hub:crane-local-ai/Kokoro-82M-v1.0-German-ONNX".to_owned(),
            language: "de".to_owned(),
            label: "Kokoro German (Kerstin)".to_owned(),
            megabytes: 330,
            origin: Origin::KokoroExport(KokoroExport {
                repo: "crane-local-ai/Kokoro-82M-v1.0-German-ONNX".to_owned(),
                model: "onnx/model.onnx".to_owned(),
                // Already flat, 510 rows of 256 float32 for its one speaker, so nothing is unpacked.
                voices: "voices/df_kerstin.bin".to_owned(),
                tokens: Some("tokenizer.json".to_owned()),
            }),
            dir: "crane-local-ai--Kokoro-82M-v1.0-German-ONNX".to_owned(),
            speakers: vec!["df_kerstin".to_owned()],
        },
        Pack {
            id: "kokoro-en-v0_19".to_owned(),
            language: "en".to_owned(),
            label: "Kokoro English".to_owned(),
            megabytes: 330,
            origin: Origin::Archive {
                url: "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-en-v0_19.tar.bz2"
                    .to_owned(),
            },
            dir: "kokoro-en-v0_19".to_owned(),
            // `a` is American and `b` British, `f` female and `m` male — the model's own naming,
            // in the order sherpa's speaker ids run.
            speakers: [
                "af",
                "af_bella",
                "af_nicole",
                "af_sarah",
                "af_sky",
                "am_adam",
                "am_michael",
                "bf_emma",
                "bf_isabella",
                "bm_george",
                "bm_lewis",
            ]
            .map(str::to_owned)
            .to_vec(),
        },
    ]
}

/// A pack as the settings page lists it: the catalogue entry plus what is on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    pub id: String,
    pub language: String,
    pub label: String,
    pub megabytes: u32,
    pub installed: bool,
    /// How many speakers the installed pack offers; zero until it is there.
    pub voices: u32,
    /// Their names where they are known, shortest list wins over the count above.
    pub speakers: Vec<String>,
}

fn find(id: &str) -> AppResult<Pack> {
    catalogue()
        .into_iter()
        .find(|pack| pack.id == id)
        .ok_or_else(|| AppError::Message(format!("unknown voice pack '{id}'")))
}

/// Where packs are unpacked: this app's own data folder, never beside the executable.
pub fn voices_dir(data_dir: &Path) -> AppResult<PathBuf> {
    let dir = data_dir.join("voices");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub(crate) fn pack_dir(data_dir: &Path, pack: &Pack) -> AppResult<PathBuf> {
    Ok(voices_dir(data_dir)?.join(&pack.dir))
}

/// A pack counts as installed once every file sherpa needs is there.
fn is_installed(dir: &Path) -> bool {
    dir.join("tokens.txt").is_file()
        && dir.join("espeak-ng-data").is_dir()
        && dir.join("model.onnx").is_file()
        && dir.join("voices.bin").is_file()
}

/// How many speakers a pack offers.
///
/// Kokoro says it in the size of its voice table: a flat array of float32 embeddings, 510 * 256 per
/// speaker. Nothing has to be loaded to read that, which is why the number costs a `stat` rather
/// than a model load on every listing.
pub(crate) const PER_SPEAKER: u64 = 510 * 256 * 4;

fn speaker_count(dir: &Path) -> u32 {
    fs::metadata(dir.join("voices.bin"))
        .map(|meta| (meta.len() / PER_SPEAKER) as u32)
        .unwrap_or(0)
}

/// The catalogue against what is on disk. Read from the folder rather than from a record of past
/// downloads, so a pack copied there by hand counts and a deleted one stops counting.
pub fn list(data_dir: &Path) -> AppResult<Vec<PackInfo>> {
    let root = voices_dir(data_dir)?;
    Ok(catalogue()
        .into_iter()
        .map(|pack| {
            let dir = root.join(&pack.dir);
            info(pack, &dir)
        })
        .collect())
}

fn info(pack: Pack, dir: &Path) -> PackInfo {
    let installed = is_installed(dir);
    PackInfo {
        id: pack.id,
        language: pack.language,
        label: pack.label,
        megabytes: pack.megabytes,
        installed,
        voices: if installed { speaker_count(dir) } else { 0 },
        speakers: pack.speakers,
    }
}

/// How much has arrived, for the settings page to show while it waits.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub id: String,
    pub received: u64,
    /// What the server promised, or none when it did not say.
    pub total: Option<u64>,
    /// The bytes are all here and are being written out. A 330 MB archive takes minutes to unpack,
    /// and a bar sitting at 100 per cent for that long reads as a download that died.
    pub unpacking: bool,
}

/// The packs whose download the user has given up on.
///
/// A list rather than one flag: two packs can be on their way at once, and a single flag would have
/// stopped both together.
static CANCELLED: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Asks one running download to stop at its next chunk.
pub fn cancel(id: &str) {
    if let Ok(mut cancelled) = CANCELLED.lock() {
        cancelled.push(id.to_owned());
    }
}

fn is_cancelled(id: &str) -> bool {
    CANCELLED
        .lock()
        .map(|cancelled| cancelled.iter().any(|entry| entry == id))
        .unwrap_or(false)
}

fn clear_cancel(id: &str) {
    if let Ok(mut cancelled) = CANCELLED.lock() {
        cancelled.retain(|entry| entry != id);
    }
}

/// Fetches a pack and unpacks it, reporting how far along it is through `report`.
pub fn install(data_dir: &Path, id: &str, report: &dyn Fn(Progress)) -> AppResult<PackInfo> {
    let pack = find(id)?;
    let dir = pack_dir(data_dir, &pack)?;
    if is_installed(&dir) {
        return Ok(info(pack, &dir));
    }

    clear_cancel(id);

    match &pack.origin {
        Origin::Archive { url } => {
            let bytes = download(url, id, 0, None, report)?;
            let size = bytes.len() as u64;
            report(Progress {
                id: id.to_owned(),
                received: size,
                total: Some(size),
                unpacking: true,
            });
            unpack(bytes, &voices_dir(data_dir)?)?;
        }
        Origin::KokoroExport(export) => hub::assemble_kokoro(data_dir, &pack, export, report)?,
    }

    if !is_installed(&dir) {
        // A half-written folder that looks installed is worse than no pack, so what arrived is
        // checked against what sherpa needs before the pack is called installed.
        let _ = fs::remove_dir_all(&dir);
        return Err(AppError::Message(
            "the download did not contain a usable voice".to_owned(),
        ));
    }

    Ok(info(pack, &dir))
}

/// Reads one URL into memory, saying how far it has come and stopping when the user has given up.
///
/// In memory rather than into the folder as it arrives: a half-written pack that looks installed
/// would be worse than holding a few hundred megabytes. `base` and `total` are what a caller
/// downloading several files in a row passes, so the page shows one bar rather than a row of them.
pub(crate) fn download(
    url: &str,
    id: &str,
    base: u64,
    total: Option<u64>,
    report: &dyn Fn(Progress),
) -> AppResult<Vec<u8>> {
    let mut response = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60 * 30))
        .build()
        .map_err(|error| AppError::Message(error.to_string()))?
        .get(url)
        .send()
        .map_err(|error| AppError::Message(error.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::Message(format!(
            "the download answered {}",
            response.status()
        )));
    }

    let total = total.or_else(|| response.content_length());
    let mut bytes: Vec<u8> = Vec::with_capacity(response.content_length().unwrap_or(0) as usize);
    let mut chunk = [0_u8; 64 * 1024];
    let mut announced = 0_u64;

    loop {
        if is_cancelled(id) {
            return Err(AppError::Message("cancelled".to_owned()));
        }
        let read = response.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);

        // Every megabyte rather than every chunk: a progress bar that repaints a thousand times a
        // second is only a way to spend the webview's time.
        let received = bytes.len() as u64;
        if received - announced >= 1024 * 1024 {
            announced = received;
            report(Progress {
                id: id.to_owned(),
                received: base + received,
                total,
                unpacking: false,
            });
        }
    }

    Ok(bytes)
}

/// Unpacks a `.tar.bz2` into a folder, entry by entry.
///
/// Not `Archive::unpack`: that canonicalises the destination, which on Windows yields a `\\?\` path,
/// and a verbatim path takes no forward slashes — so every archive whose entries carry them fails
/// with "trying to unpack outside of destination path". Joining the components here sidesteps it.
fn unpack(bytes: Vec<u8>, into: &Path) -> AppResult<()> {
    use std::path::Component;

    let decoder = bzip2::read::BzDecoder::new(Cursor::new(bytes));
    let mut archive = tar::Archive::new(decoder);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();

        // The address is this app's own, but an archive that writes outside the folder it was
        // unpacked into is never what was wanted.
        if path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::RootDir))
        {
            continue;
        }

        let target = into.join(&path);
        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&target)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        std::io::copy(&mut entry, &mut fs::File::create(&target)?)?;
    }
    Ok(())
}

pub fn remove(data_dir: &Path, id: &str) -> AppResult<()> {
    let dir = pack_dir(data_dir, &find(id)?)?;
    if dir.is_dir() {
        fs::remove_dir_all(dir)?;
    }
    Ok(())
}

/// The engine, kept alive between sentences.
///
/// Loading the model costs a second or two, and an episode is hundreds of segments; a reader who
/// presses preview twice should pay that once. The mutex is what makes one engine safe to share.
static ENGINE: Mutex<Option<(String, sherpa_onnx::OfflineTts)>> = Mutex::new(None);

fn load(data_dir: &Path, pack: &Pack) -> AppResult<()> {
    let mut slot = ENGINE
        .lock()
        .map_err(|_| AppError::Message("the speech engine is wedged".to_owned()))?;

    if slot.as_ref().is_some_and(|(id, _)| *id == pack.id) {
        return Ok(());
    }

    let dir = pack_dir(data_dir, pack)?;
    if !is_installed(&dir) {
        return Err(AppError::Message(format!("'{}' is not installed", pack.id)));
    }

    let tts = sherpa_onnx::OfflineTts::create(&engine_config(&dir, threads()))
        .ok_or_else(|| AppError::Message("the voice would not load".to_owned()))?;

    *slot = Some((pack.id.clone(), tts));
    Ok(())
}

/// How many threads the model runs on.
///
/// Measured on a 8-core / 16-thread machine, one sentence of 1.8 s took 2.7 s on one thread, 1.6 s
/// on two, 1.4 s on four and 0.94 s on eight — so the graph keeps parallelising well past the two
/// this started with. Capped at eight because the machine has other work to do and because the
/// curve is flattening there; taken from the machine's own count so four cores are not asked for
/// eight threads. Loading the model is unaffected either way: that cost is the file, not the
/// arithmetic.
fn threads() -> i32 {
    std::thread::available_parallelism()
        .map(|count| count.get().min(8) as i32)
        .unwrap_or(2)
}

fn engine_config(dir: &Path, threads: i32) -> sherpa_onnx::OfflineTtsConfig {
    sherpa_onnx::OfflineTtsConfig {
        model: sherpa_onnx::OfflineTtsModelConfig {
            num_threads: threads,
            kokoro: sherpa_onnx::OfflineTtsKokoroModelConfig {
                model: Some(dir.join("model.onnx").to_string_lossy().into_owned()),
                voices: Some(dir.join("voices.bin").to_string_lossy().into_owned()),
                tokens: Some(dir.join("tokens.txt").to_string_lossy().into_owned()),
                data_dir: Some(dir.join("espeak-ng-data").to_string_lossy().into_owned()),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

/// The samples a voice makes of a sentence, and the rate they are at.
///
/// Split from the playing because the podcast wants the samples and nothing else: it writes them
/// into a track rather than to a speaker.
pub fn synthesise(
    data_dir: &Path,
    id: &str,
    speaker: i32,
    text: &str,
) -> AppResult<(Vec<f32>, u32)> {
    let pack = find(id)?;
    load(data_dir, &pack)?;

    let mut slot = ENGINE
        .lock()
        .map_err(|_| AppError::Message("the speech engine is wedged".to_owned()))?;
    let Some((_, tts)) = slot.as_mut() else {
        return Err(AppError::Message("the speech engine vanished".to_owned()));
    };

    let audio = tts
        .generate_with_config(
            text,
            &sherpa_onnx::GenerationConfig {
                speed: 1.0,
                sid: speaker,
                ..Default::default()
            },
            None::<fn(&[f32], f32) -> bool>,
        )
        .ok_or_else(|| AppError::Message("the voice could not speak".to_owned()))?;

    Ok((audio.samples().to_vec(), audio.sample_rate() as u32))
}

/// Set while the preview should stop; cleared when the next one starts.
static SILENCED: AtomicBool = AtomicBool::new(false);

/// Cuts the sentence being previewed short.
pub fn silence() {
    SILENCED.store(true, Ordering::Relaxed);
}

/// Loads a pack's model without saying anything.
///
/// Reading 330 MB off the disk and handing it to the runtime costs about two seconds, and it is the
/// larger half of the wait before a preview starts. Paid when the voice is chosen instead, where
/// nobody is listening for silence.
pub fn warm(data_dir: &Path, id: &str) -> AppResult<()> {
    load(data_dir, &find(id)?)
}

/// Speaks one sentence out loud and returns when it has been said.
pub fn speak(data_dir: &Path, id: &str, speaker: i32, text: &str) -> AppResult<()> {
    if text.trim().is_empty() {
        return Ok(());
    }
    let (samples, rate) = synthesise(data_dir, id, speaker, text)?;
    play(&samples, rate)
}

/// Plays samples through whatever the system calls its speakers.
///
/// Waiting for the sink to empty, not for a clock: `sleep_until_end` cannot be interrupted, and the
/// stop button has to be able to end a sentence half way through. An earlier version also required
/// `get_pos() >= length`, which never became true — the position resets when the queue drains — so
/// every preview sat in the escape hatch below and took twice the audio's length.
/// Hands samples to the one thread allowed to touch the sound card, and waits for it to finish.
///
/// Everything audio happens there and nowhere else. Windows audio is COM, and opening a device
/// initialises a COM apartment **on the calling thread** — which, for a command running under
/// `spawn_blocking`, is a borrowed pool thread that afterwards goes back into the pool carrying that
/// apartment. WebView2 is COM as well, and enough threads left in that state stall its delivery:
/// the window stops taking clicks with nothing in the log, because nothing threw. One long-lived
/// thread owns the apartment for the life of the app, and no pool thread ever sees WASAPI.
pub(crate) fn play(samples: &[f32], sample_rate: u32) -> AppResult<()> {
    // Asking for a preview cancels the one before it. Without this, clicking twice queues two
    // sentences and the second is heard after the first has finished — which reads as an app that
    // ignored the click and then talked over itself a minute later.
    let ticket = REQUESTED.fetch_add(1, Ordering::SeqCst) + 1;
    silence();

    let (done, wait) = std::sync::mpsc::channel();
    player()
        .send((samples.to_vec(), sample_rate, ticket, done))
        .map_err(|_| AppError::Message("the audio thread is gone".to_owned()))?;
    // The command returns when the sentence has been said, which is what keeps the stop button on
    // screen for exactly as long as there is something to stop.
    wait.recv()
        .map_err(|_| AppError::Message("the audio thread stopped listening".to_owned()))?
}

/// How many previews have been asked for. A job whose number is no longer the latest is stale, and
/// stale jobs are dropped rather than played to an empty room.
static REQUESTED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

type Playback = (Vec<f32>, u32, u64, std::sync::mpsc::Sender<AppResult<()>>);

static PLAYER: std::sync::OnceLock<std::sync::mpsc::Sender<Playback>> = std::sync::OnceLock::new();

fn player() -> &'static std::sync::mpsc::Sender<Playback> {
    PLAYER.get_or_init(|| {
        let (sender, queue) = std::sync::mpsc::channel::<Playback>();
        std::thread::Builder::new()
            .name("voice-playback".to_owned())
            .spawn(move || {
                // Serial by construction: one sentence is played to its end before the next is
                // taken off the queue, so two previews can never fight over the device.
                while let Ok((samples, rate, ticket, done)) = queue.recv() {
                    let current = ticket == REQUESTED.load(Ordering::SeqCst);
                    let _ = done.send(if current {
                        play_here(&samples, rate)
                    } else {
                        Ok(())
                    });
                }
            })
            .expect("the audio thread must start");
        sender
    })
}

fn play_here(samples: &[f32], sample_rate: u32) -> AppResult<()> {
    SILENCED.store(false, Ordering::Relaxed);

    let stream = rodio::OutputStreamBuilder::open_default_stream()
        .map_err(|error| AppError::Message(format!("no audio output: {error}")))?;
    let sink = rodio::Sink::connect_new(stream.mixer());

    sink.append(rodio::buffer::SamplesBuffer::new(
        1,
        sample_rate,
        samples.to_vec(),
    ));

    let length =
        std::time::Duration::from_secs_f64(samples.len() as f64 / sample_rate.max(1) as f64);
    // What the device has yet to emit once the mixer has read everything. It cannot be asked for,
    // and dropping the stream at that moment cuts the end off the sentence — but it is a
    // shared-mode buffer, tens of milliseconds, so this is already generous.
    const TAIL: std::time::Duration = std::time::Duration::from_millis(600);
    const STEP: std::time::Duration = std::time::Duration::from_millis(50);
    let start = std::time::Instant::now();

    while !sink.empty() {
        if SILENCED.load(Ordering::Relaxed) {
            sink.stop();
            return Ok(());
        }
        // A device that never starts would otherwise hold this thread for as long as it stays
        // silent, and holding it helps nobody.
        if start.elapsed() > length * 2 + TAIL {
            break;
        }
        std::thread::sleep(STEP);
    }

    let quiet = std::time::Instant::now();
    while quiet.elapsed() < TAIL && !SILENCED.load(Ordering::Relaxed) {
        std::thread::sleep(STEP);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pack_is_reachable_by_id() {
        for pack in catalogue() {
            assert!(find(&pack.id).is_ok());
        }
        assert!(find("nope").is_err());
    }

    #[test]
    fn the_catalogue_covers_both_languages_the_app_speaks() {
        let languages: Vec<String> = catalogue().into_iter().map(|pack| pack.language).collect();
        assert!(languages.contains(&"en".to_owned()));
        assert!(languages.contains(&"de".to_owned()));
    }

    /// The English pack ships eleven speakers, which its voice table states in its own length:
    /// 5 766 400 bytes over 510 * 256 * 4 per speaker. A names list of another length would put
    /// the wrong name on a voice, which is worse than numbering them.
    #[test]
    fn the_english_pack_names_every_speaker_its_table_holds() {
        let pack = find("kokoro-en-v0_19").expect("the english pack");

        assert_eq!(pack.speakers.len(), 11);
        assert_eq!(5_766_400 / PER_SPEAKER as usize, pack.speakers.len());
    }

    #[test]
    fn a_folder_without_the_files_is_not_installed() {
        let dir = std::env::temp_dir().join("openexamtrainer-no-such-voice");
        assert!(!is_installed(&dir));
        assert_eq!(speaker_count(&dir), 0);
    }

    #[test]
    fn a_pack_is_listed_as_missing_before_it_is_fetched() {
        let data_dir = std::env::temp_dir().join("openexamtrainer-voice-listing");
        let _ = fs::remove_dir_all(&data_dir);

        let packs = list(&data_dir).expect("listing");

        assert_eq!(packs.len(), catalogue().len());
        assert!(packs.iter().all(|pack| !pack.installed && pack.voices == 0));

        let _ = fs::remove_dir_all(&data_dir);
    }

    /// An archive that climbs out of the folder writes over whatever it names; the entry is skipped
    /// rather than the archive refused, because the rest of it is still the pack.
    #[test]
    fn an_archive_entry_that_climbs_out_is_skipped() {
        let mut builder = tar::Builder::new(Vec::new());
        let mut climbing = tar::Header::new_gnu();
        climbing.set_size(2);
        // Written into the header's own bytes: `append_data` refuses a `..` path outright, and this
        // archive is the one thing the check below exists for.
        climbing.as_old_mut().name[..14].copy_from_slice(b"../escaped.txt");
        climbing.set_cksum();
        builder.append(&climbing, &b"no"[..]).expect("entry");

        let mut ordinary = tar::Header::new_gnu();
        ordinary.set_size(2);
        ordinary.set_cksum();
        builder
            .append_data(&mut ordinary, "tokens.txt", &b"ok"[..])
            .expect("entry");
        let tar = builder.into_inner().expect("tar");

        let mut compressed = Vec::new();
        let mut encoder = bzip2::read::BzEncoder::new(Cursor::new(tar), bzip2::Compression::fast());
        encoder.read_to_end(&mut compressed).expect("bz2");

        let into = std::env::temp_dir().join("openexamtrainer-unpack-test");
        let _ = fs::remove_dir_all(&into);
        fs::create_dir_all(&into).expect("dir");

        unpack(compressed, &into).expect("unpack");

        assert!(into.join("tokens.txt").is_file());
        assert!(!into.parent().expect("parent").join("escaped.txt").exists());

        let _ = fs::remove_dir_all(&into);
    }

    #[test]
    fn a_cancelled_download_is_forgotten_before_the_next_attempt() {
        cancel("kokoro-en-v0_19");
        assert!(is_cancelled("kokoro-en-v0_19"));
        clear_cancel("kokoro-en-v0_19");
        assert!(!is_cancelled("kokoro-en-v0_19"));
    }
}

#[cfg(test)]
mod probe {
    use super::*;

    /// Speaks a sentence with whichever pack is named, and says what came out.
    ///
    /// The point is the samples, not the sound: a model whose vocabulary does not match its tokens
    /// still returns a buffer, but a silent or absurdly short one. Length and peak are what can be
    /// checked without ears.
    ///
    /// `$env:PACK='...'; cargo test --manifest-path src-tauri/Cargo.toml -- --ignored --nocapture a_pack_says_something`
    #[test]
    #[ignore = "needs an installed pack"]
    fn a_pack_says_something() {
        let data_dir =
            std::env::var("VOICE_DATA_DIR").expect("set VOICE_DATA_DIR to the app data folder");
        let id = std::env::var("PACK").expect("set PACK to a pack id");
        let text = std::env::var("SAY")
            .unwrap_or_else(|_| "Guten Tag, so werden die Fragen klingen.".to_owned());

        let (samples, rate) =
            synthesise(Path::new(&data_dir), &id, 0, &text).expect("the voice must speak");
        let seconds = samples.len() as f32 / rate as f32;
        let peak = samples.iter().fold(0.0_f32, |a, s| a.max(s.abs()));
        println!(
            "pack={id} rate={rate} samples={} seconds={seconds:.2} peak={peak:.3}",
            samples.len()
        );

        // Numbers say a voice is not silent; only a file says whether it is worth listening to.
        if let Ok(path) = std::env::var("WAV") {
            let mut out = Vec::with_capacity(44 + samples.len() * 2);
            let pcm: Vec<u8> = samples
                .iter()
                .flat_map(|s| ((s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16).to_le_bytes())
                .collect();
            out.extend_from_slice(b"RIFF");
            out.extend_from_slice(&((36 + pcm.len()) as u32).to_le_bytes());
            out.extend_from_slice(b"WAVEfmt ");
            out.extend_from_slice(&16_u32.to_le_bytes());
            out.extend_from_slice(&1_u16.to_le_bytes());
            out.extend_from_slice(&1_u16.to_le_bytes());
            out.extend_from_slice(&rate.to_le_bytes());
            out.extend_from_slice(&(rate * 2).to_le_bytes());
            out.extend_from_slice(&2_u16.to_le_bytes());
            out.extend_from_slice(&16_u16.to_le_bytes());
            out.extend_from_slice(b"data");
            out.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
            out.extend_from_slice(&pcm);
            fs::write(&path, out).expect("the sample must be writable");
            println!("wrote {path}");
        }

        assert!(
            seconds > 1.0,
            "a sentence that short cannot be the sentence"
        );
        assert!(peak > 0.05, "the buffer is silence");
    }

    /// Where the wait before a preview goes, in numbers rather than impressions.
    ///
    /// A diagnostic, not a check: it prints what loading the model and saying one sentence cost at
    /// each thread count on the machine it ran on, and nothing here is a threshold anyone agreed on.
    ///
    /// `$env:PACK='...'; cargo test --manifest-path src-tauri/Cargo.toml -- --ignored --nocapture where_the_wait`
    #[test]
    #[ignore = "diagnostic, not a check"]
    fn where_the_wait_before_a_preview_goes() {
        let data_dir = PathBuf::from(
            std::env::var("VOICE_DATA_DIR").expect("set VOICE_DATA_DIR to the app data folder"),
        );
        let pack = find(&std::env::var("PACK").expect("set PACK to a pack id")).expect("pack");
        let dir = pack_dir(&data_dir, &pack).expect("folder");
        let text = "So werden deine Fragen klingen.";

        println!("--- {} ---", pack.id);
        for threads in [1, 2, 4, 8] {
            let start = std::time::Instant::now();
            let tts = sherpa_onnx::OfflineTts::create(&engine_config(&dir, threads)).expect("load");
            let loaded = start.elapsed();

            let mut spoken = Vec::new();
            for run in 0..2 {
                let start = std::time::Instant::now();
                let audio = tts
                    .generate_with_config(
                        text,
                        &sherpa_onnx::GenerationConfig {
                            speed: 1.0,
                            sid: 0,
                            ..Default::default()
                        },
                        None::<fn(&[f32], f32) -> bool>,
                    )
                    .expect("speak");
                let seconds = audio.samples().len() as f32 / audio.sample_rate() as f32;
                spoken.push((run, start.elapsed(), seconds));
            }

            for (run, took, seconds) in spoken {
                println!(
                    "  {threads} threads: load {:>6.2}s, run {run} {:>6.2}s for {seconds:.2}s of audio ({:.2}x real time)",
                    loaded.as_secs_f32(),
                    took.as_secs_f32(),
                    seconds / took.as_secs_f32()
                );
            }
        }
    }

    /// The whole preview, out loud and timed: model, sentence, speaker.
    ///
    /// A diagnostic, not a check — and one that makes a noise, which is the point: the last figure
    /// is how long the button keeps saying "stop" after the room has gone quiet.
    #[test]
    #[ignore = "diagnostic, and it plays out loud"]
    fn what_a_preview_costs_end_to_end() {
        let data_dir = PathBuf::from(
            std::env::var("VOICE_DATA_DIR").expect("set VOICE_DATA_DIR to the app data folder"),
        );
        let id = std::env::var("PACK").expect("set PACK to a pack id");
        let text = "So werden deine Fragen klingen.";

        let cold = std::time::Instant::now();
        let (samples, rate) = synthesise(&data_dir, &id, 0, text).expect("speak");
        let synthesised = cold.elapsed();
        let seconds = samples.len() as f32 / rate as f32;

        let playing = std::time::Instant::now();
        play(&samples, rate).expect("play");
        let played = playing.elapsed();

        let warm = std::time::Instant::now();
        let _ = synthesise(&data_dir, &id, 0, text).expect("speak again");

        println!("--- {id} ---");
        println!(
            "  first sentence   {:>6.2}s (model load included)",
            synthesised.as_secs_f32()
        );
        println!(
            "  second sentence  {:>6.2}s (engine already loaded)",
            warm.elapsed().as_secs_f32()
        );
        println!("  audio            {seconds:>6.2}s");
        println!(
            "  playback         {:>6.2}s, {:.2}s of it after the audio ended",
            played.as_secs_f32(),
            played.as_secs_f32() - seconds
        );
    }

    /// Fetches a pack into the folder `VOICE_DATA_DIR` names — the app's own, so a pack fetched
    /// here is a pack the app then has, rather than a few hundred megabytes downloaded twice.
    #[test]
    #[ignore = "downloads a few hundred megabytes"]
    fn a_pack_installs() {
        let data_dir = std::env::var("VOICE_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("openexamtrainer-voice-install"));
        let id = std::env::var("PACK").unwrap_or_else(|_| "kokoro-en-v0_19".to_owned());

        let installed = install(&data_dir, &id, &|progress| {
            if let Some(total) = progress.total {
                eprintln!("  {} / {total} bytes", progress.received);
            }
        })
        .expect("install");

        assert!(installed.installed);
        assert!(
            installed.voices > 0,
            "a pack with no speaker cannot be chosen"
        );
    }
}
