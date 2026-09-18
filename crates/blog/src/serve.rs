//! `blog serve`: local preview. Builds the site, serves it, and rebuilds when a source changes.
//!
//! Deliberately simple: sources are polled for modification times (no file-watcher dependency)
//! and the browser is not reloaded automatically.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Result, anyhow};
use tiny_http::{Header, Response, Server, StatusCode};

use crate::build;
use crate::site::Site;

pub fn serve(root: &Path, out: &Path, drafts: bool, port: u16) -> Result<()> {
    rebuild(root, out, drafts);

    let server = Server::http(("127.0.0.1", port)).map_err(|e| anyhow!("listening on port {port}: {e}"))?;
    println!("serving http://127.0.0.1:{port}/ (drafts: {drafts}); Ctrl+C to stop");

    let watcher = {
        let (root, out) = (root.to_path_buf(), out.to_path_buf());
        move || {
            let mut last = fingerprint(&root, &out);
            loop {
                std::thread::sleep(Duration::from_millis(500));
                let current = fingerprint(&root, &out);
                if current != last {
                    last = current;
                    rebuild(&root, &out, drafts);
                }
            }
        }
    };
    std::thread::spawn(watcher);

    for request in server.incoming_requests() {
        let response = match resolve(out, request.url()) {
            Some(file) => match std::fs::read(&file) {
                Ok(bytes) => Response::from_data(bytes).with_header(content_type(&file)),
                Err(_) => not_found(),
            },
            None => not_found(),
        };
        // A client that went away is not worth reporting.
        let _ = request.respond(response);
    }
    Ok(())
}

/// A failed build must not take the server down: report it and keep serving the last output.
fn rebuild(root: &Path, out: &Path, drafts: bool) {
    let result = Site::load(root, drafts).and_then(|site| build::build(&site, out));
    match result {
        Ok(summary) => println!("built {} documents, {} static files", summary.documents, summary.statics),
        Err(error) => eprintln!("build failed: {error:#}"),
    }
}

/// Maps a request URL to a file the way GitHub Pages does: directories serve `index.html`,
/// extensionless paths fall back to `<path>.html`.
fn resolve(out: &Path, url: &str) -> Option<PathBuf> {
    let path = url.split(['?', '#']).next().unwrap_or("");
    let path = percent_decode(path)?;
    if path.split('/').any(|segment| segment == "..") {
        return None;
    }
    let relative = path.trim_start_matches('/');
    let candidate = out.join(relative);
    if candidate.is_dir() {
        return Some(candidate.join("index.html")).filter(|p| p.is_file());
    }
    if candidate.is_file() {
        return Some(candidate);
    }
    Some(out.join(format!("{relative}.html"))).filter(|p| p.is_file())
}

fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = text.get(i + 1..i + 3)?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn content_type(file: &Path) -> Header {
    let mime = match file.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "pdf" => "application/pdf",
        "txt" | "md" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };
    Header::from_bytes("Content-Type", mime).expect("static header is valid")
}

fn not_found() -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string("404 not found").with_status_code(StatusCode(404))
}

/// Number of source files and the newest modification time among them.
fn fingerprint(root: &Path, out: &Path) -> (usize, Option<SystemTime>) {
    fn visit(dir: &Path, out: &Path, state: &mut (usize, Option<SystemTime>)) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path == out || name == "target" || name == ".git" || name == ".idea" {
                continue;
            }
            if path.is_dir() {
                visit(&path, out, state);
            } else if let Ok(modified) = entry.metadata().and_then(|m| m.modified()) {
                state.0 += 1;
                state.1 = state.1.max(Some(modified));
            }
        }
    }
    let mut state = (0, None);
    visit(root, out, &mut state);
    state
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;

    #[test]
    fn urls_resolve_like_github_pages() -> Result<()> {
        let dir = std::env::temp_dir().join(format!("blog-serve-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("c++/about")).context("creating test dir")?;
        std::fs::write(dir.join("index.html"), "")?;
        std::fs::write(dir.join("c++/about/index.html"), "")?;
        std::fs::write(dir.join("c++/basics.html"), "")?;
        std::fs::write(dir.join("c++/a b.png"), "")?;

        assert_eq!(resolve(&dir, "/"), Some(dir.join("index.html")));
        assert_eq!(resolve(&dir, "/c%2B%2B/about/?x=1"), Some(dir.join("c++/about").join("index.html")));
        assert_eq!(resolve(&dir, "/c++/basics"), Some(dir.join("c++/basics.html")));
        assert_eq!(resolve(&dir, "/c++/a%20b.png"), Some(dir.join("c++/a b.png")));
        assert_eq!(resolve(&dir, "/missing"), None);
        assert_eq!(resolve(&dir, "/../secret"), None);
        assert_eq!(resolve(&dir, "/%zz"), None);

        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
