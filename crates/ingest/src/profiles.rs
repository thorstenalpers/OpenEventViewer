use regex::Regex;

use crate::model::TextLine;

/// Per-vendor knowledge. Adding a dump vendor is a profile, never a change to `parse.rs`.
pub struct Profile {
    pub id: &'static str,
    pub marker: Regex,
    pub topic: Regex,
    pub option: Regex,
    pub answer: Regex,
    pub rationale: Regex,
    pub reference: Regex,
    pub furniture: Vec<Regex>,
    fingerprint: Option<Regex>,
}

impl Profile {
    pub fn generic() -> Self {
        Self {
            id: "generic",
            marker: Regex::new(r"(?i)^\s*(?:new\s+)?question\s*#?\s*(\d+)\b").expect("valid"),
            topic: Regex::new(r"(?i)^\s*-?\s*\(\s*exam\s+topic\s+(\d+)\s*\)").expect("valid"),
            option: Regex::new(r"^\s*([A-Z])[.)]\s+(.*)$").expect("valid"),
            answer: Regex::new(r"^\s*[Aa]nswer\s*:\s*([A-Z](?:[\s,]*[A-Z])*)\s*$").expect("valid"),
            rationale: Regex::new(r"(?i)^\s*explanation\s*(?:/\s*reference)?\s*:").expect("valid"),
            reference: Regex::new(r"(?i)^\s*references?\s*:").expect("valid"),
            furniture: Vec::new(),
            fingerprint: None,
        }
    }

    fn with_vendor(id: &'static str, host: &str, extra: &[&str]) -> Self {
        let mut profile = Self::generic();
        profile.id = id;
        profile.fingerprint =
            Some(Regex::new(&format!("(?i){}", regex::escape(id))).expect("escaped literal"));
        profile.furniture = std::iter::once(format!("(?i){}", regex::escape(host)))
            .chain(extra.iter().map(|p| (*p).to_string()))
            .map(|p| Regex::new(&p).expect("valid"))
            .collect();
        profile
    }

    pub fn certshared() -> Self {
        Self::with_vendor(
            "certshared",
            "certshared.com",
            &[r"(?i)^\s*guaranteed success with our exam guides"],
        )
    }

    pub fn certleader() -> Self {
        Self::with_vendor(
            "certleader",
            "certleader.com",
            &[r"(?i)^\s*the leader of it certification"],
        )
    }

    pub fn all() -> Vec<Self> {
        vec![Self::certshared(), Self::certleader()]
    }

    /// Picks the profile whose fingerprint the document carries, falling back to `generic`.
    pub fn detect(lines: &[TextLine]) -> Self {
        let sample: String = lines
            .iter()
            .take(400)
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        Self::all()
            .into_iter()
            .find(|profile| {
                profile
                    .fingerprint
                    .as_ref()
                    .is_some_and(|re| re.is_match(&sample))
            })
            .unwrap_or_else(Self::generic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(text: &str) -> TextLine {
        TextLine {
            page: 1,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 10.0,
            font_size: 10.0,
            text: text.to_string(),
        }
    }

    #[test]
    fn the_vendor_is_read_off_the_page_furniture() {
        let lines = vec![line(
            "Certshared now are offering 100% pass ensure AI-900 dumps!",
        )];
        assert_eq!(Profile::detect(&lines).id, "certshared");

        let lines = vec![line(
            "https://www.certleader.com/AI-900-dumps.html (85 Q&As)",
        )];
        assert_eq!(Profile::detect(&lines).id, "certleader");

        assert_eq!(Profile::detect(&[line("something else")]).id, "generic");
    }

    #[test]
    fn markers_match_the_shapes_seen_in_the_wild() {
        let profile = Profile::generic();
        for (input, expected) in [
            ("NEW QUESTION 1", "1"),
            ("QUESTION 42", "42"),
            ("Question #7", "7"),
            ("new question   103", "103"),
        ] {
            let captures = profile.marker.captures(input).expect(input);
            assert_eq!(&captures[1], expected);
        }
        assert!(profile
            .marker
            .captures("A question about questions")
            .is_none());
    }

    #[test]
    fn multi_letter_answers_are_captured_whole() {
        let profile = Profile::generic();
        assert_eq!(&profile.answer.captures("Answer: AD").unwrap()[1], "AD");
        assert_eq!(&profile.answer.captures("Answer: B, C").unwrap()[1], "B, C");
    }
}
