# Migration: Jekyll → custom Rust generator

Status legend: `[ ]` todo, `[~]` in progress, `[x]` done. Update this file as phases complete;
record surprises under each phase's **Notes**.

## Goal

Replace the Jekyll/github-pages build with a pure-Rust generator living in this repo, producing
the same HTML as the currently deployed site, verified by a purpose-built diff tool, deployed via
GitHub Actions → Pages.

## Ground truth

- Live site `https://nercury.github.io` was built **2020-08-11 20:34 UTC by Jekyll 3.9.0**
  (github-pages ≈ 207, kramdown 2.x, rouge 3.x), 5 minutes after HEAD `211770a`. The live site is
  the reference. `Gemfile.lock` (github-pages 202 / Jekyll 3.8.5 / kramdown 1.17) is **not** what
  produced it.
- Content: 25 posts, 9 drafts, `about.md`, `resources/mcu-02/cortex-m-basics.md` (a page with an extensionless permalink), `index.html`, `feed.xml` (hand-written RSS), no sitemap.
- Front matter keys in use: `layout`, `title`, `date`, `categories`, `permalink`.
- Post URL scheme: `/:categories/:year/:month/:day/:title.html`; date from front matter, not the
  filename (hack-the-crab: filename 11-09, URL 11-11). Category `C++` URL form: verify in Phase 1.
- Liquid in content: only `{% highlight rust|console|text %}` (67 blocks). No footnotes, IALs, TOC.
  One post with a table, two with raw HTML.
- Code block output shapes on the live site:
  - fenced, language known to Rouge: `div.language-X.highlighter-rouge > div.highlight > pre.highlight > code` + token spans
  - fenced, unknown language (e.g. `txt`): `<pre><code class="language-txt">`
  - `{% highlight X %}`: `<figure class="highlight"><pre><code class="language-X" data-lang="X">` + token spans
  - inline: `<code class="language-plaintext highlighter-rouge">`
  - highlight.js additionally runs client-side over all `pre code`.

## Decisions

| # | Decision | Rationale / risk |
|---|----------|------------------|
| D1 | Custom Rust binary: `pulldown-cmark` + `liquid` + `grass` | Full control of emitted HTML. Risk: we own the code; kramdown≠CommonMark divergences. |
| D2 | Reference = mirror of live site, committed under `archive/reference-site/` | Zero toolchain; it is what readers see. Cannot be regenerated — acceptable, it is a frozen snapshot. |
| D3 | Docker Jekyll is a **fallback**, used only if Phase 1 finds source↔site drift (or to render drafts if wanted). Pin `github-pages` 207 (Jekyll 3.9.0) in a separate `archive/Gemfile`; keep the original lockfile archived untouched. No Ruby on the host, ever. | Using the 202 lockfile would itself introduce drift. Risk: exact 2020 rouge patch version unknown. |
| D4 | Highlighting in two passes: parity first (same wrappers, plain escaped code, keep highlight.js), then Phase 8 moves to pure server-side (`syntect`) and removes hljs/jQuery. | Rouge token spans cannot be reproduced exactly in Rust. Risk: slight color shift in pass 1 (currently pygments CSS + hljs CSS merge). |
| D5 | Keep Liquid templates via the `liquid` crate | Layouts/includes port nearly verbatim. Risk: filter/whitespace differences vs Ruby Liquid. |
| D6 | Input changes minimal: front matter and filenames untouched. Divergence resolution order: generator fix → minimal input edit → allowlist entry. | |
| D7 | Drop Google Analytics | Phase 1 showed the snippet **is** emitted on every live page; its removal is an allowlisted diff. |
| D8 | Deploy with GitHub Actions (`upload-pages-artifact` + `deploy-pages`); no `gh-pages` branch | Requires manual switch of Pages source in repo settings. |
| D9 | "Nearly the same" is defined by an explicit allowlist file, never fuzzy thresholds | Auditable. |
| D10 | Rust nightly, edition 2024, **no feature gates**: only features already stabilized but not yet in beta/stable. Enforced by `unstable_features = "forbid"` workspace lint. Interesting uses logged in `plans/DEV_BRIEF.md`. Floating `nightly` locally, pinned dated nightly before Phase 7. | Low risk: code is at most ~12 weeks ahead of stable. Lint does not cover dependencies. |

## Target layout

