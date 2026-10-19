# Local live verification

FrameChord is the project name from 0.6.1 onward. Earlier evidence below keeps its original MediaAgent commands, release names and artifact paths. Existing `mediaagent/` projects remain supported in place. See [rename and upgrade guidance](../assets/skill/references/renaming.md).

Executed on macOS on October 14, 2026, using this repository's ignored `.env`. These are selected adapter smoke tests, not certification of every model, account entitlement, or media quality. Each generation smoke run allowed one request and one attempt. Speech used the exact 33-character script `Hello. This is a short local test.` Images used a teal leaf on a cream background; music was instrumental and sound effects used a quiet bell.

Artifact paths below are relative to this checkout's ignored `mediaagent/runs/` directory. Each run retains inputs, request IDs when returned, actual usage when returned, hashes, decoded metadata, limits, and initial quality `unchecked`. The explicit image-review run records an AI assessment separately from generation status. No provider credentials were copied into those records.

| Provider / capability | Explicit model / voice | Successful run | Final artifact / measured result |
| --- | --- | --- | --- |
| OpenAI image | `gpt-image-1`, low quality | `982c1d9c-67d1-42e8-8f7f-19054dc432ae` | `outputs/asset.png`, 1024×1024 |
| Google image | `gemini-nano-banana-2.1` | `911480a4-af84-49c7-9f1a-96d6df4be511` | `outputs/asset.png`, 1376×768; original JPEG retained |
| OpenRouter image | `openai/gpt-image-1` | `0f6b3621-f250-4aa3-9f03-877899f977b3` | `outputs/asset.png`, 1024×1024 |
| OpenAI image editing | `gpt-image-1`, low quality, one explicit leaf reference | `c73e11f5-16ea-444b-b653-d833a59b2d07` | `outputs/asset.png`, 1024×1024; one generation attempt |
| OpenAI speech | `gpt-4o-mini-tts` / `alloy` | `3fb0d43f-4258-4bd2-a1a5-836911117368` | `outputs/asset.wav`, 2.90 s |
| Google speech | `gemini-3.8-flash-tts` / `Kore` | `1801b3c9-34a4-4b6f-af2b-a7e9b55fee1f` | `outputs/asset.wav`, 3.12 s |
| Fish speech | `s2.1-pro-free` / licensed public Calm Narrator Voice | `7cc12bd6-0c9f-44eb-bb90-fe6babbea647` | `outputs/asset.wav`, 2.78 s |
| ElevenLabs speech | `eleven_flash_v2_5` / premade Alice | `dc5cf44f-1fc7-41bf-905f-11d2f064ed4f` | `outputs/asset.wav`, 2.32 s |
| ElevenLabs music | `music_v1`, instrumental, 3 s requested | `a9a6b106-196d-40c6-8812-6c21abe5f42d` | `outputs/asset.mp3`, 3.06 s |
| Google music | `lyria-3-clip-preview`, instrumental, 30 s preset requested | `cdf0978a-74c5-4093-a83c-ca7b8a5edbda` | `outputs/asset.mp3`, 27.01 s |
| ElevenLabs sound effect | `eleven_text_to_sound_v2`, 3 s requested | `70a7728a-9017-4802-bd6a-d180989a23dc` | `outputs/asset.mp3`, 3.03 s |
| ElevenLabs voice preview design | `eleven_ttv_v3`, exact fictional narrator script | `9a4e37f6-64f6-4200-8301-25d21fde6969` | Three local MP3 candidates: 9.48 s, 9.48 s, 9.09 s; no saved provider voice created |
| OpenAI image review | `gpt-4.1-mini`, cached leaf image | `2bc438c5-ae65-4171-a7ff-64e78bb77a85` | One AI review request; provider receipt/usage recorded; no new image generation |
| Google audio review | `gemini-3.8-flash`, verified cached 2.90 s narration | `b9ef5825-3c4c-47dd-8974-7f135c921160` | One AI assessment with returned receipt/usage; no new speech generation; AI quality passed |
| OpenAI authoring plan | `gpt-4.1-mini` | `294e1f4b-9d65-4f59-b499-cae6567392c8` | Validated job and brief in `planning-check/openai/`; no media generated |

The image-edit run initially stopped before submission because a worker startup probe competed for the worker lock. The queued run then completed with one generation attempt. Startup now observes the worker's persisted state without taking its ownership lock, and an offline test covers startup with large references.

The Google image originally failed local publication because the returned bytes were JPEG. The decoder was corrected to identify the actual format, retain the original, and publish a normalized PNG. Resuming recovered the saved response with the same single generation attempt and no additional provider request. An offline regression test covers this behavior.

An earlier Fish request used a placeholder voice binding and was rejected with HTTP 400 (`b246e9aa-d100-4819-832a-b52278359c18`); it is a failed test, not a pass. The smoke script now rejects common placeholder IDs. A separate successful test used an explicitly selected licensed public voice. No voice was created or cloned.

