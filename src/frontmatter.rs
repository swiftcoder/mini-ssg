use anyhow::anyhow;

pub fn parse<D>(data: &str) -> anyhow::Result<(D, &str)>
where
    D: serde::de::DeserializeOwned,
{
    let marker = "+++";

    let start = data.find(marker).expect("missing frontmatter");

    if start != 0 {
        return Err(anyhow!("frontmatter not at beginning of file"));
    }

    let start = start + 3;

    let end = data[start..]
        .find(marker)
        .expect("unterminated frontmatter");

    let frontmatter = &data[start..start + end];

    let end = start + end + 3;
    let extra = &data[end..];

    Ok((toml::from_str::<D>(frontmatter.trim())?, extra.trim_start()))
}