```
Cargo.toml                 workspace
crates/blog/               generator            → `cargo run -p blog -- build [--drafts]`
crates/sitediff/           comparison tool      → `cargo run -p sitediff -- <reference> <candidate>`
site.toml                  site config (replaces _config.yml)
sitediff.allow.toml        tolerated differences, each with a reason
_posts/ _drafts/ _layouts/ _includes/ _sass/ css/ js/ images/ ...   (unchanged for now)
archive/                   Gemfile, Gemfile.lock, _config.yml, reference-site/, (Dockerfile if D3 triggers)
plans/MIGRATION.md
_site/                     build output (gitignored)
```

---

## Phase 0 — Archive and layout

- [x] Create cargo workspace with empty `blog` and `sitediff` crates.
- [x] `git mv` `Gemfile`, `Gemfile.lock`, `_config.yml` → `archive/`.
- [x] Add `site.toml` carrying the values from `_config.yml` (title, email, description, baseurl, url, usernames).
- [x] `.gitignore`: add `target/`; keep `_site`.
- [x] Delete `images/.DS_Store` from the repo.

**Exit:** `cargo build` succeeds; repo tree matches target layout.
**Risk:** once `_config.yml` moves, a push to master makes GitHub's Jekyll build with defaults.
Mitigation (owner decision): work happens on local master, **nothing is pushed until the Rust
version is serviceable** (Phase 7). Consequence: no off-machine backup and no CI until then; the
Phase 7 PR gate only becomes useful after the first push.

**Notes:**

## Phase 1 — Reference snapshot

- [x] Script (`archive/mirror.sh` or a `sitediff mirror` subcommand) that fetches: every URL derived
      from source (posts, pages, `index.html`, `feed.xml`, rendered `resources/**/*.md`) **and** all
      static assets from `git ls-files`, plus a link crawl to catch anything unexpected.
- [x] Commit result to `archive/reference-site/`.
- [x] Drift check: static assets byte-identical to the working tree; every expected URL returned 200;
      no crawled URL lacks a source file.
- [x] Record facts needed later: `C++` category URL form, `site.pages` order (nav loop), whether GA
      snippet is present, how `cortex-m-basics` and `css/main.css` are emitted.

**Exit:** URL inventory written into Notes; drift verdict recorded.
**If drift is found:** trigger D3 — add `archive/Dockerfile` + `archive/Gemfile` (github-pages 207),
build the source in Docker, and use that output as reference for the drifting pages only.

**Notes:**

- Done 2026-09-18 with `archive/mirror.sh` (curl; fetches tracked static files + entry points + link crawl).
- **Drift verdict: none.** All 178 static files are byte-identical to the working tree; all 25 posts,
  `index.html`, `about/`, `feed.xml`, `css/main.css`, `resources/mcu-02/cortex-m-basics.html` fetched
  with 200. D3 (Docker Jekyll) is **not** triggered.
- Snapshot is pruned: identical static files are deleted after the check (they were 76MB of
  duplicates). Committed: 30 generated files (2.0MB) + `reference-site.manifest` (sha256 of all 208
  mirrored files) + `reference-site.report`. Consequence for Phase 2: `sitediff` compares static
  files against the manifest hashes, not against files in the reference tree.
- `.gitattributes` marks the snapshot `-text` so line endings are never converted.
- Facts:
  - `C++` category → URL segment `c++` (lowercased, `+` kept): `/c++/interesting/2016/02/22/weak_ptr-and-event-cleanup.html`.
  - **GA snippet IS present on all 28 live pages** — the D7 assumption was wrong (Ruby Liquid
    evaluated the parenthesized condition as truthy). Dropping GA is therefore a real diff → first
    `sitediff.allow.toml` entry.
  - Nav loop: only one page has a title (About Me); the rest of `site.pages` emits whitespace only,
    so page order is irrelevant after whitespace normalization.
  - `cortex-m-basics.md` renders with the `page` layout and an **empty**
    `<h1 class="post-title"></h1>`; `<title>` falls back to the site title. (It has front matter
    without a title — see the correction in Phase 3 notes.)
  - `css/main.css` is compressed single-line output (Sass `compressed` style).
  - `/di-rs/*` on the same host belongs to another repo's project site; excluded from the crawl.
  - An extensionless link to `/resources/mcu-02/cortex-m-basics` exists (Pages serves the `.html`); crawl maps it to the `.html` file.
