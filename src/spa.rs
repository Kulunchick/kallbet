//! Serves the built SPA (`web/dist`): real files as they are, and
//! `index.html` for every other path so client-side routes survive a reload.

use std::path::Path;

use tower_http::services::{ServeDir, ServeFile};

pub fn service(web_dir: &Path) -> ServeDir<ServeFile> {
    ServeDir::new(web_dir).fallback(ServeFile::new(web_dir.join("index.html")))
}
