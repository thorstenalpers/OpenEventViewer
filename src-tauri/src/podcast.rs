use std::path::{Path, PathBuf};

use rusty_mp3::{Mp3Encoder, Mp3EncoderConfig};
use serde::{Deserialize, Serialize};

use crate::dto::QuestionDto;
use crate::error::{AppError, AppResult};

/// What Windows speech synthesis is asked for. A downloaded voice brings its own rate instead, so
/// everything downstream of the synthesiser takes the rate as an argument rather than assuming one.
const WINDOWS_RATE: u32 = 22_050;
const CHANNELS: u16 = 1;
const BITS: u16 = 16;
const BYTES_PER_SAMPLE: u32 = (BITS as u32 / 8) * CHANNELS as u32;
/// 22 050 Hz puts the stream on MPEG-2, where 64 kbps is the format's own default and already
/// generous for one mono voice.
const MP3_KBPS: u32 = 64;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    /// A fifth of the size, which is the difference between an episode you carry on a phone and one
    /// you leave on the machine that made it.
    #[default]
    Mp3,
    /// What the synthesiser produced, written through unchanged. Nothing re-encodes it, so nothing
    /// can degrade it — at five times the bytes.
    Wav,
}

impl Format {
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
        }
    }
}

/// Which voice reads the episode, and which language the words around the question are in.
///
/// Not the question itself: that is the user's imported material and is never translated, here or
/// anywhere else. Choosing German for an English question bank therefore gets a German voice
/// reading English sentences — allowed, because the alternative is guessing, but rarely wanted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    /// These dumps are written in English, so the scaffolding around them starts there too.
    #[default]
    En,
    De,
}

impl Language {
    pub const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::De => "de",
        }
    }

    fn question(self, number: u32) -> String {
        match self {
            Self::En => format!("Question {number}."),
            Self::De => format!("Frage {number}."),
        }
    }

    fn option(self, letter: char, text: &str) -> String {
        match self {
            Self::En => format!(" Option {letter}: {text}."),
            Self::De => format!(" Antwort {letter}: {text}."),
        }
    }

    fn answer(self, letters: &[char]) -> String {
        let joined = letters
            .iter()
            .map(char::to_string)
            .collect::<Vec<_>>()
            .join(match self {
                Self::En => " and ",
                Self::De => " und ",
            });

        match (self, letters.len()) {
            (Self::En, 0) => "The source gives no answer for this question.".to_string(),
            (Self::De, 0) => "Die Quelle nennt für diese Frage keine Antwort.".to_string(),
            (Self::En, 1) => format!("The answer is {joined}."),
            (Self::De, 1) => format!("Die richtige Antwort ist {joined}."),
            (Self::En, _) => format!("The answers are {joined}."),
            (Self::De, _) => format!("Die richtigen Antworten sind {joined}."),
        }
    }
}

/// A downloaded voice pack and which of its speakers reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Voice {
    pub pack_id: String,
    pub speaker: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    /// Off turns the episode into a pure recall drill: question, silence, next question.
    pub include_answer: bool,
    pub include_explanation: bool,
    pub pause_seconds: f32,
    pub format: Format,
    pub language: Language,
    /// None reads with the Windows voice for the language, which is what a machine has before it
    /// has downloaded anything.
    #[serde(default)]
    pub voice: Option<Voice>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            include_answer: true,
            include_explanation: true,
            pause_seconds: 4.0,
            format: Format::default(),
            language: Language::default(),
            voice: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub question_number: u32,
    pub offset_ms: u64,
    pub title: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    pub path: String,
    pub duration_ms: u64,
    pub chapters: Vec<Chapter>,
}

/// One spoken block. The pause after a question is silence rather than a spoken instruction, so the
/// listener can answer out loud without the recording talking over them.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub question_number: u32,
    pub starts_chapter: bool,
    pub text: String,
    pub pause_after_seconds: f32,
}

