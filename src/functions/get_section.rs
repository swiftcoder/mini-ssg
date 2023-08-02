use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{section::Section, site::Site};

pub struct GetSection {
    site: Arc<Site>,
}

impl GetSection {
    pub fn new(site: Arc<Site>) -> Self {
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

        for page in self.site.pages.values() {
            if page.name.starts_with(&prefix) {
                section.pages.push(page.clone())
            }
        }

        section.pages.sort_by_key(|p| p.date.clone());
        section.pages.reverse();

        Ok(tera::to_value(section)?)
    }
}
