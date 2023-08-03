use std::{
    collections::HashSet,
    fs::{self, create_dir_all, remove_dir_all},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use chrono::Utc;
use clap::Parser;
use page::Page;
use serde::{self, Deserialize, Serialize};
use site::Site;
use tera::Tera;
use toml::value::Datetime;
use url::Url;
use walkdir::WalkDir;

use crate::{
    functions::{get_section::GetSection, get_url::GetURL, markdown::Markdown},
    highlighter::Highlighter,
    markdown::render_content,
    page::PartialPage,
};

mod frontmatter;
mod functions;
mod highlighter;
mod markdown;
mod page;
mod section;
mod site;

#[derive(Parser, Debug)]
#[command(name = "Mini Static Site Generator")]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(default_value = ".")]
    path: String,
    #[arg(default_value = "public")]
    output_dir: String,
    #[arg(short, long)]
    local: bool,
}

pub struct Context {
    home: PathBuf,
    output_dir: PathBuf,
    config: Config,
}

impl Context {
    pub fn new(home: PathBuf, output_dir: PathBuf, local: bool) -> anyhow::Result<Self> {
        let config_file = home.join("config.toml");
        let config_text = fs::read_to_string(config_file)?;
        let mut config: Config = toml::from_str(&config_text)?;

        if local {
            config.base_url = Url::from_str("http://127.0.0.1:1111")?;
        }

        Ok(Self {
            home,
            output_dir,
            config,
        })
    }

    fn clean_output_dir(&self) -> anyhow::Result<()> {
        remove_dir_all(&self.output_dir)?;
        create_dir_all(&self.output_dir)?;
        Ok(())
    }

    fn absolute<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.home.join(path.as_ref())
    }

    fn relative(&self, path: &Path) -> anyhow::Result<PathBuf> {
        Ok(path.strip_prefix(&self.home)?.into())
    }

    fn create_output_dir(&self, path: &Path) -> anyhow::Result<()> {
        let output = self.output_dir.join(path);
        Ok(fs::create_dir_all(output)?)
    }

    fn copy_to_output(&self, file: &Path, path: &Path) -> anyhow::Result<()> {
        path.parent()
            .map(|p| self.create_output_dir(p))
            .transpose()?;

        let output = self.output_dir.join(path);

        fs::copy(file, output)?;

        Ok(())
    }

    fn write_to_output(&self, path: &Path, contents: &str) -> anyhow::Result<()> {
        path.parent()
            .map(|p| self.create_output_dir(p))
            .transpose()?;

        let output = self.output_dir.join(path);

        fs::write(output, contents)?;

        Ok(())
    }
}

fn setup_template_engine(context: &Context) -> anyhow::Result<Tera> {
    let template_dir = context.absolute("templates");

    let tera = Tera::new(&template_dir.join("**").join("*").to_string_lossy())?;

    println!(
        "loaded templates: {:?}",
        tera.get_template_names().collect::<Vec<_>>()
    );

    Ok(tera)
}

#[derive(Deserialize)]
struct FrontMatter {
    title: Option<String>,
    date: Option<Datetime>,
    template: Option<String>,
    description: Option<String>,
    transparent: Option<bool>,
}

#[derive(Deserialize, Serialize)]
struct Config {
    title: String,
    base_url: Url,
}

impl Config {
    pub fn make_permalink(&self, path: &str) -> Url {
        let escaped = path.strip_suffix("index.html").unwrap_or(path);
        self.base_url.join(escaped).unwrap()
    }
}

fn output_path(relative_path: &Path, template_name: Option<&str>) -> PathBuf {
    let mut output_path = relative_path.with_extension("");
    if let Some(extension) = Path::new(template_name.unwrap_or("")).extension() {
        if extension.eq("html") {
            if output_path.file_name().map(|n| n.eq("index")).unwrap() {
                output_path.pop();
            }
            output_path = output_path.join("index.html");
        } else {
            output_path = output_path.with_extension(extension);
        }
    }

    output_path
}

fn copy_static_files(context: &Context) -> anyhow::Result<()> {
    let static_dir: PathBuf = context.absolute("static");

    for entry in WalkDir::new(&static_dir) {
        let entry = entry?;

        if !entry.file_type().is_file() {
            continue;
        }

        context.copy_to_output(entry.path(), entry.path().strip_prefix(&static_dir)?)?;
    }

    Ok(())
}

