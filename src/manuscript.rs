//! Deterministic Markdown manuscript splitting for audiobooks.
//!
//! The manuscript is data: only ATX heading lines outside fenced code are
//! interpreted, and every chapter body is an exact slice of the source.
use crate::{
    contracts::{EmptyPolicy, PreamblePolicy, Split},
    error::{Error, Result},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitChapter {
    pub id: String,
    pub title: Option<String>,
    pub body: String,
    /// One-based inclusive source lines covered by the chapter, heading included.
    pub lines: (usize, usize),
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Skipped {
    pub line: usize,
    pub title: Option<String>,
    pub reason: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SplitReport {
    pub chapters: Vec<SplitChapter>,
    pub skipped: Vec<Skipped>,
    pub warnings: Vec<String>,
    pub front_matter: Option<(usize, usize)>,
    pub bom: bool,
}

/// Longest chapter ID that keeps `{book}--{chapter}--part-NNNN` within the 64-byte ID limit.
pub fn max_chapter_id(book: &str) -> Result<usize> {
    let available = 64usize.saturating_sub(book.len() + 13);
    if available < 8 {
        return Err(Error::input(format!(
            "Audiobook ID `{book}` is too long to name its chapters and parts; use at most 43 characters"
        )));
    }
    Ok(available)
}

/// Lowercase ASCII slug; other characters collapse to single hyphens, never `--`.
pub fn slug(title: &str, max: usize) -> String {
    let mut out = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    let mut out = out.trim_end_matches('-').to_owned();
    out.truncate(max);
    out.trim_end_matches('-').to_owned()
}

fn unique(id: String, used: &mut BTreeSet<String>, max: usize) -> String {
    if used.insert(id.clone()) {
        return id;
    }
    for n in 2.. {
        let suffix = format!("-{n}");
        let mut base = id.clone();
        base.truncate(max.saturating_sub(suffix.len()));
        let candidate = format!("{}{suffix}", base.trim_end_matches('-'));
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!()
}

fn content(line: &str) -> &str {
    line.trim_end_matches(['\n', '\r'])
}

/// ATX heading level and title, or None. Closing `#` sequences are removed.
fn heading(line: &str) -> Option<(u8, String)> {
    let line = content(line);
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }
    let rest = &line[indent..];
    let level = rest.bytes().take_while(|b| *b == b'#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let after = &rest[level..];
    if !after.is_empty() && !after.starts_with([' ', '\t']) {
        return None;
    }
    let mut title = after.trim_matches([' ', '\t']);
    let without = title.trim_end_matches('#');
    if without.is_empty() {
        title = "";
    } else if without.len() != title.len() && without.ends_with([' ', '\t']) {
        title = without.trim_end_matches([' ', '\t']);
    }
    Some((level as u8, title.to_owned()))
}

/// Opening fence character and length, for ``` and ~~~ fences.
fn fence(line: &str) -> Option<(u8, usize, bool)> {
    let line = content(line);
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }
    let rest = &line[indent..];
    let marker = *rest.as_bytes().first()?;
    if marker != b'`' && marker != b'~' {
        return None;
    }
    let count = rest.bytes().take_while(|b| *b == marker).count();
    if count < 3 {
        return None;
    }
    let bare = rest[count..].trim().is_empty();
    Some((marker, count, bare))
}

fn blank(line: &str) -> bool {
    line.trim().is_empty()
}

/// Exact body text without leading and trailing whitespace-only lines.
fn body(lines: &[&str]) -> String {
    let first = lines.iter().position(|l| !blank(l));
    let last = lines.iter().rposition(|l| !blank(l));
    match (first, last) {
        (Some(first), Some(last)) => {
            let mut text = lines[first..=last].concat();
            let trimmed = text.trim_end_matches(['\n', '\r']).len();
            text.truncate(trimmed);
            text
        }
        _ => String::new(),
    }
}

pub fn split(source: &str, options: &Split, max_id: usize) -> Result<SplitReport> {
    if !(1..=6).contains(&options.heading_level) {
        return Err(Error::input("split.heading_level must be 1–6"));
    }
    let mut report = SplitReport::default();
    let source = match source.strip_prefix('\u{feff}') {
        Some(rest) => {
            report.bom = true;
            rest
        }
        None => source,
    };
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let mut start = 0;
    if lines.first().is_some_and(|l| content(l) == "---") {
        match lines
            .iter()
            .skip(1)
            .position(|l| matches!(content(l), "---" | "..."))
        {
            Some(end) => {
                report.front_matter = Some((1, end + 2));
                start = end + 2;
            }
            None => report.warnings.push(
                "Front matter starting on line 1 is never closed; it is narrated as text".into(),
            ),
        }
    }
    // (line index, level, title) of every heading outside fences.
    let mut headings = vec![];
    let mut open: Option<(u8, usize, usize)> = None;
    let mut previous_text = false;
    for (i, line) in lines.iter().enumerate().skip(start) {
        if let Some((marker, count, _)) = open {
            if let Some((m, c, bare)) = fence(line)
                && m == marker
                && c >= count
                && bare
            {
                open = None;
            }
            previous_text = false;
            continue;
        }
        if let Some((marker, count, _)) = fence(line) {
            open = Some((marker, count, i + 1));
            previous_text = false;
            continue;
        }
        if let Some((level, title)) = heading(line) {
            headings.push((i, level, title));
            previous_text = false;
            continue;
        }
        let trimmed = content(line).trim();
        let setext = if trimmed.len() >= 3 && trimmed.bytes().all(|b| b == b'=') {
            Some(1)
        } else if trimmed.len() >= 3 && trimmed.bytes().all(|b| b == b'-') {
            Some(2)
        } else {
            None
        };
        if previous_text && setext.is_some_and(|level| level <= options.heading_level) {
            report.warnings.push(format!(
                "Line {} underlines a heading (setext style); only # headings start chapters",
                i + 1
            ));
        }
        previous_text = !blank(line);
    }
    if let Some((_, _, line)) = open {
        report.warnings.push(format!(
            "Code fence opened on line {line} is never closed; headings after it are ignored"
        ));
    }
    let boundaries: Vec<_> = headings
        .iter()
        .filter(|(_, level, _)| *level <= options.heading_level)
        .collect();
    if boundaries.is_empty() {
        let found: BTreeSet<_> = headings.iter().map(|h| h.1).collect();
        return Err(Error::input(if found.is_empty() {
            "Manuscript has no # headings outside code; add chapter headings or list chapters explicitly".to_owned()
        } else {
            format!(
                "Manuscript has no headings at level {} or above; found levels {}; set split.heading_level accordingly",
                options.heading_level,
                found
                    .iter()
                    .map(u8::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }));
    }
    let mut used = BTreeSet::new();
    let first = boundaries[0].0;
    let preamble = body(&lines[start..first]);
    if !preamble.is_empty() {
        match options.preamble {
            PreamblePolicy::Include => report.chapters.push(SplitChapter {
                id: unique("preamble".into(), &mut used, max_id),
                title: None,
                body: preamble,
                lines: (start + 1, first),
            }),
            PreamblePolicy::Skip => report.skipped.push(Skipped {
                line: start + 1,
                title: None,
                reason: "preamble before the first chapter heading (split.preamble: skip)".into(),
            }),
        }
    }
    for (n, (line, _, title)) in boundaries.iter().enumerate() {
        let end = boundaries.get(n + 1).map_or(lines.len(), |next| next.0);
        let text = body(&lines[line + 1..end]);
        let title = (!title.is_empty()).then(|| title.clone());
        if text.is_empty() {
            if options.empty_chapters == EmptyPolicy::Error {
                return Err(Error::input(format!(
                    "Manuscript heading on line {} has no text before the next chapter",
                    line + 1
                )));
            }
            report.skipped.push(Skipped {
                line: line + 1,
                title,
                reason: "no text before the next chapter heading".into(),
            });
            continue;
        }
        let position = report.chapters.len() + 1;
        let mut id = title
            .as_deref()
            .map(|t| slug(t, max_id))
            .unwrap_or_default();
        if id.is_empty() {
            id = format!("chapter-{position:02}");
            id.truncate(max_id);
        }
        report.chapters.push(SplitChapter {
            id: unique(id, &mut used, max_id),
            title,
            body: text,
            lines: (line + 1, end),
        });
    }
    if report.chapters.is_empty() {
        return Err(Error::input(
            "Manuscript chapters contain no narratable text",
        ));
    }
    for chapter in &report.chapters {
        report
            .warnings
            .extend(markup_warnings(&chapter.id, &chapter.body));
    }
    Ok(report)
}

/// Markdown that speech models may read literally. Text is never rewritten.
pub fn markup_warnings(id: &str, text: &str) -> Vec<String> {
    let mut found = BTreeMap::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('|') {
            found.insert("tables", ());
        }
        if fence(line).is_some() {
            found.insert("code fences", ());
        }
        if trimmed.starts_with('#') && heading(line).is_some() {
            found.insert("subheadings marked with #", ());
        }
    }
    if text.contains("](") {
        found.insert("links or images", ());
    }
    if text.contains("**") || text.contains("__") || text.contains("`") {
        found.insert("emphasis or code markers", ());
    }
    let bytes = text.as_bytes();
    if bytes
        .windows(2)
        .any(|w| w[0] == b'<' && (w[1].is_ascii_alphabetic() || w[1] == b'/'))
    {
        found.insert("HTML tags", ());
    }
    found
        .into_keys()
        .map(|kind| {
            format!("Chapter `{id}` contains {kind}; speech models may read the symbols aloud")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn opts(level: u8) -> Split {
        Split {
            heading_level: level,
            ..Split::default()
        }
    }
    #[test]
    fn splits_at_atx_headings_with_exact_bodies() {
        let source = "# One #\n\nFirst line.\nSecond line.\n\n# Two\nText two.\n";
        let r = split(source, &opts(1), 40).unwrap();
        assert_eq!(r.chapters.len(), 2);
        assert_eq!(r.chapters[0].id, "one");
        assert_eq!(r.chapters[0].title.as_deref(), Some("One"));
        assert_eq!(r.chapters[0].body, "First line.\nSecond line.");
        assert_eq!(r.chapters[0].lines, (1, 5));
        assert_eq!(r.chapters[1].body, "Text two.");
        assert_eq!(split(source, &opts(1), 40).unwrap(), r);
    }
    #[test]
    fn fences_front_matter_bom_and_crlf() {
        let source = "\u{feff}---\ntitle: x\n---\n# A\r\nBody\r\n```\n# not a heading\n```\nAfter\r\n# B\nTwo\n";
        let r = split(source, &opts(1), 40).unwrap();
        assert!(r.bom);
        assert_eq!(r.front_matter, Some((1, 3)));
        assert_eq!(r.chapters.len(), 2);
        assert_eq!(
            r.chapters[0].body,
            "Body\r\n```\n# not a heading\n```\nAfter"
        );
        assert!(r.warnings.iter().any(|w| w.contains("code fences")));
        let unclosed = split("# A\ntext\n~~~~\n# Hidden\n", &opts(1), 40).unwrap();
        assert_eq!(unclosed.chapters.len(), 1);
        assert!(unclosed.warnings.iter().any(|w| w.contains("never closed")));
        let open_front = split("---\nnot closed\n# A\nText\n", &opts(1), 40).unwrap();
        assert!(open_front.front_matter.is_none());
        assert_eq!(open_front.chapters[0].id, "preamble");
    }
    #[test]
    fn empty_chapters_preamble_and_levels() {
        let source = "Intro words.\n\n# Part One\n\n## Chapter 1\nHello.\n## Chapter 2\nBye.\n";
        let r = split(source, &opts(2), 40).unwrap();
        assert_eq!(
            r.chapters.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
            ["preamble", "chapter-1", "chapter-2"]
        );
        assert_eq!(r.skipped.len(), 1);
        assert_eq!(r.skipped[0].title.as_deref(), Some("Part One"));
        let strict = Split {
            empty_chapters: EmptyPolicy::Error,
            ..opts(2)
        };
        assert!(split(source, &strict, 40).is_err());
        let skip = Split {
            preamble: PreamblePolicy::Skip,
            ..opts(2)
        };
        assert_eq!(
            split(source, &skip, 40).unwrap().chapters[0].id,
            "chapter-1"
        );
        let error = split("## Only\ntext\n", &opts(1), 40).unwrap_err();
        assert!(error.to_string().contains("found levels 2"));
        assert!(split("plain text\n", &opts(1), 40).is_err());
    }
    #[test]
    fn non_headings_and_setext_warnings() {
        let source = "# A\n#tag\n    # indented\n\\# escaped\nTitle\n===\n";
        let r = split(source, &opts(1), 40).unwrap();
        assert_eq!(r.chapters.len(), 1);
        assert!(r.chapters[0].body.contains("#tag"));
        assert!(r.warnings.iter().any(|w| w.contains("setext")));
    }
    #[test]
    fn slugs_are_unique_bounded_and_never_double_hyphenated() {
        assert_eq!(slug("Chapter 1 — The Start!", 40), "chapter-1-the-start");
        assert_eq!(slug("a--b", 40), "a-b");
        assert_eq!(slug("第一章", 40), "");
        assert_eq!(slug("abcdefghij", 5), "abcde");
        let source = "# Same\nOne\n# Same\nTwo\n# 第一章\nThree\n";
        let r = split(source, &opts(1), 40).unwrap();
        assert_eq!(
            r.chapters.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
            ["same", "same-2", "chapter-03"]
        );
        assert!(max_chapter_id(&"x".repeat(44)).is_err());
        assert_eq!(max_chapter_id("book").unwrap(), 47);
    }
}
