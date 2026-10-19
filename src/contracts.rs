use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
    clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Openai,
    Google,
    Fish,
    Openrouter,
    Elevenlabs,
}
impl Provider {
    pub fn key(self) -> &'static str {
        match self {
            Self::Openai => "OPENAI_API_KEY",
            Self::Google => "GEMINI_API_KEY",
            Self::Fish => "FISH_API_KEY",
            Self::Openrouter => "OPENROUTER_API_KEY",
            Self::Elevenlabs => "ELEVENLABS_API_KEY",
        }
    }
}
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
    clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum Media {
    Image,
    Infographic,
    Speech,
    Podcast,
    Audiobook,
    Music,
    Sfx,
    Text,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub provider: Provider,
    pub model: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub requests: u32,
    pub images: u32,
    pub audio_seconds: u32,
    pub speech_chars: u32,
    pub revisions: u32,
    pub attempts: u32,
    pub concurrency: usize,
    pub per_provider: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            requests: 20,
            images: 8,
            audio_seconds: 1800,
            speech_chars: 60000,
            revisions: 3,
            attempts: 3,
            concurrency: 4,
            per_provider: 2,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[schemars(range(min = 1, max = 2))]
    pub schema_version: u32,
    #[serde(default)]
    pub defaults: BTreeMap<Media, Selection>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub default_settings: BTreeMap<Media, Settings>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub default_locales: BTreeMap<Media, String>,
    #[serde(default)]
    pub consistency: Consistency,
    #[serde(default)]
    pub limits: Limits,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: 2,
            defaults: BTreeMap::new(),
            default_settings: BTreeMap::new(),
            default_locales: BTreeMap::new(),
            consistency: Consistency::default(),
            limits: Limits::default(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Consistency {
    /// Known original identity assets always pin provider/model; this flag also rejects unknown sources.
    pub require_known_identity_model: bool,
    pub inherit_source_settings: bool,
}
impl Default for Consistency {
    fn default() -> Self {
        Self {
            require_known_identity_model: false,
            inherit_source_settings: true,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssetOrigin {
    pub selection: Selection,
    #[serde(default)]
    pub settings: Settings,
    pub locale: Option<String>,
    pub sha256: String,
    pub artifact: Option<String>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_fidelity: Option<String>,
    pub image_quality: Option<String>,
    pub format: Option<String>,
    pub size: Option<String>,
    pub duration_seconds: Option<u32>,
    pub voice: Option<String>,
    pub seed: Option<u64>,
    pub instrumental: Option<bool>,
}
impl Settings {
    /// Overlay explicitly present values, preserving unspecified defaults.
    pub fn overlay(&mut self, other: &Self) {
        if other.input_fidelity.is_some() {
            self.input_fidelity = other.input_fidelity.clone();
        }
        if other.image_quality.is_some() {
            self.image_quality = other.image_quality.clone();
        }
        if other.format.is_some() {
            self.format = other.format.clone();
        }
        if other.size.is_some() {
            self.size = other.size.clone();
        }
        if other.duration_seconds.is_some() {
            self.duration_seconds = other.duration_seconds;
        }
        if other.voice.is_some() {
            self.voice = other.voice.clone();
        }
        if other.seed.is_some() {
            self.seed = other.seed;
        }
        if other.instrumental.is_some() {
            self.instrumental = other.instrumental;
        }
    }
    pub fn common(&mut self, other: &Self) {
        if self.input_fidelity != other.input_fidelity {
            self.input_fidelity = None;
        }
        if self.image_quality != other.image_quality {
            self.image_quality = None;
        }
        if self.format != other.format {
            self.format = None;
        }
        if self.size != other.size {
            self.size = None;
        }
        if self.duration_seconds != other.duration_seconds {
            self.duration_seconds = None;
        }
        if self.voice != other.voice {
            self.voice = None;
        }
        if self.seed != other.seed {
            self.seed = None;
        }
        if self.instrumental != other.instrumental {
            self.instrumental = None;
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Visual {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub origins: BTreeMap<String, AssetOrigin>,
    pub description: PathBuf,
    pub selection: Selection,
    #[serde(default)]
    pub references: Vec<PathBuf>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Voice {
    pub description: PathBuf,
    pub selection: Selection,
    pub binding: Option<String>,
    #[serde(default)]
    pub references: Vec<PathBuf>,
    #[serde(default)]
    pub samples: Vec<VoiceSample>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VoiceSample {
    pub audio: PathBuf,
    pub transcript: PathBuf,
    pub consent: PathBuf,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConsentPurpose {
    ReferenceSynthesis,
    SavedVoiceClone,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VoiceConsent {
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    pub subject: String,
    pub authorized_by: String,
    pub granted: bool,
    pub purposes: Vec<ConsentPurpose>,
    pub evidence: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VoiceClone {
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    pub id: String,
    pub character: PathBuf,
    pub name: String,
    pub recordings: Vec<PathBuf>,
    pub consent: PathBuf,
    #[serde(default)]
    pub bind: bool,
    #[serde(default)]
    pub replace_binding: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Character {
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    pub id: String,
    pub persona: PathBuf,
    pub visual: Option<Visual>,
    pub voice: Option<Voice>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InfoSection {
    pub heading: String,
    pub copy: String,
    #[serde(default)]
    pub values: Vec<InfoValue>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InfoValue {
    pub label: String,
    pub value: String,
    pub unit: Option<String>,
    pub source: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Infographic {
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    pub title: String,
    pub audience: String,
    pub key_message: String,
    pub canvas: String,
    pub layout: String,
    pub sections: Vec<InfoSection>,
    pub visual_brief: PathBuf,
    #[serde(default)]
    pub references: Vec<InfoReference>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceRole {
    Identity,
    Style,
    Layout,
    Content,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InfoReference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<AssetOrigin>,
    pub path: PathBuf,
    pub role: ReferenceRole,
}
/// A legacy plain path, or an explicitly role-labelled image reference.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum JobReference {
    Path(PathBuf),
    Detailed(Box<InfoReference>),
}
impl JobReference {
    pub fn path(&self) -> &std::path::Path {
        match self {
            Self::Path(path) => path,
            Self::Detailed(reference) => &reference.path,
        }
    }
    pub fn role(&self) -> ReferenceRole {
        match self {
            Self::Path(_) => ReferenceRole::Content,
            Self::Detailed(reference) => reference.role,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Turn {
    pub id: String,
    pub character: Option<PathBuf>,
    pub text_file: PathBuf,
    pub selection: Option<Selection>,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub voice_samples: Vec<VoiceSample>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CastMember {
    pub alias: String,
    pub character: PathBuf,
    pub direction: Option<PathBuf>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Assembly {
    #[serde(default)]
    pub pause_ms: u32,
    #[serde(default)]
    pub turn_gain_db: BTreeMap<String, f32>,
    /// Optional integrated loudness target; peak ceiling is -1 dBFS.
    pub loudness_lufs: Option<f64>,
}
/// Published audio container for audiobook chapters and the full book.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AudioFormat {
    #[default]
    Wav,
    Mp3,
}
impl AudioFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
        }
    }
}
/// `chapters` publishes one file per chapter; `full` also publishes the combined book.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    Chapters,
    #[default]
    Full,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EmptyPolicy {
    #[default]
    Skip,
    Error,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PreamblePolicy {
    #[default]
    Include,
    Skip,
}
/// How a Markdown manuscript is divided into chapters. Only ATX headings are boundaries.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Split {
    #[schemars(range(min = 1, max = 6))]
    pub heading_level: u8,
    #[serde(default)]
    pub empty_chapters: EmptyPolicy,
    #[serde(default)]
    pub preamble: PreamblePolicy,
}
impl Default for Split {
    fn default() -> Self {
        Self {
            heading_level: 1,
            empty_chapters: EmptyPolicy::Skip,
            preamble: PreamblePolicy::Include,
        }
    }
}
/// One explicitly listed audiobook chapter; its text file holds the exact narrated words.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Chapter {
    pub id: String,
    pub title: Option<String>,
    pub text_file: PathBuf,
    /// Optional chapter-specific delivery direction; never spoken.
    pub brief: Option<PathBuf>,
    #[serde(default)]
    pub settings: Settings,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AudiobookOptions {
    #[serde(default)]
    pub output_mode: OutputMode,
    #[serde(default)]
    pub format: AudioFormat,
    /// Constant MP3 bitrate; 32–160 kbps at the 24 kHz mono working rate (default 64).
    pub mp3_bitrate_kbps: Option<u16>,
    /// Speak each chapter title before its text.
    #[serde(default)]
    pub announce_titles: bool,
    /// Upper bound for one provider request; never above the model's own limit.
    pub max_chars_per_request: Option<u32>,
    pub title: Option<String>,
    pub author: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AudiobookRole {
    Book,
    Chapter,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChapterInfo {
    /// One-based position in the complete book, stable across chapter subsets.
    pub index: u32,
    pub count: u32,
    pub title: Option<String>,
    pub spoken_title: bool,
    pub source: String,
    pub text_sha256: String,
    pub chars: u64,
}
/// Resolved audiobook packaging carried by book and chapter jobs, never by provider leaves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AudiobookPlan {
    pub role: AudiobookRole,
    pub output_mode: OutputMode,
    pub format: AudioFormat,
    pub mp3_bitrate_kbps: u16,
    pub title: Option<String>,
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chapter: Option<ChapterInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<Vec<String>>,
}
/// Sidecar written beside audiobook outputs (`<book>.chapters.json`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChapterIndex {
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    pub book: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub format: AudioFormat,
    pub output_mode: OutputMode,
    pub sample_rate: u32,
    pub channels: u16,
    pub pause_ms: u32,
    /// Relative path of the combined book, when `output_mode` is `full`.
    pub full: Option<IndexedFile>,
    pub chapters: Vec<IndexedChapter>,
    #[serde(default)]
    pub warnings: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IndexedFile {
    pub file: String,
    pub duration_seconds: f64,
    pub sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IndexedChapter {
    pub index: u32,
    pub id: String,
    pub title: Option<String>,
    pub file: String,
    pub sha256: String,
    /// Position within the combined book; with `chapters` output it is the
    /// position the chapter would have with the configured pause.
    pub start_seconds: f64,
    pub duration_seconds: f64,
    pub end_seconds: f64,
    pub source: String,
    pub text_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Job {
    #[schemars(range(min = 1, max = 2))]
    pub schema_version: u32,
    pub id: String,
    pub media: Media,
    pub selection: Option<Selection>,
    pub brief: Option<PathBuf>,
    pub text_file: Option<PathBuf>,
    pub character: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cast: Vec<CastMember>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assembly: Option<Assembly>,
    pub spec: Option<PathBuf>,
    #[serde(default)]
    pub references: Vec<JobReference>,
    #[serde(default)]
    pub voice_samples: Vec<VoiceSample>,
    #[serde(default)]
    pub turns: Vec<Turn>,
    #[serde(default)]
    pub settings: Settings,
    pub locale: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub output: Option<PathBuf>,
    /// Audiobook source: one Markdown manuscript split at headings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manuscript: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub split: Option<Split>,
    /// Audiobook source: explicit ordered chapters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chapters: Vec<Chapter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audiobook: Option<AudiobookOptions>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Batch {
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    pub jobs: Vec<PathBuf>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<AssetOrigin>,
    pub role: ReferenceRole,
    pub name: String,
    pub mime: String,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub base64: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consent_hash: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Effective {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assembly: Option<Assembly>,
    pub id: String,
    pub media: Media,
    pub selection: Selection,
    pub prompt: String,
    pub text: Option<String>,
    pub locale: Option<String>,
    pub settings: Settings,
    pub character_hash: Option<String>,
    pub references: Vec<Reference>,
    pub depends_on: Vec<String>,
    pub output: Option<PathBuf>,
    #[serde(default)]
    pub turns: Vec<Effective>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audiobook: Option<AudiobookPlan>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Queued,
    Running,
    Cancelling,
    Succeeded,
    Partial,
    Failed,
    Cancelled,
    Blocked,
    Unknown,
}
impl State {
    pub fn terminal(self) -> bool {
        !matches!(self, Self::Queued | Self::Running | Self::Cancelling)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Unchecked,
    Passed,
    NeedsRevision,
    Inconclusive,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub id: String,
    pub job_id: String,
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
    pub format: String,
    pub quality: Quality,
    pub parent: Option<String>,
    pub duration_seconds: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JobRecord {
    pub input: Effective,
    pub fingerprint: String,
    pub state: State,
    pub attempts: u32,
    pub artifacts: Vec<Artifact>,
    pub error: Option<String>,
    pub request_id: Option<String>,
    pub usage: Option<serde_json::Value>,
    pub cached: bool,
    pub retry_not_before: Option<u64>,
    pub duration_accounted: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Run {
    #[schemars(range(min = 1, max = 2))]
    pub schema_version: u32,
    pub id: String,
    pub budget_group: String,
    pub request_id: Option<String>,
    pub state: State,
    pub jobs: Vec<JobRecord>,
    pub limits: Limits,
    pub fresh: bool,
    pub overwrite: bool,
    pub parent_artifact: Option<String>,
    pub submission_fingerprint: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub requests: u32,
    pub images: u32,
    pub audio_seconds: u32,
    pub speech_chars: u32,
    pub revisions: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub check: String,
    pub severity: String,
    pub expected: String,
    pub observed: String,
    pub evidence: String,
    pub location: Option<String>,
    pub suggested_fix: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Review {
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    pub artifact: String,
    pub quality: Quality,
    pub provenance: String,
    pub findings: Vec<Finding>,
    pub reviewer: Option<Selection>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationAttempt {
    pub attempt: u32,
    pub state: State,
    pub request_id: Option<String>,
    pub usage: Option<serde_json::Value>,
    pub error: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    pub id: String,
    pub selection: Selection,
    pub input: serde_json::Value,
    pub state: State,
    pub attempts: u32,
    pub request_id: Option<String>,
    pub usage: Option<serde_json::Value>,
    pub error: Option<String>,
    pub retry_not_before: Option<u64>,
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub history: Vec<OperationAttempt>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VoiceCandidate {
    pub id: String,
    pub artifact: Artifact,
    pub language: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VoiceDesign {
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    pub id: String,
    pub character_id: String,
    pub identity_hash: String,
    pub selection: Selection,
    pub speech_selection: Selection,
    pub binding_before: Option<String>,
    pub description: String,
    pub text: String,
    pub limits: Limits,
    pub candidates: Vec<VoiceCandidate>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Revision {
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    pub parent_artifact: String,
    pub instructions: String,
    pub method: String,
    pub target_turn: Option<String>,
}
