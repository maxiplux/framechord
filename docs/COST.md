# What FrameChord costs

FrameChord has no subscription, credits, account or hosted service. Every request goes
to the provider you configured and is billed by that provider at its API list price.
The keys stay in your project's `.env`, the records stay in your repository, and nothing
is marked up. This page shows the published rates, prices a typical month three ways,
and says plainly when a subscription tool is the better choice.

All figures were read from the official pricing pages on **2026-10-15** and are stored
with their sources in [`assets/pricing.json`](../assets/pricing.json), the table the
`usage` command uses for labelled estimates. Prices change; the refresh checklist at the
end keeps the table, this page and the README together.

## How you pay

- **Per request, at list price.** An image costs what Google or OpenAI charge for that
  image; narration costs what the speech provider charges for those characters or audio
  tokens. FrameChord submits the request and records the receipt.
- **Volume limits, not a dollar cap.** Budgets (`--limits-file`, defaults in the README)
  bound requests, images, audio seconds, speech characters and revisions per run lineage.
  They stop runaway batches; they do not convert to currency.
- **Nothing is charged twice for identical work.** Matching fingerprints reuse verified
  local artifacts unless you pass `--fresh`.
- **`framechord usage --json`** reports provider-returned units and, beside them,
  list-price estimates computed from the bundled table. Estimates are labelled
  `estimated`, carry the table's verification date, and never replace a cost a provider
  returns. Cached reuse is `cached_no_charge`; anything the table cannot price is
  `not_estimable`, because unknown is not zero.

## Published provider rates

Standard tier, paid API, USD. Google image models have no free API tier; Gemini TTS and
Gemini 3.8 Flash have a free tier whose data may be used to improve Google products.

