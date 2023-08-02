use std::collections::HashMap;

use crate::page::Page;

pub struct Site {
    pub pages: HashMap<String, Page>,
}

impl Site {
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
        }
    }
}
