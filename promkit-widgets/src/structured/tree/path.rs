use std::{fs, path};

use super::treez::Adapter;

pub struct PathAdapter;

impl Adapter for PathAdapter {
    type Node = path::PathBuf;
    type Error = anyhow::Error;

    fn id_of(&self, node: &Self::Node) -> Result<String, Self::Error> {
        node.file_name()
            .and_then(|name| name.to_str())
            .map(ToOwned::to_owned)
            .or_else(|| {
                let rendered = node.display().to_string();
                (!rendered.is_empty()).then_some(rendered)
            })
            .ok_or_else(|| anyhow::anyhow!("Failed to convert path to string"))
    }

    fn children_of(&self, node: &Self::Node) -> Result<Vec<Self::Node>, Self::Error> {
        if !node.is_dir() {
            return Ok(Vec::new());
        }

        let mut directories = Vec::new();
        let mut files = Vec::new();

        for entry in fs::read_dir(node)? {
            let path = entry?.path();
            if path.is_dir() {
                directories.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }

        directories.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        directories.extend(files);

        Ok(directories)
    }
}
