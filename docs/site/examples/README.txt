FRAMECHORD GUIDE EXAMPLES
Extract this examples/ folder into your target project. Run the guide's commands
from that project root. Keep this directory structure: YAML paths are relative
to their containing file. The guide's direct commands default to offline previews.
Remove --dry-run only when you intend to make paid provider requests.

Explicit model choices are examples supported by the current FrameChord release, not guarantees
of account access. No credentials are included. Set only needed keys in your
project root .env or process environment. Never commit keys.

cast.yaml needs your own mira/kito characters with accepted compatible originals.
project-config.yaml contains editable defaults. Read and choose them before init.
The podcast contains two speech requests; the batch contains an image and speech.
The audiobook makes one speech request per chapter (two) and assembles locally.
The sample schedule and original lyrics are illustrative authoring content.

SCENARIO PACKS
podcast-kit.yaml: artwork, two-speaker episode and separate intro music.
audiobook.yaml: a two-chapter audiobook from manuscript.md, one narrator, MP3 book
  with chapter markers; audiobook-chapters.yaml lists chapter files instead.
lesson-kit.yaml: structured schedule infographic and matching narration.
app-kit.yaml: an app empty-state illustration and notification sound.
bilingual.yaml: separately authored English and Spanish narration scripts.
Use limits.yaml for bounded previews; a podcast kit resolves four provider calls.
Music is delivered separately, not automatically mixed into the podcast.
The examples do not publish content, build an app or produce a video.