pub fn build_script(questions: &[QuestionDto], options: &Options) -> Vec<Segment> {
    let mut segments = Vec::new();

    for question in questions {
        let mut spoken = format!(
            "{} {}",
            options.language.question(question.number),
            // Without a terminal stop the synthesiser runs the stem straight into "Option A".
            sentence(&speakable(&question.stem))
        );
        for option in &question.options {
            spoken.push_str(
                &options
                    .language
                    .option(option.letter, &speakable(&option.text)),
            );
        }
        segments.push(Segment {
            question_number: question.number,
            starts_chapter: true,
            text: spoken,
            pause_after_seconds: options.pause_seconds,
        });

        if options.include_answer {
            segments.push(Segment {
                question_number: question.number,
                starts_chapter: false,
                text: options.language.answer(&question.answer_letters),
                pause_after_seconds: 0.8,
            });
        }

        if options.include_explanation && !question.explanation.trim().is_empty() {
            segments.push(Segment {
                question_number: question.number,
                starts_chapter: false,
                text: speakable(&question.explanation),
                pause_after_seconds: 1.5,
            });
        }
    }

    segments
}

fn sentence(text: &str) -> String {
    let trimmed = text.trim_end();
    if trimmed.is_empty() || trimmed.ends_with(['.', '?', '!', ':']) {
        trimmed.to_string()
    } else {
        format!("{trimmed}.")
    }
}

/// Speech synthesis reads a URL character by character and a bullet as "asterisk". Both are noise
/// in an audio track, so they never reach the synthesiser.
fn speakable(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for word in text.split_whitespace() {
        if word.starts_with("http://") || word.starts_with("https://") {
            continue;
        }
        let cleaned: String = word
            .chars()
            .filter(|c| !matches!(c, '*' | '#' | '`' | '|' | '<' | '>'))
            .collect();
        if cleaned.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&cleaned);
    }
    out
}

pub fn generate(
    questions: &[QuestionDto],
    options: &Options,
    work_dir: &Path,
    destination: &Path,
) -> AppResult<Episode> {
    let segments = build_script(questions, options);
    if segments.is_empty() {
        return Err(AppError::Message("nothing selected to record".into()));
    }

    let (spoken, rate) = match &options.voice {
        Some(voice) => through_pack(&segments, voice, work_dir)?,
        None => through_windows(&segments, options.language, work_dir)?,
    };

    let mut audio = Vec::new();
    let mut chapters = Vec::new();
    for (segment, pcm) in segments.iter().zip(&spoken) {
        if segment.starts_chapter {
            chapters.push(Chapter {
                question_number: segment.question_number,
                offset_ms: bytes_to_ms(audio.len(), rate),
                title: format!("Question {}", segment.question_number),
            });
        }
        audio.extend_from_slice(pcm);
        audio.extend(silence(segment.pause_after_seconds, rate));
    }

    let encoded = match options.format {
        Format::Mp3 => mp3(&audio, rate)?,
        Format::Wav => wav(&audio, rate),
    };
    std::fs::write(destination, encoded).map_err(|error| AppError::Message(error.to_string()))?;

    Ok(Episode {
        path: destination.to_string_lossy().to_string(),
        duration_ms: bytes_to_ms(audio.len(), rate),
        chapters,
    })
}

/// Every segment as PCM, read by a downloaded voice.
///
/// Nothing is staged on disk: the model hands back samples, and writing them out as WAV files only
/// to read them straight back is work the Windows path is forced into and this one is not.
fn through_pack(
    segments: &[Segment],
    voice: &Voice,
    data_dir: &Path,
) -> AppResult<(Vec<Vec<u8>>, u32)> {
    let mut spoken = Vec::with_capacity(segments.len());
    let mut rate = 0;

    for segment in segments {
        let (samples, sample_rate) =
            crate::voice::synthesise(data_dir, &voice.pack_id, voice.speaker, &segment.text)?;
        rate = sample_rate;
        spoken.push(
            samples
                .iter()
                .flat_map(|sample| {
                    ((sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16).to_le_bytes()
                })
                .collect(),
        );
    }

    if rate == 0 {
        return Err(AppError::Message("the voice returned nothing".into()));
    }
    Ok((spoken, rate))
}

/// Every segment as PCM, read by a Windows voice through one PowerShell run.
fn through_windows(
    segments: &[Segment],
    language: Language,
    work_dir: &Path,
) -> AppResult<(Vec<Vec<u8>>, u32)> {
    let staging = work_dir.join("podcast");
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|error| AppError::Message(error.to_string()))?;

    let manifest: Vec<serde_json::Value> = segments
        .iter()
        .enumerate()
        .map(|(index, segment)| {
            serde_json::json!({
                "file": staging.join(format!("{index:04}.wav")).to_string_lossy(),
                "text": segment.text,
            })
        })
        .collect();
    let manifest_path = staging.join("segments.json");
    std::fs::write(&manifest_path, serde_json::to_string(&manifest)?)
        .map_err(|error| AppError::Message(error.to_string()))?;

    synthesize(&manifest_path, language)?;

    let mut spoken = Vec::with_capacity(segments.len());
    for index in 0..segments.len() {
        let bytes = std::fs::read(staging.join(format!("{index:04}.wav"))).map_err(|error| {
            AppError::Message(format!("segment {index} was not synthesised: {error}"))
        })?;
        spoken.push(pcm_of(&bytes, WINDOWS_RATE)?);
    }

    let _ = std::fs::remove_dir_all(&staging);
    Ok((spoken, WINDOWS_RATE))
}