- Pre-existing broken links on the live site (content bugs, reproduced faithfully for parity; fix in Phase 9):
  - `/rust/opengl/tutorial/2018/02/11/docs.gl` (link written without scheme)
  - `/rust/pointers/2014/10/30/Nercury/ccc99ec11f49043c14ed` (gist link written without host)

## Phase 2 — `sitediff`

- [x] File-set diff (missing / extra paths).
- [x] HTML: parse with `html5ever`; normalize inter-element whitespace, attribute order, entity
      forms; **code blocks compared by text content only** (token spans stripped) while wrapper
      element/class structure is still compared.
- [x] `feed.xml`: structural compare; ignore `pubDate`/`lastBuildDate`/`generator`; item
      `description` is unescaped and compared as normalized HTML.
- [x] CSS: byte compare, falling back to normalized compare (whitespace, trailing semicolons, color case).
- [x] Other assets: compare candidate file hashes against `archive/reference-site.manifest` (static files are pruned from the snapshot).
- [x] `sitediff.allow.toml`: entries = path glob + selector/pattern + mandatory `reason`.
      Report unused allowlist entries.
- [x] Output: per-file summary, readable unified diff of normalized DOM, non-zero exit on differences.

**Exit:** reference vs itself → 0 diffs; reference vs a hand-mutated copy → every mutation reported.
**Risk:** over-normalization hides real differences. Mitigation: mutation test set covers text
change, attribute change, element reorder, missing file, changed code text.

**Notes:**

- Done 2026-09-18. Crate `crates/sitediff` (lib + bin): `scraper`/html5ever for HTML, `roxmltree`
  for the feed, `similar` for line diffs, `globset` + `regex` + `toml` for the allowlist.
- Approach: every file type is converted to a **canonical line form** (one DOM node per line,
  indented), then line-diffed. What is normalized away is documented at the top of `src/html.rs`.
- Usage: `cargo run -p sitediff -- archive/reference-site _site` (manifest and
  `sitediff.allow.toml` are picked up by default). Flags: `--only <glob>` to iterate on one post,
  `--limit <n>` files with hunks (default 5), `-q` summary only, `-v` list identical files,
  `--no-manifest`. Exit code 1 on any unallowed difference.
- Hunk markers: `-` reference, `+` candidate, `~` after the sign = line covered by the allowlist.
- Allowlist has three rule kinds — `[[file]]`, `[[drop]]` (CSS selector + optional `contains`,
  applied before comparison, per side), `[[line]]` (regex on normalized lines). `reason` is
  mandatory; unused rules are reported as warnings (suppressed under `--only`).
- Feed lines carry their full path (`rss/channel/item[3]/pubDate: ...`) so rules can target the
  channel build date without also excusing item dates. `<description>` is unescaped and compared as HTML.
- Static files pruned from the snapshot are verified by sha256 against `reference-site.manifest`.
- `sitediff.allow.toml` seeded with: GA script drop (reference side), feed build dates, feed generator.
- Exit criteria met via `cargo test -p sitediff` (`tests/mutations.rs`, runs against the real snapshot):
  reference vs itself = all same; 10 meaningful mutations all reported (text, attribute, element
  order, code text, space between inline elements, heading id, feed title, CSS value, missing file,
  extra file); 6 cosmetic mutations not reported (block whitespace, token span class, token span
  removed, attribute order, entity spelling, CSS formatting).
- Known blind spots (accepted): leading/trailing space *inside* an inline element
  (`<a> x</a>`), class token order, highlighter `<span>`s inside `<pre>`; CSS normalizer lowercases
  anything hex-like after `#` (would also lowercase an id selector such as `#FACE`).

## Phase 3 — Generator skeleton

- [x] Load `site.toml`; discover posts, drafts, pages, static files (Jekyll's exclusion rules:
      `_*`, `.*`, plus `archive/`, `crates/`, `plans/`, `target/`, cargo files).
- [x] YAML front matter parsing; files without front matter are static, except `.md` (optional-front-matter behavior).
- [x] URL computation: categories, front matter date, `permalink`, `C++` form per Phase 1 notes.
- [x] Static copy; SCSS via `grass` (compressed/expanded style matching the reference).
- [x] Emit placeholder HTML at the right paths.

**Exit:** `sitediff` reports zero file-set differences and CSS passes.
**Risk:** `grass` vs Ruby Sass formatting (`lighten()`/`darken()` rounding, number precision).

