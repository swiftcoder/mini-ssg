use std::path::PathBuf;

use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Page {
    #[serde(skip)]
    pub name: String,
    #[serde(skip)]
    pub output_path: PathBuf,
    #[serde(skip)]
    pub template_name: String,
    pub title: String,
    pub description: String,
    pub date: String,
    pub content: String,
    pub summary: Option<String>,
    pub permalink: String,
}
