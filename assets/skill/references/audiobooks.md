# Audiobooks

Use `media: audiobook` when the user wants a book, story or long document read
aloud by one narrator, by chapter or as one full book. It is not a podcast:
podcasts are multi-speaker conversations, and a book must not be adapted into
one unless the user asks.

## Author

1. Put the exact text in a local Markdown file (no URL fetching). Keep the
   user's words; never summarize, translate or tidy them without being asked.
2. Start from `templates/audiobook.yaml` (one manuscript split at headings) or
   `templates/audiobook-chapters.yaml` (explicit chapter files). Both need job
   `schema_version: 2`.
3. Choose the narrator: a `character` with an accepted voice, an explicit
   `selection` and `settings.voice`, or the project's audiobook or speech
   default. One narrator per book; a chapter cannot change the voice.
4. Put delivery direction in `brief` (never spoken). Fish and ElevenLabs saved
   voices do not accept delivery direction; omit the brief for them.

Manuscript rules: chapters start at `#` headings at or above
`split.heading_level`; headings inside code fences, `#tag`, indented and
escaped `#` are ignored; YAML front matter is skipped; text before the first
heading becomes a `preamble` chapter unless `split.preamble: skip`; headings
with no text are skipped (or rejected with `empty_chapters: error`). Chapter
IDs are slugs of the titles. Warnings flag Markdown links, tables, HTML and
emphasis markers that a speech model may read aloud; fix the text only with the
user's agreement.

## Check before paying

```text
framechord validate --kind job framechord/jobs/my-book/job.yaml --json
framechord generate --job framechord/jobs/my-book/job.yaml --output-dir out --dry-run --json
```

`validate` returns a chapter table (ID, title, source lines, characters,
request parts, estimated seconds, first and last 80 characters), skipped
headings, warnings, the narrator and where it came from, totals and a budget
check. Show the user the chapter list and totals. A full book usually exceeds
the default run limits (20 requests, 60,000 speech characters, 1,800 audio
seconds); generation then stops before any request with the minimum limits
needed. Raise `limits` (`config setup --file` or `--limits-file`) only with the
user's approval, or produce a few chapters per run.

## Generate

- One chapter at a time: `generate --job JOB --chapter ID [--chapter ID]
  --output-dir out`. No full-book file is made in a subset run.
- Whole book: `generate --job JOB --output-dir out`. Completed chapter parts
  are reused from cache, so a final full run only assembles.

Outputs go to `out/<book-id>/`: `chapter-NN-<id>.<wav|mp3>` per chapter and,
with `output_mode: full`, `<book-id>.<wav|mp3>`, `<book-id>.chapters.json`
(start time, duration and hash per chapter) and `<book-id>.cue`. MP3 books carry
ID3 chapter markers. With `output_mode: chapters` only chapter files and the
chapter index are written. Audio is 24 kHz mono; WAV stops at about 24.8 hours,
so use MP3 or chapters mode for longer books. This is not an ACX/Audible
delivery specification.

Report the chapter paths, the full-book path and any `missing` chapters from
the result's `audiobook` summary. A failed part blocks only its chapter and the
full book; resume or fix and generate again.

## Revise

`revise --artifact CHAPTER_OR_BOOK_ARTIFACT --chapter ID --instructions-file
notes.md` regenerates that chapter with added delivery direction, reuses the
other chapters from cache and reassembles the book. To change the words, edit
the text and generate again. Listening is the only way to accept narration
quality.
