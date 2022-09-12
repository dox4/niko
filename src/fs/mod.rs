use walkdir::WalkDir;

pub fn walking_dir(path: &str) -> impl Iterator<Item = walkdir::DirEntry> {
    WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
}