Bounded account voice lookup also succeeded for Google, Fish, and ElevenLabs. Fish's owned-voice list was empty; the explicit licensed public catalog supplied the speech test voice. Voice bindings remain provider/account-specific.

Infographic typography/data accuracy, vocal music, repeated character consistency, podcast synthesis, long narration, Google/OpenRouter planning, saved-voice creation/binding, and recorded-voice cloning were not verified by these tests. Podcast/chunk assembly, cache, dependency, revision, and recovery behavior have offline fixture coverage. Human visual and listening review remains necessary before accepting canonical character references or claiming quality.

The opt-in voice-design smoke test created a temporary fictional character, saved its authoring snapshot inside the ignored run, generated previews with one request/attempt, and removed only its temporary authoring folder. Three candidates remain unaccepted. Saved provider voice creation, canonical binding, and verification refresh have mocked coverage; no provider account voice was created during testing. The image review used a hash-verified cached copy, recorded its returned AI findings/assessment and provider receipt, and did not assess character consistency.

Repeating the exact OpenAI image review reused its completed operation result; the shared budget remained at one request and operation history at one attempt. An explicit metadata lookup of the existing ElevenLabs Alice binding also succeeded (`requires_verification: false`, `is_verified: false` as returned). This verifies account lookup handling, not voice ownership, subjective quality, or new voice creation.


The audio review reused the original OpenAI clip after checking exact effective
inputs and artifact hashes. Its new lineage allowed one request, spent only on
Google's assessment; it returned AI quality `passed` with no findings. That proves
API assessment handling, not an independent human transcription/voice-quality
check. The original speech run is unchanged. Repeating the same review reuses its
operation; no additional assessment call is needed.

Filtered bounded catalogs also passed: Google `Kore`, `en-US`, one result; Fish
licensed public `Calm`, `en`, two one-result pages with continuation page 3;
ElevenLabs `premade`, `en`, two one-result pages with a continuation token. Queries
retained the same filters between pages. Neither filtering nor catalog discovery
created provider voices.

Recorded cloning and Fish inline reference synthesis have no executed live test:
no speaker-selected recording or real permission record was supplied. Offline
fixtures cover denied/missing permission, scope, exact transcripts, binary upload,
idempotent cloning, verification-aware binding, and portable consent dependencies.
`scripts/live-recording.py` provides explicit opt-in bounded tests only when actual
permission and selected local files exist. It never invents permission or accepts
source recordings automatically. Provider reconciliation has mocked GET coverage,
including pending/completed status and recovery without new paid attempts.


## Fictional character candidates

On October 14, 2026, an explicit request to show characters generated two new
fictional candidates through the CLI: Mira, an explorer, and Kito, a small robot.
Run `54372645-39f4-4bc9-b0d6-536fc0101eda` used OpenAI `gpt-image-1`, low quality,
1024×1024, two independent concurrent jobs, a two-request/two-image budget, and
one attempt per job. Both completed with PNG outputs:

- `outputs/demo-mira.png`
- `outputs/demo-kito.png`

Authored persona/visual definitions are in `mediaagent/characters/demo-mira/` and
`mediaagent/characters/demo-kito/`; companion jobs are under
`mediaagent/jobs/character-demo/`. Neither candidate was promoted to a canonical
reference. These are one image per distinct character, so repeated identity
consistency remains unverified. Visual inspection found Mira's head/boots cropped
against the requested full-body framing; this candidate needs a framing revision.
Kito's candidate is visibly framed with the whole character. No voice was generated
or created, and no additional generation was submitted to fix Mira automatically.


## Forest play using both character references

An explicit request to put Mira and Kito together in a forest generated run
`5a5f0d53-4d54-4bbc-9a4a-77e3e2f5aa2a`, using both previous PNGs as references
with OpenAI `gpt-image-1`, medium quality, 1536×1024. One request/attempt completed
and produced `outputs/forest-play.png`. The authored job/brief/limits are under
`mediaagent/jobs/forest-play/`. No canonical character references were changed.

The user rejected this scene for identity drift: Mira's facial geometry and Kito's
body/limb proportions changed. The original scene is now recorded as
`needs_revision`, with the user's actual feedback and agent observations. The job
had supplied plain content references and omitted input fidelity. It does not pass
character consistency acceptance despite successful API/file handling.

## Explicit identity and high-fidelity correction

Run `02a685fb-6592-4534-9d68-2aaf2c75eaf9` used the two original portraits as explicit
identity references, OpenAI `gpt-image-1`, `input_fidelity: high`, high output
quality, and one image request/attempt. It produced
`outputs/forest-play-identity.png`. Agent inspection found Kito's compact design
closer, but Mira's face still changed; this is not an accepted consistency result.
No canonical references were changed, and no low-fidelity result was reused.

