use std::{collections::HashMap, path::PathBuf};

use serde::Serialize;
use url::Url;

/// Page variables that are available when shortcodes are rendered
#[derive(Serialize, Clone)]
pub struct PartialPage {
    pub title: String,
    pub description: String,
    pub date: Option<String>,
    pub permalink: Url,
}

/// The full set of page variables
#[derive(Serialize, Clone)]
pub struct Page {
    #[serde(skip)]
    pub name: String,
    #[serde(skip)]
    pub output_path: PathBuf,
    #[serde(skip)]
    pub template_name: String,
    #[serde(skip)]
    pub taxonomy: Option<(String, String)>,
    pub title: String,
    pub description: String,
    pub date: Option<String>,
    pub permalink: Url,
    pub content: String,
    pub summary: Option<String>,
    // pub key: String,
    pub taxonomies: HashMap<String, Vec<String>>,
}