/// Windows speech synthesis, one process for the whole episode. Offline, no key, nothing leaves the
/// machine — which is why it is the default voice rather than a hosted one.
///
/// The voice is chosen by language rather than left to the system default. A machine set to German
/// would otherwise read an English question bank in a German voice, which is not an accent but a
/// different set of letter sounds — and it would do it without reporting anything. Missing the
/// requested language is an error naming what *is* installed, because "install a voice" is
/// something the user can act on and silence is not.
fn synthesize(manifest: &Path, language: Language) -> AppResult<()> {
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Speech
$format = New-Object System.Speech.AudioFormat.SpeechAudioFormatInfo({WINDOWS_RATE}, [System.Speech.AudioFormat.AudioBitsPerSample]::Sixteen, [System.Speech.AudioFormat.AudioChannel]::Mono)
$segments = Get-Content -Raw -Encoding UTF8 '{manifest}' | ConvertFrom-Json
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
try {{
    $installed = $synth.GetInstalledVoices() | Where-Object {{ $_.Enabled }}
    $match = $installed | Where-Object {{ $_.VoiceInfo.Culture.TwoLetterISOLanguageName -eq '{language}' }} | Select-Object -First 1
    if (-not $match) {{
        # Written and exited rather than thrown: a missing voice is an expected outcome on a machine
        # that never installed one, and the user gets the message, not PowerShell's stack trace.
        $have = ($installed | ForEach-Object {{ "$($_.VoiceInfo.Name) ($($_.VoiceInfo.Culture.Name))" }}) -join ', '
        [Console]::Error.WriteLine("no '{language}' speech voice is installed. Add one in Windows Settings under Time and language, Speech. Installed: $have")
        exit 1
    }}
    $synth.SelectVoice($match.VoiceInfo.Name)
    foreach ($segment in $segments) {{
        $synth.SetOutputToWaveFile($segment.file, $format)
        $synth.Speak($segment.text)
    }}
}} finally {{
    $synth.SetOutputToNull()
    $synth.Dispose()
}}
"#,
        manifest = manifest.display(),
        language = language.code()
    );

    let output = crate::quiet(std::process::Command::new("powershell"))
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .map_err(|error| AppError::Message(format!("could not start powershell: {error}")))?;

    if !output.status.success() {
        return Err(AppError::Message(format!(
            "speech synthesis failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(())
}

/// Turns a document into an episode — the notes' own summary rather than a question bank.
///
/// Markdown headings become chapters, because that is what a heading is: the point where the
/// subject changes. Everything else is read paragraph by paragraph, with the markup stripped by
/// the same filter that keeps URLs and bullets out of the question track.
pub fn script_of_document(text: &str, pause_seconds: f32) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut chapter = 0_u32;

    for block in text.split("\n\n") {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            continue;
        }
        let heading = trimmed.starts_with('#');
        let spoken = speakable(&trimmed.replace('\n', " "));
        if spoken.is_empty() {
            continue;
        }
        if heading {
            chapter += 1;
        }
        segments.push(Segment {
            question_number: chapter,
            starts_chapter: heading,
            text: sentence(&spoken),
            pause_after_seconds: if heading { 0.4 } else { pause_seconds },
        });
    }

    segments
}

/// Records a document with whichever voice the options name.
pub fn generate_document(
    text: &str,
    title: &str,
    options: &Options,
    work_dir: &Path,
    destination: &Path,
) -> AppResult<Episode> {
    let segments = script_of_document(text, options.pause_seconds.min(1.5));
    if segments.is_empty() {
        return Err(AppError::Message("there is nothing to read".into()));
    }

    let (spoken, rate) = match &options.voice {
        Some(voice) => through_pack(&segments, voice, work_dir)?,
        None => through_windows(&segments, options.language, work_dir)?,
    };

    let mut audio = Vec::new();
    let mut chapters = Vec::new();
    for (segment, pcm) in segments.iter().zip(&spoken) {
        if segment.starts_chapter {
            chapters.push(Chapter {
                question_number: segment.question_number,
                offset_ms: bytes_to_ms(audio.len(), rate),
                title: segment.text.trim_end_matches('.').to_owned(),
            });
        }
        audio.extend_from_slice(pcm);
        audio.extend(silence(segment.pause_after_seconds, rate));
    }

    // A document with no headings still has one chapter: the document.
    if chapters.is_empty() {
        chapters.push(Chapter {
            question_number: 0,
            offset_ms: 0,
            title: title.to_owned(),
        });
    }

    let encoded = match options.format {
        Format::Mp3 => mp3(&audio, rate)?,
        Format::Wav => wav(&audio, rate),
    };
    std::fs::write(destination, encoded).map_err(|error| AppError::Message(error.to_string()))?;

    Ok(Episode {
        path: destination.to_string_lossy().to_string(),
        duration_ms: bytes_to_ms(audio.len(), rate),
        chapters,
    })
}

/// Reads one sentence with the Windows voice for a language, so the settings page can preview it
/// the same way it previews a downloaded pack.
pub fn preview(work_dir: &Path, language: Language, text: &str) -> AppResult<()> {
    let staging = work_dir.join("preview");
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|error| AppError::Message(error.to_string()))?;

    let spoken = staging.join("0000.wav");
    let manifest_path = staging.join("segments.json");
    std::fs::write(
        &manifest_path,
        serde_json::to_string(&serde_json::json!([{
            "file": spoken.to_string_lossy(),
            "text": text,
        }]))?,
    )
    .map_err(|error| AppError::Message(error.to_string()))?;

    synthesize(&manifest_path, language)?;
    let pcm = pcm_of(
        &std::fs::read(&spoken).map_err(|error| AppError::Message(error.to_string()))?,
        WINDOWS_RATE,
    )?;
    let _ = std::fs::remove_dir_all(&staging);

    let samples: Vec<f32> = pcm
        .chunks_exact(2)
        .map(|pair| f32::from(i16::from_le_bytes([pair[0], pair[1]])) / 32768.0)
        .collect();
    crate::voice::play(&samples, WINDOWS_RATE)
}

