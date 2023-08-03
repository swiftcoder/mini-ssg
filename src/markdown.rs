use anyhow::anyhow;
use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Tag};
use std::{ops::Range, str::FromStr};
use tera::Tera;
use url::Url;

use combine::{
    between,
    parser::{
        char::{spaces, string as Str},
        range::take_while,
        repeat::SepBy,
    },
    sep_by, EasyParser, Parser, Stream,
};

use crate::{highlighter::Highlighter, page::PartialPage};

#[derive(Clone, Debug)]
pub struct Argument {
    name: String,
    value: String,
}

#[derive(Clone, Debug)]
pub struct ShortCode {
    name: String,
    arguments: Vec<Argument>,
}

pub fn lit<I>(l: &'static str) -> impl Parser<I, Output = String>
where
    I: Stream<Token = char>,
{
    Str(l).map(|s| s.to_string()).skip(spaces())
}

fn parse_shortcode(input: &str) -> anyhow::Result<ShortCode> {
    let ident = || take_while(|c: char| c.is_alphanumeric() || c == '_').skip(spaces());
    let literal_str = between(lit("\""), lit("\""), take_while(|c: char| c != '\"')).skip(spaces());
    let arg = (ident(), lit("="), literal_str).map(|t: (&str, String, &str)| Argument {
        name: t.0.to_string(),
        value: t.2.to_string(),
    });
    let arg_list: SepBy<Vec<_>, _, _> = sep_by(arg, lit(","));
    let args = between(lit("("), lit(")"), arg_list);

    let mut function = between(
        lit("{{"),
        lit("}}"),
        (ident(), args).map(|t: (&str, _)| ShortCode {
            name: t.0.to_string(),
            arguments: t.1,
        }),
    );

    let result = function
        .easy_parse(input)
        .map_err(|e| e.map_range(|r| format!("{:?}", r)))
        .map_err(|e| e.map_position(|p| p.translate_position(input)))?;

    Ok(result.0)
}

pub fn render_shortcode(input: &str, page: &PartialPage, tera: &Tera) -> anyhow::Result<String> {
    let shortcode = parse_shortcode(input)?;

    for template in tera.get_template_names() {
        if let Some(name) = template.strip_prefix("shortcodes/") {
            let mut short_name = name.to_string();
            if let Some(i) = short_name.rfind('.') {
                short_name = short_name[0..i].to_string();
            }

            if short_name == shortcode.name {
                let mut ctx = tera::Context::new();

                for arg in &shortcode.arguments {
                    ctx.insert(&arg.name, &arg.value);
                }

                ctx.insert("page", page);

                return Ok(tera.render(template, &ctx)?);
            }
        }
    }

    Err(anyhow!("unknown shortcode '{}'", shortcode.name))
}

pub fn render_markdown(
    input: &str,
    page: &PartialPage,
    highlighter: &Highlighter,
) -> anyhow::Result<String> {
    let mut events = vec![];

    let mut in_code_block = false;
    let mut lang = String::new();
    let mut code = String::new();

    for event in pulldown_cmark::Parser::new(input) {
        match event {
            Event::Start(Tag::Image(link_type, mut dest_url, title)) => {
                // transform any relative URLs to absolute
                // if we don't do this, page summaries rendered on other pages
                // will contain (broken) relative links
                if Url::from_str(&dest_url).is_err() {
                    let result = page.permalink.join(&dest_url).unwrap();
                    dest_url = result.to_string().into();
                }
                events.push(Event::Start(Tag::Image(link_type, dest_url, title)));
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                lang = if let CodeBlockKind::Fenced(name) = kind {
                    name.to_string()
                } else {
                    "".to_string()
                };
            }
            Event::Text(t) if in_code_block => {
                code.push_str(&t);
            }
            Event::End(Tag::CodeBlock(_)) if in_code_block => {
                let result = highlighter.highlight(&lang, &code)?;

                events.push(Event::Html(CowStr::from(result)));

                in_code_block = false;
                code = String::new();
            }
            _ => events.push(event),
        }
    }

    let mut contents = String::new();
    html::push_html(&mut contents, events.into_iter());

    Ok(contents)
}

enum ContentRange {
    Markdown(Range<usize>),
    ShortCode(Range<usize>),
}

pub fn render_content(
    input: &str,
    page: &PartialPage,
    tera: &Tera,
    highlighter: &Highlighter,
) -> anyhow::Result<String> {
    let mut input = input.to_string();

    let mut ranges = vec![];

    let mut last = 0;
    while let Some(start) = input[last..].find("{{") {
        if start > 0 {
            ranges.push(ContentRange::Markdown(last..last + start));
        }

        if let Some(end) = input[last + start..].find("}}") {
            ranges.push(ContentRange::ShortCode(
                last + start..last + start + end + 2,
            ));
            last = last + start + end + 2;
        } else {
            return Err(anyhow!("unterminated shortcode"));
        }
    }

    if last < input.len() {
        ranges.push(ContentRange::Markdown(last..input.len()))
    }

    ranges.reverse();

    for range in ranges {
        match range {
            ContentRange::Markdown(r) => input.replace_range(
                r.clone(),
                &render_markdown(&input[r.clone()], page, highlighter)?,
            ),
            ContentRange::ShortCode(r) => {
                input.replace_range(r.clone(), &render_shortcode(&input[r.clone()], page, tera)?)
            }
        }
    }

    Ok(input)
}
