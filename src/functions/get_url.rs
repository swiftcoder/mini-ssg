use std::collections::HashMap;
use url::Url;

pub struct GetURL {
    base_url: Url,
}

impl GetURL {
    pub fn new(base_url: Url) -> Self {
        GetURL { base_url }
    }
}

impl tera::Function for GetURL {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let path = args
            .get("path")
            .cloned()
            .map(tera::from_value::<String>)
            .transpose()?
            .expect("missing path");

        let result = self.base_url.join(path.trim()).unwrap();

        Ok(tera::to_value::<String>(result.into())?)
    }

    fn is_safe(&self) -> bool {
        true
    }
}
