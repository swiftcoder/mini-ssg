use serde::Serialize;

use crate::page::Page;

#[derive(Serialize)]
pub struct Section {
    pub pages: Vec<Page>,
}