A separate explicit `gpt-image-2` comparison completed in run
`888c4087-cff2-4e57-850b-30c50c0c2299`, from the same two original identity images,
with its model-specific fidelity parameter omitted as required by the API. One
request/attempt generated `outputs/forest-play-identity-image2.png`, 1536×1024.
Agent visual inspection found the original Mira facial features and Kito's compact
design substantially closer than both rejected image-1 scenes. It remains a review
candidate: the user has not accepted the likeness. The existing character profiles
and canonical references were not changed. This is not proof of consistency across
additional scenes, expressions, models, or providers.

Regression checks after the correction passed all 86 offline tests, formatting,
and Clippy. One earlier concurrency assertion ran while a live image request held
one shared provider slot and observed only one mock request concurrently; rerunning
after the live request completed passed the expected two-way concurrency assertion.
No paid request was repeated as a result of that offline test failure.


## Read-only provider authentication checks

`mediaagent doctor --live --timeout-seconds 10 --json --non-interactive` completed
with exit 0 using target-root credentials. Exactly five independent read-only GETs
returned HTTP 200 with the expected response shapes: OpenAI `/v1/models`, Google
`/v1beta/models?pageSize=1`, Fish `/wallet/self/api-credit`, OpenRouter `/api/v1/key`,
and ElevenLabs `/v1/user`. All five rows reported authentication `accepted`; no media,
provider voices, or run records were generated. Credential file integrity was checked
before/after. Raw responses, account IDs, key previews, and balances were discarded.
This verifies the listed authentication endpoints at the time of execution, not
all generation endpoints, specific model/voice entitlements, or billing readiness.


## Site showcase batch (0.7.0)

October 15, 2026, installed `framechord 0.7.0`, repository-root credentials. The 27
manifest entries in `docs/site/media/jobs/showcase-batch.yaml` (13 OpenAI
`gpt-image-2.5-flare` images and infographics, 12 Google `gemini-3.8-flash-tts`
narrations in en-US and es-419, a 6-turn Gemini podcast and a 60-second Google
`lyria-3.5` instrumental) passed `batch --dry-run` before execution. They expand to 33
run jobs: the podcast becomes six turn requests plus a local assembly job, for 32
provider requests in total. The first run
`fdb96da4-1243-4ca1-8ec2-215a4433c550` completed 30 jobs; the last podcast turn, the
assembly and the music were blocked before submission because the explicit limits
file allowed 30 requests for 32 provider jobs. After the limit was raised to 36, run
`411fb953-5cfd-45fa-a48b-2938e159a0b7` reused 31 verified cached results and submitted
only the remaining turn and the music; every job succeeded. A first request for a
30-second Lyria song was rejected offline with the full-song minimum message.

All 13 Google `gemini-3.8-flash` audio reviews (`review check --ai`) of the narrations
and assembled podcast passed with no findings. Twelve concurrent `review check`
invocations on artifacts of the same run were rejected before submission with "Wait
for generation to finish before AI review" while one review held the run; run
sequentially, all passed. Agent inspection found every infographic label and number
exact and no stray lettering in the illustrations. `usage` reports a partial list-price
estimate of about $0.37 (image output tokens and the song); Gemini TTS returned only
total tokens and is not estimable. Human listening and acceptance of final quality
remain pending. Per-asset hashes, usage and reviews are in
`docs/site/media/provenance.json`.

## Audiobook sample (October 19, 2026)

One explicitly approved live run of the new `media: audiobook` job,
`docs/site/media/jobs/audiobook.yaml`, built from branch `feature/audiobook` (binary
version still `0.7.0`): Google `gemini-3.8-flash-tts`, voice `Charon`, en-US, a
two-chapter manuscript of 759 narrated characters with spoken chapter titles, MP3 at
64 kbps, `output_mode: full`. The whole-book preflight reported 2 requests and 759
speech characters within the default limits before the run started.

Run `9ad83451-d230-4013-8005-d5e6b68be5fe` succeeded with two provider requests, one
per chapter, and no retries. Gemini returned only total tokens (785 and 854), so
`usage` reports both chapter requests as `not_estimable` and the chapter and book
assembly jobs as `not_estimated`; at the dated rate of $0.00225 per 10 s the audio is
about $0.0135 at list price. Results: chapter WAV durations 28.04 s and 30.48 s, a
1.5 s pause, and a 60.02 s full book (`lamp-counter.mp3`, 480,966 bytes,
SHA-256 `333c280f…5e2c0c`). `ffprobe` reads two ID3 chapters at 0.000–28.040 s and
29.540–60.020 s, matching `lamp-counter.chapters.json`; the cue sheet places track 2
at `00:29:41`. The files are published unchanged under `docs/site/media/audiobook/`.

This verifies the live request, local assembly, MP3 encoding and chapter metadata on
macOS. It does not verify narration quality, pronunciation or that every word was
read: no AI audio review was run and nobody has listened to the result yet.
