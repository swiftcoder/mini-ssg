use std::collections::HashMap;
use url::Url;

use crate::{slugify, Taxonomy};

pub struct GetTaxonomyURL {
    base_url: Url,
    taxonomies: HashMap<String, Taxonomy>,
}

impl GetTaxonomyURL {
    pub fn new(base_url: Url, taxonomies: &[Taxonomy]) -> Self {
        let taxonomies = taxonomies
            .iter()
            .map(|t| (t.name.to_string(), t.clone()))
            .collect::<HashMap<String, Taxonomy>>();
        GetTaxonomyURL {
            base_url,
            taxonomies,
        }
    }
}

impl tera::Function for GetTaxonomyURL {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let kind = args
            .get("kind")
            .cloned()
            .map(tera::from_value::<String>)
            .transpose()?
            .expect("missing kind");
        let name = args
            .get("name")
            .cloned()
            .map(tera::from_value::<String>)
            .transpose()?
            .expect("missing name");

        if let Some(taxonomy) = self.taxonomies.get(&kind) {
            let path = slugify(&(taxonomy.name.clone() + "/" + &name));
            let result = self.base_url.join(path.trim()).unwrap();

            Ok(tera::to_value::<String>(result.into())?)
        } else {
            Err(format!("no such taxonomy {}", kind).into())
        }
    }

    fn is_safe(&self) -> bool {
        true
    }
}