| Provider / model | What you pay | Notes | Source |
| --- | --- | --- | --- |
| Google `gemini-nano-banana-2.1` (default images and infographics) | $0.0336 per 1K image, $0.0504 per 2K, $0.113 per 4K | | [Gemini API pricing](https://ai.google.dev/gemini-api/docs/pricing) |
| Google `gemini-3.1-flash-image` | $0.045 (0.5K), $0.067 (1K), $0.101 (2K), $0.151 (4K) per image | | same |
| Google `gemini-3.1-flash-lite-image` | $0.0336 per 1K image | Only 1K is listed | same |
| Google `gemini-3-pro-image` | $0.134 per 1K or 2K image, $0.24 per 4K | | same |
| Google `gemini-3.8-flash-tts` (default narration and podcasts) | $9.00 per 1M audio tokens, $0.50 per 1M text tokens | Google states this is "Equivalent to $0.00225 per 10s audio" at 25 tokens per second; from 2027-01-01 the rates double | same |
| Google `gemini-3.8-flash-lite-tts` | $6.00 per 1M audio tokens, $0.50 per 1M text tokens | "Equivalent to $0.0015 per 10s audio"; doubles from 2027-01-01 | same |
| Google `lyria-3.5` (default music) | $0.08 per song | Price is per generation, not per second | same |
| Google `gemini-3.8-flash` (default planning) | $0.75 per 1M input tokens, $3.75 per 1M output | Promotional through 2026-12-31 | same |
| OpenAI `gpt-image-2.5-flare` / `-sunburst` | $8.00 per 1M image input tokens, $30.00 per 1M image output tokens | Per-image cost depends on size and quality; FrameChord estimates from reported output tokens only | [OpenAI API pricing](https://developers.openai.com/api/docs/pricing) |
| OpenAI `gpt-4o-mini-tts` | $12.00 per 1M audio output tokens, $0.60 per 1M text input | | same |
| ElevenLabs `eleven_v3` | $0.08 per 1,000 characters | Pay-as-you-go rate | [ElevenLabs API pricing](https://elevenlabs.io/pricing/api) |
| ElevenLabs `eleven_flash_v2_5` | $0.04 per 1,000 characters | | same |
| ElevenLabs `music_v2_5` | $0.15 per minute | The page FAQ says music is billed per generation; treat the per-minute figure as approximate | same |
| ElevenLabs `eleven_text_to_sound_v2` (sound effects) | $0.12 per minute | Same per-generation caveat | same |
| Fish Audio | Subscription credits, about 600–625 credits per minute of speech | No public per-character API rate, so `usage` reports Fish jobs as not estimable | [Fish Audio plans](https://fish.audio/plan/) |
| OpenRouter | Whatever the routed model charges | OpenRouter returns the cost in its response; `usage` reports it as `provider_reported`, never estimated | [OpenRouter](https://openrouter.ai/docs/quickstart) |

## A typical month, priced three ways

Assume a small product team that, in one month, generates 40 illustrations, narrates
30 minutes of articles, makes 4 music beds and 20 three-second interface sounds.

**At provider rates through FrameChord (Google defaults plus ElevenLabs effects):**

| Asset | Arithmetic | Cost |
| --- | --- | --- |
| 40 illustrations at 1K on Nano Banana 2.1 | 40 × $0.0336 | $1.34 |
| 30 minutes of narration on Gemini 3.8 Flash TTS | 180 ten-second blocks × $0.00225 (text input tokens add a fraction of a cent) | $0.41 |
| 4 music beds on Lyria 3.5 | 4 × $0.08 | $0.32 |
| 20 three-second effects on ElevenLabs | 1 minute × $0.12 | $0.12 |
| **Total** | | **about $2.20** |

The same narration on ElevenLabs costs $0.86 (flash) or $1.73 (v3) for the roughly
21,600 characters that 30 minutes holds at FrameChord's own 12-characters-per-second
reservation ratio; four one-minute beds on ElevenLabs `music_v2_5` cost $0.60. With any
of those ElevenLabs substitutions the month lands between $2 and $4 at list price;
choosing a premium image model changes the picture (the same 40 images on
`gemini-3-pro-image` alone are $5.36). Failed attempts, revisions and AI reviews are
extra requests and are counted by `usage` when they return usage.

**Higgsfield** ([pricing](https://higgsfield.ai/pricing), 2026-10-15): Starter is
$19 per month for roughly 100 Nano Banana Pro image generations and selected models;
Plus is $59 per month ($47 billed annually) for roughly 500 generations and all models.
Starter's allowance covers the 40 images; narration, music beds and interface sounds
are not part of that plan, so the fair comparison is images alone: $1.34 at provider
rates against the $19 monthly fee. Higgsfield is a video-first web product; its page
also states that plan-included unlimited models and free generations "are not
accessible on MCP/CLI", so the plan's best value is reached in its web editor, not from
a terminal or an agent.

**Magnific** ([pricing](https://magnific.com/pricing), 2026-10-15; now the Freepik
suite): Premium is $20 per month ($14.50 billed annually) for 240,000 credits a year,
Premium+ is $45 for 600,000. A Nano Banana 2 image costs 75 credits at 2K and 150 at
4K, a Lyria 3 song costs 160 credits, and three seconds of sound effects cost 15. The
month above uses about 3,940 of the roughly 20,000 credits a Premium month provides
(narration is not priced on the page), so it costs $20, about nine times the provider
rates. Credits are valid for a year with no monthly reset, which softens, but does not
remove, the fixed monthly fee.

**Where a plan catches up.** At full use, Magnific Premium's 20,000 monthly credits buy
about 266 Nano Banana 2 images at 2K, roughly $0.075 each. That is cheaper than Google's
list price for that older model (`gemini-3.1-flash-image`, $0.101 at 2K) and more than
the current Nano Banana 2.1 at 2K ($0.0504). Higgsfield Starter's $19 equals about 565 Nano Banana 2.1 images at 1K
through the API. If you generate several hundred images every month through the same
model, price the plan against the table above; below that, pay-per-use wins by a wide
margin.

## When a subscription tool is the better choice

Be honest about the fit. Choose Higgsfield, Magnific or a similar product when you need:

- **Video.** FrameChord does not generate video and advertises no video adapter.
- **Upscaling and enhancement.** Magnific's upscalers are its headline feature;
  FrameChord has none.
- **Very high, steady volume** through one model, where a plan's bundled price per asset
  drops below the list price.
- **A visual editor, gallery or team workspace** rather than files in a repository.
- **No API keys.** FrameChord needs a key for every provider you use.

## The "90 percent" claim, scoped

FrameChord is the cheaper option for the assets a software project usually needs:
article and documentation illustrations, infographics with exact copy, narration and
podcast audio, music beds and interface sounds, generated inside the repository by the
same agent that writes the code. For those, a few dollars a month at list price replaces
a fixed plan, and the inputs, receipts and outputs stay versioned next to the code. It
is not the cheaper option for video, upscaling or studio-scale image volume.

## Audiobooks

An audiobook is priced like any long narration: each chapter is split into requests
of at most 4,000 characters (less on models with a lower limit), and each request is
priced on its own row. As a worked example from the rates above (read 2026-10-15): an
80,000-word novel is roughly 450,000 characters, or 8 to 10 hours of narration. On the
default `gemini-3.8-flash-tts` at "$0.00225 per 10s audio", that is about $6.75 to $8.45
of audio output plus a few cents of text input; on ElevenLabs `eleven_flash_v2_5` at
$0.04 per 1,000 characters, about $18. The actual duration depends on the voice and
pace, Google's rates double from 2027-01-01, and `usage` reports the real figures
afterwards. A whole book is far beyond the default run limits (20 requests, 60,000
speech characters), so FrameChord stops before the first request and names the limits
to raise; produce a chapter first with `--chapter` to check the voice before paying for
the rest.

## How `usage` estimates work

- `cost_status` per job is `estimated`, `provider_reported`, `cached_no_charge` or
  `not_estimable`; review and voice operations, and the podcast, audiobook chapter and
  audiobook book jobs that only assemble clips, are `not_estimated`.
- Images are priced per artifact by resolution tier from the recorded width and height
  (0.5K up to 0.6 megapixels, 1K up to 2, 2K up to 8, 4K above). OpenAI images are
  priced from reported output tokens and marked `partial` because input tokens use
  different rates; when the recorded usage carries `input_tokens_details` and the table
  has both `text_token_in` and `image_token_in` entries for the model, the split input
  tokens are priced too and `partial` clears. The bundled table has no OpenAI image
  text-input rate yet, so bundled estimates stay output-only.
- Speech is priced per character for ElevenLabs and per reported token for Google and
  OpenAI. Podcast turns are separate jobs and are priced individually; the parent job
  that assembles the clips locally is `not_estimated`. Music and effects use per-song
  prices or decoded durations; a recorded duration that is not a finite, non-negative
  number makes the row `not_estimable` instead of an estimate without an amount.
- `estimated_total_usd` sums only `estimated` rows; `currency_total` stays `null`
  because providers do not report a uniform cost.
- `--price-table FILE` supplies your own dated JSON or YAML table, validated before
  use, for negotiated rates or a later price list.

## Refresh checklist

1. Open every `source` URL in `assets/pricing.json` and update `usd`, `per`,
   `qualifier`, `valid_until` and `note` to match the page; bump `verified_at` (dates
   must be real calendar days in `YYYY-MM-DD` form, or validation refuses the table).
2. Run `cargo test --locked --test pricing` (the table must validate and name only
   current, supported catalog models).
3. Update the rate table, the worked month and the competitor paragraphs above, and the
   cost bullets in the README, with the new date.
4. Re-check the Higgsfield and Magnific pages; their plan names and allowances change.
5. Run `python3 scripts/build-site.py` so the web guide's generated regions match, and
   record the refresh in `docs/STATUS.md`.
