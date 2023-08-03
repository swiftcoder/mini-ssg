use std::path::Path;

use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};

use crate::Context;

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Highlighter {
    pub fn new(context: &Context) -> anyhow::Result<Self> {
        let mut syntax_set_builder = SyntaxSet::load_defaults_newlines().into_builder();
        syntax_set_builder.add_from_folder(context.absolute(Path::new("syntaxes")), true)?;
        let syntax_set = syntax_set_builder.build();

        let theme_set = ThemeSet::load_defaults();

        Ok(Self {
            syntax_set,
            theme_set,
        })
    }

    pub fn highlight(&self, lang: &str, input: &str) -> anyhow::Result<String> {
        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let theme = &self.theme_set.themes["base16-ocean.dark"];

        Ok(highlighted_html_for_string(
            input,
            &self.syntax_set,
            syntax,
            theme,
        )?)
    }
}
