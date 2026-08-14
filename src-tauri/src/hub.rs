//! Building the one voice that has to be assembled before sherpa can speak it.
//!
//! The German Kokoro exists on Hugging Face only as a community export: a bare `.onnx` and a `.npz`
//! of voice embeddings, the layout the Python runtime wants. sherpa needs `tokens.txt`, an
//! `espeak-ng-data` folder and the voice table as a flat `.bin` besides, so the missing parts are
//! fetched from the sherpa mirror and the model is described here.
//!
//! Two public, read-only endpoints are used: a repository's file tree and its files. No token is
//! sent, so nothing private is ever visible here.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::voice::{KokoroExport, Pack, Progress, PER_SPEAKER};

const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// The sherpa mirror the token table and the phonemiser's data come from.
///
/// A community Kokoro export carries neither, and both are the same for every Kokoro voice: the
/// token table is the model family's own, and espeak-ng's data covers every language at once.
const MIRROR: &str = "csukuangfj/kokoro-en-v0_19";

#[derive(Debug, Deserialize)]
struct TreeEntry {
    #[serde(rename = "type")]
    kind: String,
    path: String,
    #[serde(default)]
    size: u64,
}

fn get_json<T: serde::de::DeserializeOwned>(url: &str) -> AppResult<T> {
    let response = reqwest::blocking::Client::builder()
        .timeout(TIMEOUT)
        // The hub answers anonymous callers, but asks to be told who is calling.
        .user_agent("OpenExamTrainer")
        .build()
        .map_err(|error| AppError::Message(error.to_string()))?
        .get(url)
        .send()
        .map_err(|error| AppError::Message(error.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::Message(format!(
            "the hub answered {}",
            response.status()
        )));
    }

    response
        .json::<T>()
        .map_err(|error| AppError::Message(error.to_string()))
}

/// The files of one repository, flattened, with their sizes.
fn tree(repo: &str) -> AppResult<Vec<TreeEntry>> {
    let entries: Vec<TreeEntry> = get_json(&format!(
        "https://huggingface.co/api/models/{repo}/tree/main?recursive=true"
    ))?;
    Ok(entries
        .into_iter()
        .filter(|entry| entry.kind == "file")
        .collect())
}

fn file_url(repo: &str, path: &str) -> String {
    format!("https://huggingface.co/{repo}/resolve/main/{path}")
}

/// Assembles a Kokoro export into something sherpa can speak.
///
/// Four things have to be true and only one of them is in the repository: the model, a flat table
/// of voices, the token table, and the metadata sherpa reads out of the model. The export has the
/// first as a file and the second as a NumPy archive; the third comes from the mirror, and the
/// fourth is written here.
pub fn assemble_kokoro(
    data_dir: &Path,
    pack: &Pack,
    export: &KokoroExport,
    report: &dyn Fn(Progress),
) -> AppResult<()> {
    let KokoroExport {
        repo,
        model,
        voices,
        tokens: own_tokens,
    } = export;
    let dir = crate::voice::pack_dir(data_dir, pack)?;
    fs::create_dir_all(&dir)?;

    let files = tree(repo)?;
    let size_of = |name: &str| {
        files
            .iter()
            .find(|entry| entry.path == name)
            .map(|entry| entry.size)
            .unwrap_or(0)
    };
    let espeak: Vec<TreeEntry> = tree(MIRROR)?
        .into_iter()
        .filter(|entry| entry.path.starts_with("espeak-ng-data/"))
        .collect();
    let total = size_of(model) + size_of(voices) + espeak.iter().map(|e| e.size).sum::<u64>();

    let raw = crate::voice::download(&file_url(repo, model), &pack.id, 0, Some(total), report)?;
    let mut done = raw.len() as u64;

    let table =
        crate::voice::download(&file_url(repo, voices), &pack.id, done, Some(total), report)?;
    done += table.len() as u64;
    // Some exports ship the table already flat, which is what sherpa wants; the shape is then the
    // file's own length and nothing has to be unpacked.
    let (flat, speakers, style_dim) = if voices.ends_with(".npz") {
        flatten_voices(&table)?
    } else {
        shape_of(table)
    };
    fs::write(dir.join("voices.bin"), flat)?;

    // The model's own vocabulary where it states one, the mirror's where it does not.
    let tokens = match own_tokens {
        Some(path) => {
            let raw =
                crate::voice::download(&file_url(repo, path), &pack.id, done, Some(total), report)?;
            tokens_from_vocabulary(&raw)?.into_bytes()
        }
        None => crate::voice::download(
            &file_url(MIRROR, "tokens.txt"),
            &pack.id,
            done,
            Some(total),
            report,
        )?,
    };
    fs::write(dir.join("tokens.txt"), tokens)?;

    for entry in &espeak {
        let bytes = crate::voice::download(
            &file_url(MIRROR, &entry.path),
            &pack.id,
            done,
            Some(total),
            report,
        )?;
        write_under(&dir, &entry.path, &bytes)?;
        done += bytes.len() as u64;
    }

    report(Progress {
        id: pack.id.clone(),
        received: total,
        total: Some(total),
        unpacking: true,
    });
    fs::write(
        dir.join("model.onnx"),
        with_metadata(raw, pack, speakers, style_dim),
    )?;
    Ok(())
}