**Notes:**

- Done 2026-09-18. `crates/blog`: `config.rs` (site.toml), `front_matter.rs`, `site.rs` (discovery,
  classification, URLs), `build.rs` (static copy, SCSS, placeholder documents), `main.rs`
  (`blog build [--out _site] [--drafts]`, `blog urls [--drafts]`).
- Exit criteria met: `sitediff archive/reference-site _site` → 208 files, **0 missing, 0 extra**,
  178 static files hash-identical, `css/main.css` passes. The 29 remaining DIFFERENT are the
  placeholder HTML pages and the feed (Phases 4-6).
- Discovery walks the filesystem like Jekyll (not `git ls-files`): skips names starting with
  `_`, `.`, `#` or ending in `~`, plus `exclude` from `site.toml`. Untracked junk in the working
  tree would therefore be published; sitediff reports it as EXTRA.
- **Correction to Phase 1:** `cortex-m-basics.md` *does* have front matter (`layout: page`,
  `permalink: /resources/mcu-02/cortex-m-basics`, no title). It is not a default-layout case. The
  extensionless permalink is written to `<permalink>.html`; the URL itself stays extensionless.
  Markdown files without front matter are still treated as pages (optional-front-matter), but no
  current file needs that.
- Post URL rule verified against all 25 reference paths: categories lowercased, de-duplicated,
  not slugified; date from front matter, falling back to the file name.
- Drafts: `build --drafts` renders all 9 (39 documents). Draft URLs use the front matter date.
- YAML via `serde_yaml_ng` (maintained fork of the archived `serde_yaml`).
- CSS: `grass` (compressed) vs Ruby Sass differ only in notation. Handled in sitediff's CSS
  normalizer: BOM, `0.2em`/`.2em`, `"C"`/`"“"`, selector order within a list (`@extend`
  output order differs). One allowlist entry: `color:black` → `color:#000`. grass output starts
  with a UTF-8 BOM because it emits the literal `“`; harmless, kept.
- safety: `build` refuses to clear an output directory containing `.git`, `site.toml` or `Cargo.toml`.
- Expected dead-code warnings in `blog` (config fields, `layout`, `tags`) until Phase 4 uses them.

## Phase 4 — Templates

- [x] `liquid` crate with partials from `_includes/`, nested layouts from `_layouts/`.
- [x] Objects: `site` (config, `posts` newest-first, `pages` in reference order, `time`), `page`, `post`, `content`, `jekyll.version` (allowlisted in feed anyway).
- [x] Filters as needed: `date` (`%-d` support), `date_to_rfc822`, `xml_escape`, `prepend`, `replace`.
- [x] Remove the GA block from `_includes/head.html` (D7).

**Exit:** `index.html`, `/about/`, and page chrome of all posts diff clean (article body may still differ).
**Risk:** the `{% if (site.GH_ENV == "gh_pages") %}` parenthesized condition is invalid Liquid that
Ruby tolerates — moot once removed.

**Notes:**

- Done 2026-09-18. New modules in `crates/blog`: `templates.rs` (partials from `_includes`, layout
  chain from `_layouts`, `site`/`page` objects), `filters.rs` (`xml_escape`, `date_to_rfc822`),
  `markdown.rs` (plain CommonMark stub, replaced in Phase 5).
- Exit criteria met: `index.html` **same**; `about/` differs by one smart apostrophe (Phase 5
  typography); a chrome-only run (temporary `[[drop]] article.post-content` on both sides) shows
  all 28 HTML pages identical outside the article, except `…12-buffers.html`, where stray `<b>`
  from the unported Markdown body swallows the footer in the parsed tree (body issue, Phase 5).
  The chrome-only trick is worth reusing: copy the allowlist, add that drop rule, pass `--allow`.
- `liquid` 0.26 with `liquid-lib`'s `jekyll` feature for the unquoted `{% include head.html %}`.
  Layouts and includes are used **unchanged**, apart from the two edits below. The stdlib `date`
  filter handles `%b %-d, %Y` correctly.
- Template edits: GA block removed from `_includes/head.html` (D7); `feed.xml` generator string
  no longer claims Jekyll (already allowlisted).
- **Strictness difference:** Ruby Liquid renders a missing key as empty; this crate errors on
  `{{ page.missing }}`. `page.title`, `date`, `layout`, `permalink` are always defined (nil when
  absent). `{% if page.author %}` on a missing key is fine. Any *new* key output without an `if`
  guard must be present in every document's front matter.
