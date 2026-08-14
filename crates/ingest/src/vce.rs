use std::path::Path;

/// Container signatures seen in the wild, and whether a decoder exists for them.
///
/// The Avanset container is encrypted — the AZ-900 samples measure 7.998 bits of entropy per byte
/// over 1.3 MB — so a decoder is reverse-engineering work per version, not parsing. No row gains a
/// decoder without a fixture test that round-trips a real file to questions.
const KNOWN: &[(&[u8], &str)] = &[(&[0x85, 0xA8, 0x06, 0x02, 0x04, 0x00, 0x00, 0x00], "avanset")];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probe {
    Unsupported {
        container: String,
        signature: String,
    },
    Unknown {
        signature: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum VceError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{path} is too short to be a VCE container")]
    TooShort { path: String },
    #[error(
        "no decoder for the {container} container (signature {signature}). \
         Export the exam to PDF from your VCE player and import that file instead."
    )]
    NoDecoder {
        container: String,
        signature: String,
    },
    #[error(
        "unrecognised container (signature {signature}). \
         Report this signature so the support matrix can grow."
    )]
    UnknownContainer { signature: String },
}

pub fn probe(path: &Path) -> Result<Probe, VceError> {
    let bytes = std::fs::read(path).map_err(|source| VceError::Read {
        path: path.display().to_string(),
        source,
    })?;
    if bytes.len() < 8 {
        return Err(VceError::TooShort {
            path: path.display().to_string(),
        });
    }
    let head = &bytes[..8];
    let signature = head
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");

    Ok(match KNOWN.iter().find(|(magic, _)| *magic == head) {
        Some((_, container)) => Probe::Unsupported {
            container: (*container).to_string(),
            signature,
        },
        None => Probe::Unknown { signature },
    })
}

/// The import entry point. Always fails today, but fails with the specific reason and the
/// documented alternative rather than a generic parse error.
pub fn import(path: &Path) -> Result<Vec<crate::model::Question>, VceError> {
    match probe(path)? {
        Probe::Unsupported {
            container,
            signature,
        } => Err(VceError::NoDecoder {
            container,
            signature,
        }),
        Probe::Unknown { signature } => Err(VceError::UnknownContainer { signature }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_file(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        let mut file = std::fs::File::create(&path).expect("create");
        file.write_all(bytes).expect("write");
        path
    }

    #[test]
    fn the_avanset_signature_is_recognised_but_unsupported() {
        let path = temp_file(
            "openexamtrainer-avanset.vce",
            &[0x85, 0xA8, 0x06, 0x02, 0x04, 0x00, 0x00, 0x00, 0xDE, 0xAD],
        );

        assert_eq!(
            probe(&path).expect("probe"),
            Probe::Unsupported {
                container: "avanset".to_string(),
                signature: "85 a8 06 02 04 00 00 00".to_string(),
            }
        );

        let error = import(&path).expect_err("no decoder exists");
        assert!(error.to_string().contains("Export the exam to PDF"));
    }

    #[test]
    fn an_unrecognised_signature_is_reported_verbatim() {
        let path = temp_file("openexamtrainer-unknown.vce", &[1, 2, 3, 4, 5, 6, 7, 8]);

        let error = import(&path).expect_err("unknown container");
        assert!(error.to_string().contains("01 02 03 04 05 06 07 08"));
    }
}
