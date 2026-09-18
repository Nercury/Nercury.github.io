//! Guards against over-normalization: every meaningful mutation of the real reference snapshot
//! must be reported, and every cosmetic one must not.

use std::path::{Path, PathBuf};

use sitediff::allow::AllowList;
use sitediff::compare::{self, Options, Report, Status};

const SETUP: &str = "rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-00-setup.html";
const WINDOW: &str = "rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-01-window.html";
const OWNERSHIP: &str = "rust/guide/2015/01/19/ownership.html";
const TYPED_ARRAYS: &str = "rust/interesting/2015/12/12/typed-arrays.html";
const TRANSMUTE: &str = "rust/pointers/2014/10/30/rust-mem-transmute-and-void-pointers.html";
const CORTEX: &str = "resources/mcu-02/cortex-m-basics.html";

fn reference() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../archive/reference-site")
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// Replace the first occurrence of `from`; panics if the anchor is gone so tests cannot rot silently.
fn mutate(root: &Path, file: &str, from: &str, to: &str) {
    let path = root.join(file);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains(from), "anchor {from:?} not found in {file}");
    std::fs::write(path, text.replacen(from, to, 1)).unwrap();
}

fn run(candidate: &Path, allow: &str) -> Report {
    let options = Options {
        reference: reference(),
        candidate: candidate.to_path_buf(),
        manifest: None,
        only: None,
        allow: AllowList::parse(allow).unwrap(),
    };
    compare::run(&options).unwrap()
}

fn hunks<'a>(report: &'a Report, file: &str) -> &'a str {
    match &report.files[file] {
        Status::Different { hunks, .. } => hunks,
        other => panic!("{file}: expected Different, got {other:?}"),
    }
}

#[test]
fn reference_equals_itself() {
    let report = run(&reference(), "");
    assert_eq!(report.failures(), 0);
    assert!(report.files.len() >= 30);
    assert!(report.files.values().all(|s| *s == Status::Same));
}

#[test]
fn meaningful_mutations_are_reported() {
    let dir = tempfile::tempdir().unwrap();
    let site = dir.path();
    copy_tree(&reference(), site);

    mutate(site, "index.html", ">Posts<", ">Postz<"); // text
    mutate(site, "about/index.html", "class=\"post-title\"", "class=\"post-titlex\""); // attribute
    mutate(
        site,
        OWNERSHIP,
        "<link rel=\"stylesheet\" href=\"/css/main.css\">\n    <link rel=\"stylesheet\" href=\"/css/pygments/default.css\">",
        "<link rel=\"stylesheet\" href=\"/css/pygments/default.css\">\n    <link rel=\"stylesheet\" href=\"/css/main.css\">",
    ); // element order
    mutate(site, TRANSMUTE, "<span class=\"mi\">333</span>", "<span class=\"mi\">334</span>"); // code text
    mutate(site, SETUP, "</code> and <code", "</code>and <code"); // space between inline elements
    mutate(site, WINDOW, "<h2 id=\"", "<h2 id=\"x-"); // heading id
    mutate(site, "feed.xml", "<title>Hack the Crab at RustFest</title>", "<title>Hack the Crab</title>");
    mutate(site, "css/main.css", "color:#111", "color:#222");
    std::fs::remove_file(site.join(CORTEX)).unwrap();
    std::fs::write(site.join("surprise.html"), "<p>hi</p>").unwrap();

    let report = run(site, "");

    assert!(hunks(&report, "index.html").contains("Postz"));
    assert!(hunks(&report, "about/index.html").contains("post-titlex"));
    assert!(hunks(&report, OWNERSHIP).contains("/css/pygments/default.css"));
    assert!(hunks(&report, TRANSMUTE).contains("334"));
    assert!(hunks(&report, SETUP).contains("\"and \""));
    assert!(hunks(&report, WINDOW).contains("id=\"x-"));
    assert!(hunks(&report, "feed.xml").contains("rss/channel/item[1]/title: \"Hack the Crab\""));
    assert!(hunks(&report, "css/main.css").contains("color:#222"));
    assert_eq!(report.files[CORTEX], Status::Missing);
    assert_eq!(report.files["surprise.html"], Status::Extra);
    assert_eq!(report.failures(), 10);
}

