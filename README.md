# nercury.github.io

Source of [nercury.github.io](https://nercury.github.io), built by a small Rust static site generator
that lives in this repository. No Ruby, no Node, no JavaScript on the site.

Requires Rust nightly (`rust-toolchain.toml` selects it). Conventions: [plans/DEV_BRIEF.md](plans/DEV_BRIEF.md).
How this came to be (migrated from Jekyll in 2026): [plans/MIGRATION.md](plans/MIGRATION.md).

## Writing

    cargo run -p blog -- serve --drafts     # http://127.0.0.1:4000, rebuilds on save (refresh manually)

- A post is `_posts/YYYY-MM-DD-slug.markdown` with front matter:

      ---
      layout: post
      title:  "Title"
      date:   2026-09-18
      categories: rust opengl
      ---

  Its URL is `/<categories>/<yyyy>/<mm>/<dd>/<slug>.html`. The date in the **front matter** wins over
  the one in the file name. Changing either categories or date changes the URL.
- Work in progress goes to `_drafts/` (same format); only built with `--drafts`, never deployed.
- Markdown is CommonMark + tables, strikethrough and smart punctuation. Things to know:
  - Code goes in fenced blocks with a language: `rust`, `toml`, `cpp`, `c`, `bash`, `glsl`, `cs`,
    `haskell`, `python`, ... (anything [bat](https://github.com/sharkdp/bat) knows). `txt`, `text`,
    `console` and unknown names render as plain text. Highlighting happens at build time.
  - **Generics in prose need backticks**: a bare `Vec<T>` or `Buffer<B>` is passed through as an
    HTML tag and can swallow or restyle the rest of the page.
  - Liquid (`{{ }}`, `{% %}`) is *not* processed in Markdown files, so Rust's `{{` needs no escaping.
  - Heading ids come from the heading text (lowercase, punctuation dropped, spaces → `-`);
    rewording a heading breaks deep links to it.
- Pages (`about.md`, `index.html`, `feed.xml`) and everything in `_layouts/` and `_includes/` are
  Liquid templates. Unlike Jekyll, printing a front matter key that a page does not define is a build
  error; guard optional keys with `{% if page.key %}`.
- Every other file is copied as-is, except names starting with `_`, `.`, `#`, ending in `~`, and the
  paths listed under `exclude` in `site.toml`. Note that this includes untracked files.
- Site settings and the code color theme (`syntax_theme`) are in `site.toml`.

## Commands

    cargo run -p blog -- build [--drafts] [--out _site]
    cargo run -p blog -- serve [--drafts] [--port 4000]
    cargo run -p blog -- urls [--drafts]        # every output URL and its source file
    cargo test --workspace

## Deployment

Pushing to `master` runs `.github/workflows/deploy.yml`: tests, site build, a drafts smoke build, then
GitHub Pages deployment. CI uses a pinned nightly (`RUSTUP_TOOLCHAIN` in the workflow).

## Checking for regressions: sitediff

`archive/reference-site/` is a snapshot of the site as Jekyll built it in 2020. `sitediff` compares a
build against it structurally (normalized DOM, code compared as text, static files by hash):

    cargo run -p blog -- build
    cargo run -p sitediff -- archive/reference-site _site

It is a manual tool, not a CI gate. Run it after changing `crates/blog` or updating dependencies
(`cargo update`): expected output is only content you added or edited on purpose, reported as `EXTRA` /
`DIFFERENT`. Accepted differences live in `sitediff.allow.toml`, each with a reason.
`archive/mirror.sh [url] [dir]` snapshots a deployed site for the same kind of comparison.
