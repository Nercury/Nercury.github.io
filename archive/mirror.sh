#!/usr/bin/env bash
# Mirror a deployed copy of the site for comparison with generator output.
#
#   archive/mirror.sh [BASE_URL] [OUT_DIR]
#
# Fetches (1) every static file tracked in git, (2) the generated entry points, and (3) everything
# reachable by crawling same-site href/src links. Writes:
#   OUT_DIR/                 mirrored tree (directory URLs saved as index.html); static files that
#                            are byte-identical to the working tree are pruned (KEEP_STATIC=1 keeps them)
#   OUT_DIR.manifest         "sha256  path" for every mirrored file, including pruned ones
#   OUT_DIR.report           fetch failures, static drift, crawled URLs with no known source
# Run from the repository root.
set -u

BASE="${1:-https://nercury.github.io}"
OUT="${2:-archive/reference-site}"
REPORT="$OUT.report"
MANIFEST="$OUT.manifest"

rm -rf "$OUT"; mkdir -p "$OUT"; : > "$REPORT"
work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT
touch "$work/done" "$work/static" "$work/queue"

# Source files that are copied verbatim (no front matter, not underscore-prefixed, not tooling).
# Keep in sync with `exclude` in site.toml.
git ls-files \
  | grep -v -E '^(_|\.|archive/|crates/|plans/|Cargo\.|site\.toml$|rust-toolchain\.toml$|sitediff\.allow\.toml$)' \
  | grep -v -E '^(about\.md|index\.html|feed\.xml|css/main\.scss)$' \
  | grep -v -E '\.md$' > "$work/static"

urlencode() { sed -e 's/%/%25/g' -e 's/ /%20/g' -e 's/+/%2B/g' <<<"$1"; }
urldecode() { local s="${1//+/%2B}"; printf '%b' "${s//%/\\x}"; }

# fetch <site-relative path, decoded, no leading slash>
fetch() {
  local path="$1" file code
  grep -qxF -- "$path" "$work/done" && return 0
  echo "$path" >> "$work/done"
  file="$path"
  case "$path" in ""|*/) file="${path}index.html" ;; esac
  mkdir -p "$OUT/$(dirname "$file")"
  code=$(curl -s -o "$OUT/$file" -w '%{http_code}' "$BASE/$(urlencode "$path")")
  if [ "$code" != "200" ]; then
    rm -f "$OUT/$file"
    echo "FETCH $code /$path" >> "$REPORT"
    return 1
  fi
  case "$file" in *.html) extract_links "$file" "$path" ;; esac
}

# Queue same-site links found in an HTML file.
extract_links() {
  local file="$1" page="$2" dir link
  dir="$(dirname "/$page.")"   # directory of the page URL
  grep -o -E '(href|src)="[^"#?]+' "$OUT/$file" | sed -E 's/^(href|src)="//' | while read -r link; do
    case "$link" in
      "$BASE"/*) link="${link#"$BASE"}" ;;
      http://nercury.github.io/*) link="${link#http://nercury.github.io}" ;;
      *://*|//*|mailto:*|javascript:*|data:*) continue ;;
      /*) ;;
      *) link="$dir/$link" ;;
    esac
    link="$(urldecode "${link#/}")"
    # Other project sites served under the same host are not part of this site.
    case "$link" in di-rs/*) continue ;; esac
    # GitHub Pages serves /foo.html for /foo; treat extensionless links as the .html file.
    case "${link##*/}" in ""|*.*) ;; *) link="$link.html" ;; esac
    echo "$link"
  done >> "$work/queue"
}

while IFS= read -r p; do fetch "$p"; done < "$work/static"
for p in "" "about/" "feed.xml" "css/main.css" "resources/mcu-02/cortex-m-basics.html"; do fetch "$p"; done

# Crawl until the queue stops producing new paths.
while :; do
  sort -u "$work/queue" | grep -vxF -f "$work/done" > "$work/next" || true
  : > "$work/queue"
  [ -s "$work/next" ] || break
  while IFS= read -r p; do fetch "$p"; done < "$work/next"
done

# Static drift: mirrored static files must be byte-identical to the working tree.
while IFS= read -r p; do
  [ -f "$OUT/$p" ] || continue
  cmp -s "$p" "$OUT/$p" || echo "DRIFT /$p" >> "$REPORT"
done < "$work/static"

# Crawled files that are neither static sources nor HTML/feed/css output: unexpected.
( cd "$OUT" && find . -type f | sed 's|^\./||' | sort ) > "$work/files"
grep -vxF -f "$work/static" "$work/files" | grep -v -E '(\.html|^feed\.xml|^css/main\.css)$' \
  | sed 's|^|UNEXPECTED /|' >> "$REPORT"

( cd "$OUT" && xargs -d '\n' sha256sum < "$work/files" ) > "$MANIFEST"

# Identical static files carry no information beyond the manifest; do not keep ~76MB of copies.
if [ "${KEEP_STATIC:-0}" != "1" ]; then
  while IFS= read -r p; do
    [ -f "$OUT/$p" ] && cmp -s "$p" "$OUT/$p" && rm -f "$OUT/$p"
  done < "$work/static"
  find "$OUT" -type d -empty -delete
fi

echo "mirrored $(wc -l < "$work/files") files from $BASE into $OUT"
echo "generated (non-static) files:"
grep -vxF -f "$work/static" "$work/files" | sed 's/^/  /'
echo "report ($REPORT):"
if [ -s "$REPORT" ]; then sort "$REPORT" | sed 's/^/  /'; else echo "  clean"; fi
