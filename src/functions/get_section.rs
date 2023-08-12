use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::{section::Section, site::Site};

/// Unlike Zola, you don't have to declare sections. get_section() just recursively
/// grabs all pages that are children of the requested section.
pub struct GetSection {
    site: Arc<RwLock<Site>>,
}

impl GetSection {
    pub fn new(site: Arc<RwLock<Site>>) -> Self {
        Self { site }
    }
}

impl tera::Function for GetSection {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let path = args
            .get("path")
            .cloned()
            .map(tera::from_value::<String>)
            .transpose()?
            .expect("missing path");

        let mut prefix = PathBuf::from(path);
        prefix.pop();
        let prefix = prefix.to_string_lossy().to_string();

        let mut section = Section { pages: vec![] };

        let site = self.site.try_read().map_err(|e| e.to_string())?;

        for page in site.pages.values() {
            if page.name.starts_with(&prefix) {
                section.pages.push(page.clone())
            }
        }

        section.pages.sort_by_key(|p| p.date.clone());
        section.pages.reverse();

        Ok(tera::to_value(section)?)
    }
}