/// Rewrites a Hugging Face tokenizer's vocabulary as the table sherpa reads.
///
/// One `token id` pair per line, ordered by id. The space token is a line that begins with a space
/// and sherpa reads it that way round, so the pairs are written by hand rather than through
/// anything that would trim them.
fn tokens_from_vocabulary(raw: &[u8]) -> AppResult<String> {
    let odd = |what: &str| AppError::Message(format!("the token table is {what}"));

    let parsed: serde_json::Value =
        serde_json::from_slice(raw).map_err(|error| odd(&error.to_string()))?;
    let vocabulary = parsed
        .get("model")
        .and_then(|model| model.get("vocab"))
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| odd("not a tokenizer with a vocabulary"))?;

    let mut pairs: Vec<(&String, i64)> = vocabulary
        .iter()
        .map(|(token, id)| {
            id.as_i64()
                .map(|id| (token, id))
                .ok_or_else(|| odd("holding an id that is not a number"))
        })
        .collect::<AppResult<_>>()?;
    if pairs.is_empty() {
        return Err(odd("empty"));
    }
    pairs.sort_by_key(|(_, id)| *id);

    Ok(pairs
        .into_iter()
        .map(|(token, id)| format!("{token} {id}\n"))
        .collect())
}

/// The speaker count and shape of a table that is already flat.
fn shape_of(table: Vec<u8>) -> (Vec<u8>, u32, String) {
    let speakers = (table.len() as u64 / PER_SPEAKER).max(1) as u32;
    let rows = table.len() / (256 * 4 * speakers as usize);
    (table, speakers, format!("{rows},1,256"))
}

/// Reads the one array out of a `.npz` and returns it as sherpa's flat table.
///
/// The archive is a zip and NumPy writes its arrays uncompressed, so the entry is taken from the
/// local header rather than by pulling a zip reader in for one file. Returned with the speaker
/// count and the shape, both of which the model's metadata has to state and only this file knows.
fn flatten_voices(npz: &[u8]) -> AppResult<(Vec<u8>, u32, String)> {
    let odd = |what: &str| AppError::Message(format!("the voice table is {what}"));

    if npz.get(..4) != Some(b"PK\x03\x04") {
        return Err(odd("not a NumPy archive"));
    }
    let word = |at: usize| -> AppResult<usize> {
        let bytes = npz.get(at..at + 2).ok_or_else(|| odd("truncated"))?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]) as usize)
    };
    if word(8)? != 0 {
        return Err(odd("compressed, which NumPy does not do by default"));
    }
    let stored = npz
        .get(18..22)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)
        .ok_or_else(|| odd("truncated"))?;
    let at = 30 + word(26)? + word(28)?;
    // To the entry's own length, not to the end of the file: the zip's central directory sits
    // behind it and would ride along as seventy-eight stray bytes.
    let npy = npz.get(at..at + stored).ok_or_else(|| odd("truncated"))?;

    if npy.get(..6) != Some(b"\x93NUMPY") {
        return Err(odd("not a NumPy array"));
    }
    let header_len = u16::from_le_bytes([npy[8], npy[9]]) as usize;
    let header = String::from_utf8_lossy(&npy[10..10 + header_len]).to_string();
    let body = npy
        .get(10 + header_len..)
        .ok_or_else(|| odd("truncated"))?
        .to_vec();

    if !header.contains("'<f4'") {
        return Err(odd("not a table of 32-bit floats"));
    }

    let speakers = (body.len() as u64 / PER_SPEAKER).max(1) as u32;
    let rows = body.len() / (256 * 4 * speakers as usize);
    Ok((body, speakers, format!("{rows},1,256")))
}

/// Appends the metadata sherpa's Kokoro loader insists on.
///
/// Protobuf takes repeated fields wherever they appear, so the entries are written onto the end of
/// the model rather than the file being rebuilt. The exports leave these out because the Python
/// runtime never reads them.
fn with_metadata(mut model: Vec<u8>, pack: &Pack, speakers: u32, style_dim: String) -> Vec<u8> {
    let entry = |key: &str, value: &str| {
        let mut payload = Vec::new();
        payload.push(0x0a);
        payload.push(key.len() as u8);
        payload.extend_from_slice(key.as_bytes());
        payload.push(0x12);
        payload.push(value.len() as u8);
        payload.extend_from_slice(value.as_bytes());
        let mut field = vec![0x72, payload.len() as u8];
        field.extend_from_slice(&payload);
        field
    };

    // `voice` is the espeak voice the phonemiser is asked for, so it is the language tag rather
    // than the pack's name.
    for (key, value) in [
        ("model_type", "kokoro".to_owned()),
        ("language", pack.label.clone()),
        ("has_espeak", "1".to_owned()),
        ("sample_rate", "24000".to_owned()),
        ("version", "1".to_owned()),
        ("voice", pack.language.clone()),
        ("style_dim", style_dim),
        ("n_speakers", speakers.to_string()),
    ] {
        model.extend_from_slice(&entry(key, &value));
    }
    model
}

