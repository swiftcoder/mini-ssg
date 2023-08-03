use pulldown_cmark::html;

pub struct Markdown {}

impl tera::Filter for Markdown {
    fn filter(
        &self,
        value: &tera::Value,
        _args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let input = tera::from_value::<String>(value.clone())?;

        let parser = pulldown_cmark::Parser::new(&input);

        let mut contents = String::new();
        html::push_html(&mut contents, parser);

        Ok(tera::to_value(contents)?)
    }
}