- **Deviation from Jekyll:** Liquid is not run over Markdown sources. No source contains `{{ }}`,
  and the only tag used, `{% highlight %}`, is handled by the Markdown renderer in Phase 5. Rust
  code with `{{` in future posts is therefore safe without `{% raw %}`.
- All other front matter keys pass through to templates as `page.<key>` (YAML → Liquid values).
- Dates: front matter times are treated as UTC (as on the old GitHub build); `site.time` is build time.

## Phase 5 — Markdown parity (highest risk)

- [ ] kramdown-compatible heading id generation (incl. duplicates `-1`, `-2`, stripping rules).
- [ ] Smart typography matching kramdown (quotes, dashes, ellipsis) — compare against pulldown-cmark's smart punctuation, patch where they differ.
- [ ] Code block wrappers exactly as listed under Ground truth, including the known/unknown-to-Rouge
      language split (hardcode the set of languages observed in the reference).
- [ ] `{% highlight %}` pre-pass → `figure.highlight` output.
- [ ] Inline code classes; tables; raw HTML passthrough; list/paragraph nesting (`about.md`).
- [ ] Iterate post by post, oldest → newest; log every divergence and its resolution (D6 order) in Notes.

**Exit:** all 25 posts + pages diff clean modulo allowlist; every allowlist entry has a reason.
**Failure mode:** a long tail of kramdown quirks. Stop rule: if a quirk needs >~50 lines of
generator code to serve one occurrence, edit the input instead.

**Notes:**

## Phase 6 — Feed and drafts

- [ ] `feed.xml` via the same Liquid pipeline (`layout: null`), last 10 posts.
- [ ] `--drafts` flag; all 9 drafts render without error (no reference to diff against unless D3 was triggered).

**Exit:** feed diffs clean; `build --drafts` succeeds.

**Notes:**

## Phase 7 — Deployment

- [ ] `.github/workflows/deploy.yml`: checkout → pinned nightly toolchain (from `rust-toolchain.toml`) + cache → `cargo run --release -p blog -- build` → `upload-pages-artifact` → `deploy-pages` (on push to master).
- [ ] PR/branch job: build + `sitediff archive/reference-site _site` as a gate.
- [ ] **Manual (owner):** Settings → Pages → Source = GitHub Actions.
- [ ] First push of master (workflow included); after deploy, run the mirror script against live and `sitediff` it against `_site`.
- [ ] Close/delete stale dependabot bundler branches.

**Exit:** live site served from the Actions artifact, diff clean.
**Risk:** switching Pages source before the workflow is on master gives a window with no deploy;
and the first push also triggers GitHub's legacy Jekyll build against a repo without `_config.yml`.
Order: switch Pages source to GitHub Actions **first** (legacy build stops, old site keeps being
served), then push, then confirm the workflow deployed.

**Notes:**

## Phase 8 — Pure server-side highlighting

- [ ] `syntect` with CSS classes (no inline styles); generate a theme stylesheet.
- [ ] Unify the three code block shapes into one; convert `{% highlight %}` blocks to fences (mechanical).
- [ ] Server-side heading anchors replacing the jQuery script.
- [ ] Remove `js/highlight.pack.js`, `js/jquery.min.js`, `css/styles/*`, `css/pygments/`, netdna font-awesome link (keep or inline the one icon used).
- [ ] `sitediff` remains the content guard: code text and all non-code DOM must stay identical; wrapper/head changes go into a new, reviewed allowlist section.
- [ ] Visual check in a browser of one post per language.

**Exit:** no client-side JS required for rendering; content diff clean.
**Risk:** syntect's bundled grammars lack/underperform for GLSL, TOML, gdb → plain text fallback, or
add `.sublime-syntax` files.

**Notes:**

## Phase 9 — Cleanup

- [ ] `site.url` → `https`.
- [ ] Fix the two broken links recorded in Phase 1 notes (then allowlist or re-baseline).
- [ ] Remove dead assets; decide whether `archive/reference-site/` stays in the repo (it is the CI gate — if kept, freeze the allowlist; if removed, drop the gate).
- [ ] README: how to write a post, build, preview, run sitediff.
- [ ] Optional: `blog serve` with file watching.

**Exit:** README accurate; no Jekyll artifacts outside `archive/`.

**Notes:**