fn bytes_to_ms(len: usize, rate: u32) -> u64 {
    (len as u64 * 1000) / u64::from(rate * BYTES_PER_SAMPLE)
}

fn silence(seconds: f32, rate: u32) -> Vec<u8> {
    let samples = (seconds.max(0.0) * rate as f32) as usize;
    vec![0u8; samples * BYTES_PER_SAMPLE as usize]
}

/// Returns the `data` chunk of a RIFF/WAVE buffer.
///
/// The synthesiser is pinned to one format, so the segments concatenate as raw PCM — but the header
/// is still checked rather than assumed, because a wrong sample rate would play back at the wrong
/// pitch with no error anywhere.
fn pcm_of(bytes: &[u8], rate: u32) -> AppResult<Vec<u8>> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(AppError::Message("segment is not a WAVE file".into()));
    }

    let mut cursor = 12;
    let mut data: Option<Vec<u8>> = None;
    let mut checked = false;

    while cursor + 8 <= bytes.len() {
        let id = &bytes[cursor..cursor + 4];
        let size =
            u32::from_le_bytes(bytes[cursor + 4..cursor + 8].try_into().expect("4 bytes")) as usize;
        let start = cursor + 8;
        let end = (start + size).min(bytes.len());

        match id {
            b"fmt " if size >= 16 => {
                let channels = u16::from_le_bytes(bytes[start + 2..start + 4].try_into().unwrap());
                let found = u32::from_le_bytes(bytes[start + 4..start + 8].try_into().unwrap());
                let bits = u16::from_le_bytes(bytes[start + 14..start + 16].try_into().unwrap());
                if channels != CHANNELS || found != rate || bits != BITS {
                    return Err(AppError::Message(format!(
                        "segment is {found} Hz / {bits} bit / {channels} ch, expected \
                         {rate} Hz / {BITS} bit / {CHANNELS} ch"
                    )));
                }
                checked = true;
            }
            b"data" => data = Some(bytes[start..end].to_vec()),
            _ => {}
        }
        cursor = start + size + (size % 2);
    }

    if !checked {
        return Err(AppError::Message("segment has no fmt chunk".into()));
    }
    data.ok_or_else(|| AppError::Message("segment has no data chunk".into()))
}