#[test]
fn cosmetic_mutations_are_not_reported() {
    let dir = tempfile::tempdir().unwrap();
    let site = dir.path();
    copy_tree(&reference(), site);

    mutate(site, TYPED_ARRAYS, "</p>\n\n<p>", "</p>\n\n\n      <p>"); // whitespace between blocks
    mutate(site, WINDOW, "<span class=\"k\">", "<span class=\"kd\">"); // highlighter token class
    mutate(site, TRANSMUTE, "<span class=\"mi\">333</span>", "333"); // highlighter token removed
    mutate(site, OWNERSHIP, "<a class=\"site-title\" href=\"/\">", "<a href=\"/\"   class=\"site-title\">"); // attr order
    mutate(site, SETUP, "’", "&rsquo;"); // entity spelling
    mutate(site, "css/main.css", "{margin:0;padding:0}", " {\n  margin: 0;\n  padding: 0;\n}\n"); // css formatting

    let report = run(site, "");
    let failed: Vec<_> = report.files.iter().filter(|(_, s)| s.is_failure()).collect();
    assert!(failed.is_empty(), "{failed:?}");
}

#[test]
fn allowlist_rules_apply_and_unused_rules_are_reported() {
    let dir = tempfile::tempdir().unwrap();
    let site = dir.path();
    copy_tree(&reference(), site);

    // Candidate: analytics removed from every page, feed rebuilt at another time, one extra file.
    let report = run(site, "");
    for file in report.files.keys().filter(|f| f.ends_with(".html")) {
        let text = std::fs::read_to_string(site.join(file)).unwrap();
        let start = text.find("<script>\n      (function(i,s,o,g,r,a,m)").expect("GA block start");
        let end = start + text[start..].find("</script>").unwrap() + "</script>".len();
        std::fs::write(site.join(file), format!("{}{}", &text[..start], &text[end..])).unwrap();
    }
    mutate(site, "feed.xml", "<lastBuildDate>Tue, 11 Aug 2020", "<lastBuildDate>Fri, 18 Sep 2026");
    std::fs::write(site.join(".nojekyll"), "").unwrap();

    let without = run(site, "");
    assert!(hunks(&without, "index.html").contains("GoogleAnalyticsObject"));
    assert!(hunks(&without, "feed.xml").contains("lastBuildDate"));
    assert_eq!(without.files[".nojekyll"], Status::Extra);

    let allow = r#"
        [[drop]]
        path = "**/*.html"
        selector = "script"
        contains = "GoogleAnalyticsObject"
        side = "reference"
        reason = "analytics dropped"

        [[line]]
        path = "feed.xml"
        pattern = '^rss/channel/lastBuildDate:'
        reason = "build time"

        [[file]]
        path = ".nojekyll"
        reason = "pages marker"

        [[line]]
        path = "index.html"
        pattern = 'never matches anything'
        reason = "stale rule"
    "#;
    let with = run(site, allow);
    assert_eq!(with.failures(), 0);
    assert_eq!(with.files["index.html"], Status::Same);
    assert_eq!(with.files["feed.xml"], Status::Allowed { lines: 2 });
    assert_eq!(with.files[".nojekyll"], Status::Ignored);
    assert_eq!(with.unused_rules.len(), 1);
    assert!(with.unused_rules[0].contains("stale rule"));
}

#[test]
fn rules_require_a_reason() {
    assert!(AllowList::parse("[[file]]\npath = \"x\"\nreason = \" \"").is_err());
    assert!(AllowList::parse("[[file]]\npath = \"x\"").is_err());
}

#[test]
fn pruned_static_files_are_checked_against_the_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let (reference, candidate) = (dir.path().join("ref"), dir.path().join("cand"));
    std::fs::create_dir_all(&reference).unwrap();
    std::fs::create_dir_all(candidate.join("images")).unwrap();
    std::fs::write(candidate.join("images/a.png"), "abc").unwrap();
    std::fs::write(candidate.join("images/b.png"), "changed").unwrap();
    let manifest = dir.path().join("ref.manifest");
    // sha256("abc"); b.png and c.png carry the same hash but differ / are absent in the candidate.
    let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    std::fs::write(&manifest, format!("{abc} *images/a.png\n{abc} *images/b.png\n{abc}  images/c.png\n")).unwrap();

    let options = Options { reference, candidate, manifest: Some(manifest), only: None, allow: AllowList::default() };
    let report = compare::run(&options).unwrap();
    assert_eq!(report.files["images/a.png"], Status::Same);
    assert_eq!(report.files["images/b.png"], Status::HashMismatch);
    assert_eq!(report.files["images/c.png"], Status::Missing);
}