/// Writes one file of the repository under the pack's folder.
///
/// The path comes from the hub, so it is taken apart and rebuilt from its plain components: a `..`
/// in it would otherwise write outside the folder.
fn write_under(dir: &Path, path: &str, bytes: &[u8]) -> AppResult<()> {
    let mut target = dir.to_path_buf();
    for part in path.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return Err(AppError::Message(format!(
                "the hub sent an odd path: {path}"
            )));
        }
        target.push(part);
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(target, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voice::Origin;

    fn pack() -> Pack {
        Pack {
            id: "hub:x/y".to_owned(),
            language: "de".to_owned(),
            label: "Kokoro German".to_owned(),
            megabytes: 1,
            origin: Origin::KokoroExport(KokoroExport {
                repo: "x/y".to_owned(),
                model: "model.onnx".to_owned(),
                voices: "voices.npz".to_owned(),
                tokens: None,
            }),
            dir: "x--y".to_owned(),
            speakers: Vec::new(),
        }
    }

    /// The space is a token like any other and its line begins with one; a writer that trimmed it
    /// would leave sherpa without the character that separates every word.
    #[test]
    fn a_vocabulary_becomes_sherpas_own_table() {
        let raw = br#"{"model":{"vocab":{"b":2,"$":0," ":16,"a":1}}}"#;

        let table = tokens_from_vocabulary(raw).expect("table");

        assert_eq!(table, "$ 0\na 1\nb 2\n  16\n");
    }

    #[test]
    fn a_file_that_is_not_a_tokenizer_is_refused_by_name() {
        let error = tokens_from_vocabulary(b"{\"model\":{}}")
            .expect_err("no vocabulary")
            .to_string();

        assert!(error.contains("vocabulary"), "{error}");
    }

    #[test]
    fn a_flat_table_is_measured_rather_than_unpacked() {
        let (table, speakers, shape) = shape_of(vec![0_u8; 510 * 256 * 4 * 2]);

        assert_eq!(speakers, 2);
        assert_eq!(shape, "510,1,256");
        assert_eq!(table.len(), 510 * 256 * 4 * 2, "the bytes are handed on");
    }

    #[test]
    fn a_numpy_archive_becomes_a_flat_table() {
        // One stored entry holding a (2, 1, 4) float32 array: the same shape a Kokoro style table
        // has, only small enough to write out here.
        let rows = 2_usize;
        let body: Vec<u8> = (0..rows * 4)
            .flat_map(|i| (i as f32).to_le_bytes())
            .collect();
        let header =
            format!("{{'descr': '<f4', 'fortran_order': False, 'shape': ({rows}, 1, 4), }}");
        let mut npy = b"\x93NUMPY\x01\x00".to_vec();
        npy.extend_from_slice(&(header.len() as u16).to_le_bytes());
        npy.extend_from_slice(header.as_bytes());
        npy.extend_from_slice(&body);

        let mut npz = b"PK\x03\x04".to_vec();
        npz.resize(18, 0);
        npz.extend_from_slice(&(npy.len() as u32).to_le_bytes());
        npz.resize(26, 0);
        npz.extend_from_slice(&4_u16.to_le_bytes());
        npz.extend_from_slice(&0_u16.to_le_bytes());
        npz.extend_from_slice(b"a.np");
        npz.extend_from_slice(&npy);
        // What a real archive keeps behind the entry, and what must not be read.
        npz.extend_from_slice(b"PK\x01\x02trailing rubbish");

        let (flat, speakers, _) = flatten_voices(&npz).expect("flatten");

        assert_eq!(
            flat, body,
            "the payload is the table, and nothing behind it"
        );
        assert_eq!(speakers, 1);
    }

    #[test]
    fn a_compressed_archive_is_refused_by_name() {
        let mut npz = b"PK\x03\x04".to_vec();
        npz.resize(40, 0);
        npz[8] = 8; // deflate

        let error = flatten_voices(&npz).expect_err("compressed").to_string();

        assert!(error.contains("compressed"), "{error}");
    }

    #[test]
    fn the_metadata_carries_the_packs_own_language() {
        let out = with_metadata(vec![1, 2, 3], &pack(), 1, "510,1,256".to_owned());

        let tail = String::from_utf8_lossy(&out[3..]).to_string();
        assert!(tail.contains("voice"), "{tail}");
        assert!(tail.contains("de"), "{tail}");
        assert!(tail.contains("510,1,256"), "{tail}");
        assert_eq!(&out[..3], &[1, 2, 3], "the model itself is left alone");
    }

    #[test]
    fn a_climbing_path_is_refused() {
        let dir = std::env::temp_dir().join("openexamtrainer-hub-test");

        assert!(write_under(&dir, "../escaped.txt", b"no").is_err());
    }
}