/// PCM to MPEG-2 Layer III.
///
/// The chapter offsets are not recomputed for this: they are times, and the encoder rearranges the
/// bytes, not the clock. It does pad the tail to a whole frame, so the file runs up to one frame —
/// about 26 ms here — longer than the PCM it was given.
fn mp3(pcm: &[u8], rate: u32) -> AppResult<Vec<u8>> {
    let samples: Vec<i16> = pcm
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
        .collect();

    let mut encoder = Mp3Encoder::new(Mp3EncoderConfig {
        bitrate_kbps: MP3_KBPS,
        vbr_quality: None,
    });
    encoder
        .push_pcm_s16(&samples, CHANNELS, rate)
        .map_err(|error| AppError::Message(format!("mp3 encode: {error}")))?;
    encoder.finish();

    let mut out = Vec::new();
    loop {
        match encoder.next_packet() {
            Ok(packet) => out.extend_from_slice(&packet),
            Err(rusty_mp3::Error::Eof) => break,
            Err(error) => return Err(AppError::Message(format!("mp3 encode: {error}"))),
        }
    }
    Ok(out)
}

fn wav(pcm: &[u8], rate: u32) -> Vec<u8> {
    let byte_rate = rate * BYTES_PER_SAMPLE;
    let mut out = Vec::with_capacity(pcm.len() + 44);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&((36 + pcm.len()) as u32).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&CHANNELS.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&(BYTES_PER_SAMPLE as u16).to_le_bytes());
    out.extend_from_slice(&BITS.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    out.extend_from_slice(pcm);
    out
}