fn process_templated_files(
    context: &Context,
    tera: &Tera,
    highlighter: &Highlighter,
) -> anyhow::Result<Site> {
    let static_file_extensions = HashSet::from(["png", "webp", "jpg", "jpeg", "gif", "gif"]);

    let mut site = Site::new();

    let content_dir: PathBuf = context.absolute("content");

    for entry in WalkDir::new(&content_dir) {
        let entry = entry?;

        if !entry.file_type().is_file() {
            continue;
        }

        if let Some(extension) = entry.path().extension() {
            if static_file_extensions.contains(extension.to_str().unwrap_or("")) {
                println!(
                    "copying {} to output",
                    context.relative(entry.path())?.display()
                );

                context.copy_to_output(entry.path(), entry.path().strip_prefix(&content_dir)?)?;
                continue;
            }
        }

        println!("compiling {}", context.relative(entry.path())?.display());

        let contents = fs::read_to_string(entry.path())?;

        let (frontmatter, body) = frontmatter::parse::<FrontMatter>(&contents)?;

        if let Some(filename) = entry.path().file_name() {
            if filename.to_string_lossy().starts_with('_') {
                continue;
            }
        }

        let template_name = frontmatter.template.as_deref().unwrap_or("page.html");

        let output_path = output_path(
            entry.path().strip_prefix(&content_dir)?,
            Some(template_name),
        );

        let permalink = context.config.make_permalink(output_path.to_str().unwrap());

        let name = output_path.to_str().unwrap().to_string();

        let partial = PartialPage {
            title: frontmatter.title.unwrap_or(
                entry
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            ),
            date: frontmatter.date.and_then(|d| d.date).map(|d| d.to_string()),
            description: frontmatter.description.unwrap_or_default(),
            permalink,
        };

        let mut summary = None;

        if let Some(start) = body.find("<!--") {
            if let Some(end) = body[start + 4..].find("-->") {
                if body[start + 4..start + 4 + end]
                    .trim()
                    .eq_ignore_ascii_case("more")
                {
                    summary = Some(render_content(
                        &body[0..start],
                        &partial,
                        tera,
                        highlighter,
                    )?);
                }
            }
        }

        let content = render_content(body, &partial, tera, highlighter)?;

        let page = Page {
            name,
            output_path,
            template_name: template_name.to_string(),
            title: partial.title,
            date: partial.date,
            description: partial.description,
            permalink: partial.permalink.clone(),
            content,
            summary,
            key: partial.permalink.into(),
        };

        site.pages.insert(page.name.clone(), page);

        // let contents = render_page(context, tera, &page)?;

        // context.write_to_output(&output_path, &contents)?;
    }

    Ok(site)
}

fn render_page(
    context: &Context,
    tera: &Tera,
    page: &Page,
    site: Arc<Site>,
) -> anyhow::Result<String> {
    let mut ctx = tera::Context::new();

    let mut pages = site
        .pages
        .values()
        .filter(|p| p.date.is_some())
        .collect::<Vec<_>>();
    pages.sort_by_key(|p| p.date.clone().unwrap());
    pages.reverse();

    ctx.insert("config", &context.config);
    ctx.insert("page", &page);
    ctx.insert("pages", &pages);
    ctx.insert("current_url", &page.permalink);
    ctx.insert("last_updated", &Utc::now().to_string());

    Ok(tera.render(&page.template_name, &ctx)?)
}

fn render_pages_for_site(context: &Context, tera: &Tera, site: Arc<Site>) -> anyhow::Result<()> {
    for page in site.pages.values() {
        let contents = render_page(context, tera, page, site.clone())?;

        context.write_to_output(&page.output_path, &contents)?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("running with {args:?}");

    let home = PathBuf::from_str(&args.path)?;
    let output_dir = home.join(&args.output_dir);

    let context = Context::new(home, output_dir, args.local)?;

    context.clean_output_dir()?;

    copy_static_files(&context)?;

    let highlighter = Highlighter::new(&context)?;

    let mut tera = setup_template_engine(&context)?;

    tera.register_function("get_url", GetURL::new(context.config.base_url.clone()));
    tera.register_filter("markdown", Markdown {});

    let site = Arc::new(process_templated_files(&context, &tera, &highlighter)?);

    tera.register_function("get_section", GetSection::new(site.clone()));

    render_pages_for_site(&context, &tera, site.clone())?;

    Ok(())
}