pub fn default_destination(data_dir: &Path, title: &str, format: Format) -> PathBuf {
    let slug: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    data_dir.join(format!("{}.{}", slug.trim_matches('-'), format.extension()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::AnswerOption;
    use openexamtrainer_ingest::model::QuestionKind;

    fn question(number: u32, letters: &[char], explanation: &str) -> QuestionDto {
        QuestionDto {
            id: number as i64,
            number,
            topic: Some(1),
            kind: QuestionKind::SingleChoice,
            stem: format!("Stem {number}"),
            options: vec![
                AnswerOption {
                    letter: 'A',
                    text: "first".into(),
                    is_correct: letters.contains(&'A'),
                },
                AnswerOption {
                    letter: 'B',
                    text: "second".into(),
                    is_correct: letters.contains(&'B'),
                },
            ],
            answer_letters: letters.to_vec(),
            matrix: Vec::new(),
            explanation: explanation.into(),
            references: Vec::new(),
            source_page: 1,
            confidence: 1.0,
            needs_source: false,
            warnings: Vec::new(),
            figures: Vec::new(),
        }
    }

    #[test]
    fn a_question_becomes_a_prompt_a_pause_an_answer_and_a_rationale() {
        let segments = build_script(
            &[question(1, &['B'], "Because it is B.")],
            &Options::default(),
        );

        assert_eq!(segments.len(), 3);
        assert!(segments[0].text.starts_with("Question 1. Stem 1."));
        assert!(segments[0].text.contains("Option A: first."));
        assert_eq!(segments[0].pause_after_seconds, 4.0);
        assert!(segments[0].starts_chapter);
        assert_eq!(segments[1].text, "The answer is B.");
        assert_eq!(segments[2].text, "Because it is B.");
    }

    /// The words around the question change language; the question does not. Its stem and its
    /// options are the user's imported material, and a translated exam question is a different
    /// question.
    #[test]
    fn german_changes_the_scaffolding_and_leaves_the_material_alone() {
        let options = Options {
            language: Language::De,
            ..Options::default()
        };

        let segments = build_script(&[question(1, &['B'], "Weil B stimmt.")], &options);

        assert!(segments[0].text.starts_with("Frage 1. Stem 1."));
        assert!(segments[0].text.contains("Antwort A: first."));
        assert_eq!(segments[1].text, "Die richtige Antwort ist B.");

        let plural = build_script(&[question(2, &['A', 'B'], "")], &options);
        assert_eq!(plural[1].text, "Die richtigen Antworten sind A und B.");

        let none = build_script(&[question(3, &[], "")], &options);
        assert_eq!(
            none[1].text,
            "Die Quelle nennt für diese Frage keine Antwort."
        );
    }

    #[test]
    fn recall_mode_records_only_the_questions() {
        let options = Options {
            include_answer: false,
            include_explanation: false,
            pause_seconds: 6.0,
            ..Options::default()
        };

        let segments = build_script(
            &[question(1, &['A'], "why"), question(2, &['B'], "why")],
            &options,
        );

        assert_eq!(segments.len(), 2);
        assert!(segments.iter().all(|s| s.starts_chapter));
        assert!(segments.iter().all(|s| s.pause_after_seconds == 6.0));
    }

    #[test]
    fn a_multi_answer_question_is_read_as_a_plural() {
        let segments = build_script(&[question(1, &['A', 'B'], "")], &Options::default());

        assert_eq!(segments[1].text, "The answers are A and B.");
    }

    #[test]
    fn urls_and_markup_never_reach_the_synthesiser() {
        let spoken = speakable("See * https://learn.microsoft.com/en-us/azure/x for `details`");

        assert_eq!(spoken, "See for details");
    }

    #[test]
    fn segments_concatenate_into_one_track_with_the_silence_between_them() {
        let first = wav(&[1u8; 4410 * 2], WINDOWS_RATE);
        let second = wav(&[2u8; 2205 * 2], WINDOWS_RATE);

        let mut audio = pcm_of(&first, WINDOWS_RATE).expect("first");
        audio.extend(silence(0.5, WINDOWS_RATE));
        audio.extend(pcm_of(&second, WINDOWS_RATE).expect("second"));

        // 200 ms + 500 ms + 100 ms
        assert_eq!(bytes_to_ms(audio.len(), WINDOWS_RATE), 800);
        let round_tripped = pcm_of(&wav(&audio, WINDOWS_RATE), WINDOWS_RATE).expect("round trip");
        assert_eq!(round_tripped, audio);
    }

    /// Checked rather than assumed, the same way the WAV path checks its segments: encode a known
    /// tone and decode it back. An encoder that emits silence, or clocks the stream wrongly, would
    /// otherwise ship as a file that plays as nothing or at the wrong pitch, with no error anywhere.
    #[test]
    fn an_mp3_decodes_back_to_the_tone_that_went_into_it() {
        let count = WINDOWS_RATE as usize;
        let mut pcm = Vec::with_capacity(count * 2);
        for n in 0..count {
            let phase = std::f32::consts::TAU * 440.0 * (n as f32 / WINDOWS_RATE as f32);
            let value = (phase.sin() * 0.5 * f32::from(i16::MAX)) as i16;
            pcm.extend_from_slice(&value.to_le_bytes());
        }

        let encoded = mp3(&pcm, WINDOWS_RATE).expect("encode");
        // Eleven set sync bits open every Layer III frame, including the leading Xing header.
        assert_eq!(encoded[0], 0xFF, "no frame sync");
        assert_eq!(encoded[1] & 0xE0, 0xE0, "no frame sync");
        assert!(encoded.len() < pcm.len() / 4, "{} bytes", encoded.len());

        let mut decoder = rusty_mp3::Mp3Decoder::new();
        decoder.push(&encoded);
        decoder.flush();

        let mut decoded = Vec::new();
        let (mut rate, mut channels) = (0, 0);
        while let Ok(frame) = decoder.next_frame() {
            rate = frame.sample_rate;
            channels = frame.channels;
            decoded.extend_from_slice(&frame.samples);
        }

        assert_eq!(rate, WINDOWS_RATE);
        assert_eq!(channels, CHANNELS);

        // Both ends pad to whole frames, so the length lands near one second rather than on it.
        let decoded_ms = (decoded.len() as u64 * 1000) / u64::from(WINDOWS_RATE);
        assert!(decoded_ms.abs_diff(1000) < 150, "{decoded_ms} ms");

        // A half-amplitude sine is 0.354 RMS. Anything near zero means it encoded to silence.
        let rms = (decoded.iter().map(|s| s * s).sum::<f32>() / decoded.len() as f32).sqrt();
        assert!(rms > 0.2, "decoded to near silence: {rms}");
    }

    /// The only proof the whole path works: script → PowerShell → per-segment WAVs → one track.
    #[test]
    #[cfg(windows)]
    fn an_episode_is_synthesised_end_to_end() {
        let work = std::env::temp_dir().join("openexamtrainer-episode-test");
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&work).expect("work dir");
        let destination = work.join("episode.wav");

        let episode = generate(
            &[question(1, &['B'], "Because it is B.")],
            // WAV rather than the default, so the written file can be compared against the track
            // byte for byte. What the MP3 encoder does with that track is the round trip's job.
            &Options {
                include_answer: true,
                include_explanation: true,
                pause_seconds: 1.0,
                format: Format::Wav,
                ..Options::default()
            },
            &work,
            &destination,
        )
        .expect("episode");

        assert_eq!(episode.chapters.len(), 1);
        assert_eq!(episode.chapters[0].offset_ms, 0);
        assert_eq!(episode.chapters[0].title, "Question 1");
        assert!(episode.duration_ms > 2_000, "{} ms", episode.duration_ms);

        let written = std::fs::read(&destination).expect("file");
        assert_eq!(&written[0..4], b"RIFF");
        assert_eq!(
            bytes_to_ms(
                pcm_of(&written, WINDOWS_RATE).expect("pcm").len(),
                WINDOWS_RATE
            ),
            episode.duration_ms
        );

        // The same episode in the format the app actually defaults to. Encoding the real spoken
        // track rather than a tone is the point: a synthesised voice is what this ships.
        let as_mp3 = work.join("episode.mp3");
        let mp3_episode = generate(
            &[question(1, &['B'], "Because it is B.")],
            &Options {
                include_answer: true,
                include_explanation: true,
                pause_seconds: 1.0,
                format: Format::Mp3,
                ..Options::default()
            },
            &work,
            &as_mp3,
        )
        .expect("mp3 episode");

        let encoded = std::fs::read(&as_mp3).expect("mp3 file");
        assert_eq!(encoded[0], 0xFF, "no frame sync");
        assert_eq!(encoded[1] & 0xE0, 0xE0, "no frame sync");
        assert!(
            encoded.len() < written.len() / 4,
            "{} bytes against {} as WAV",
            encoded.len(),
            written.len()
        );
        // Chapters are times, so they survive the change of container untouched.
        assert_eq!(mp3_episode.chapters, episode.chapters);

        let _ = std::fs::remove_dir_all(&work);
    }

    /// Averaged Goertzel power at one frequency, in dB. Blocked rather than run over the whole
    /// signal in one pass: a single long Goertzel is dominated by whatever the signal happened to
    /// be doing, and the point here is to compare two signals band by band.
    #[cfg(windows)]
    fn band_db(samples: &[f32], hz: f32) -> f32 {
        const BLOCK: usize = 1024;
        let w = std::f32::consts::TAU * hz / WINDOWS_RATE as f32;
        let coeff = 2.0 * w.cos();
        let mut total = 0.0f64;
        let mut blocks = 0usize;
        for chunk in samples.chunks_exact(BLOCK) {
            let (mut s1, mut s2) = (0.0f32, 0.0f32);
            for &x in chunk {
                let s0 = coeff.mul_add(s1, x) - s2;
                s2 = s1;
                s1 = s0;
            }
            total += f64::from(s1.mul_add(s1, s2 * s2) - coeff * s1 * s2);
            blocks += 1;
        }
        10.0 * ((total / blocks.max(1) as f64).max(1e-20)).log10() as f32
    }

    /// What the MP3 encoder costs the synthesised voice, in numbers rather than adjectives.
    ///
    /// A diagnostic, not a check: there is no threshold here that anyone agreed on, and a run that
    /// prints a worse figure than yesterday is information, not a failure.
    ///
    /// `cargo test --manifest-path src-tauri/Cargo.toml -- --ignored --nocapture mp3_against_the_wav`
    #[test]
    #[ignore = "diagnostic, not a check"]
    #[cfg(windows)]
    fn mp3_against_the_wav_it_came_from() {
        let work = std::env::temp_dir().join("openexamtrainer-quality");
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&work).expect("work dir");
        let destination = work.join("episode.wav");

        let questions: Vec<QuestionDto> = (1..=3)
            .map(|n| {
                question(
                    n,
                    &['B'],
                    "Anomaly detection identifies unusual patterns, which is why it covers fraud.",
                )
            })
            .collect();
        generate(
            &questions,
            &Options {
                include_answer: true,
                include_explanation: true,
                pause_seconds: 0.5,
                format: Format::Wav,
                ..Options::default()
            },
            &work,
            &destination,
        )
        .expect("episode");

        let written = std::fs::read(&destination).expect("wav");
        let pcm = pcm_of(&written, WINDOWS_RATE).expect("pcm");
        let source: Vec<f32> = pcm
            .chunks_exact(2)
            .map(|p| f32::from(i16::from_le_bytes([p[0], p[1]])) / 32768.0)
            .collect();

        let encoded = mp3(&pcm, WINDOWS_RATE).expect("encode");
        let mut decoder = rusty_mp3::Mp3Decoder::new();
        decoder.push(&encoded);
        decoder.flush();
        let mut decoded = Vec::new();
        while let Ok(frame) = decoder.next_frame() {
            decoded.extend_from_slice(&frame.samples);
        }
        assert!(!decoded.is_empty(), "nothing decoded");

        // The decoder emits the encoder's delay as leading samples, and the value depends on the
        // MPEG version. Rather than assume it, find it: the lag that correlates best is the lag.
        let probe = 40_000.min(source.len() / 2);
        let mut best = (0usize, f64::MIN);
        for lag in 0..3_000usize {
            if lag + probe > decoded.len() {
                break;
            }
            let score: f64 = (0..probe)
                .map(|i| f64::from(source[i]) * f64::from(decoded[lag + i]))
                .sum();
            if score > best.1 {
                best = (lag, score);
            }
        }
        let lag = best.0;
        let overlap = (source.len()).min(decoded.len() - lag);
        let (a, b) = (&source[..overlap], &decoded[lag..lag + overlap]);

        let signal: f64 = a.iter().map(|&x| f64::from(x) * f64::from(x)).sum();
        let noise: f64 = a
            .iter()
            .zip(b)
            .map(|(&x, &y)| {
                let d = f64::from(x) - f64::from(y);
                d * d
            })
            .sum();
        let snr = 10.0 * (signal / noise.max(1e-30)).log10();

        // Segmental SNR over 20 ms, skipping the pauses: an average dominated by silence says more
        // about the silence than about the voice.
        let window = (WINDOWS_RATE as usize * 20) / 1000;
        let mut segs = Vec::new();
        for (x, y) in a.chunks_exact(window).zip(b.chunks_exact(window)) {
            let s: f64 = x.iter().map(|&v| f64::from(v) * f64::from(v)).sum();
            if s / window as f64 <= 1e-6 {
                continue;
            }
            let n: f64 = x
                .iter()
                .zip(y)
                .map(|(&p, &q)| {
                    let d = f64::from(p) - f64::from(q);
                    d * d
                })
                .sum();
            segs.push(10.0 * (s / n.max(1e-30)).log10());
        }
        let seg_snr = segs.iter().sum::<f64>() / segs.len().max(1) as f64;

        let seconds = source.len() as f64 / f64::from(WINDOWS_RATE);
        println!("--- mp3 against the wav it came from ---");
        println!("  duration          {seconds:.1} s");
        println!(
            "  size              {} B mp3 vs {} B wav  ({:.1}x smaller)",
            encoded.len(),
            written.len(),
            written.len() as f64 / encoded.len() as f64
        );
        println!(
            "  actual bitrate    {:.0} kbps",
            (encoded.len() as f64 * 8.0) / seconds / 1000.0
        );
        println!(
            "  decoder delay     {lag} samples ({:.0} ms)",
            lag as f64 * 1000.0 / f64::from(WINDOWS_RATE)
        );
        println!("  SNR overall       {snr:.1} dB");
        println!(
            "  SNR segmental     {seg_snr:.1} dB over {} voiced windows",
            segs.len()
        );
        println!(
            "  peak              {:.3} in, {:.3} out",
            a.iter().fold(0.0f32, |m, v| m.max(v.abs())),
            b.iter().fold(0.0f32, |m, v| m.max(v.abs()))
        );
        println!("  band energy kept (dB relative to the WAV, 0 = untouched):");
        for hz in [
            200.0, 500.0, 1_000.0, 2_000.0, 3_000.0, 4_000.0, 5_000.0, 6_000.0, 8_000.0, 10_000.0,
        ] {
            println!(
                "    {hz:>6.0} Hz      {:+.1}",
                band_db(b, hz) - band_db(a, hz)
            );
        }

        let _ = std::fs::remove_dir_all(&work);
    }

    #[test]
    fn a_segment_in_the_wrong_format_is_refused_rather_than_played_at_the_wrong_pitch() {
        let mut bytes = wav(&[0u8; 100], WINDOWS_RATE);
        // Rewrite the sample rate in the fmt chunk to 44100.
        bytes[24..28].copy_from_slice(&44_100u32.to_le_bytes());

        let error = pcm_of(&bytes, WINDOWS_RATE).expect_err("format mismatch");

        assert!(error.to_string().contains("44100 Hz"), "{error}");
    }
}
