use akstorytowiki::{parse_line, Line};

use anyhow::{Context, Result};
use std::{collections::HashMap, env, fs};

fn cleanup_open_tags(
    content: &mut String,
    last_author: &mut Option<String>,
    is_narration: &mut bool,
    is_subtitle: &mut bool,
) {
    if last_author.is_some() {
        content.push_str("}}\n");
        *last_author = None;
    }
    if *is_narration {
        content.push_str("|mode=speech}}\n");
        *is_narration = false;
    }
    if *is_subtitle {
        content.push_str("|mode=subtitle}}\n");
        *is_subtitle = false;
    }
}

fn main() -> Result<()> {
    let args = env::args()
        .nth(1)
        .context("You must pass the name of the Arknights story file")?;
    let file = fs::read_to_string(args).context("Couldn't open the supplied file")?;

    let lines: Vec<Line> = file.lines().map(|l| parse_line(l).unwrap()).collect();

    let mut content = String::new();
    let mut backgrounds = HashMap::new();
    let mut last_background = String::new();
    let mut characters = vec![];
    let mut last_author = None;
    let mut is_narration = false;
    let mut is_subtitle = false;
    let mut current_options = HashMap::new();

    for line in lines {
        match line {
            Line::Background { image, .. } => {
                let image = image.unwrap_or(last_background.clone());
                cleanup_open_tags(
                    &mut content,
                    &mut last_author,
                    &mut is_narration,
                    &mut is_subtitle,
                );
                if !backgrounds.contains_key(&image) {
                    backgrounds.insert(image.clone(), backgrounds.len() + 1);
                }
                if content.ends_with("{{sc|fades out and in|mode=background}}\n") {
                    content = content
                        .strip_suffix("{{sc|fades out and in|mode=background}}\n")
                        .unwrap()
                        .to_string();
                }
                content.push_str(&format!(
                    "{{{{sc|{}|mode=background}}}}\n",
                    backgrounds.get(&image).unwrap()
                ));
                last_background = image;
            }
            Line::Line { name, text } | Line::Multiline { name, text } => {
                if !characters.contains(&name) {
                    characters.push(name.clone());
                }
                cleanup_open_tags(&mut content, &mut None, &mut is_narration, &mut is_subtitle);
                if last_author.as_ref() == Some(&name) {
                    content.push_str(&format!("<br/>{}", text));
                } else if last_author.is_none() {
                    content.push_str(&format!("{{{{sc|{}|{}", name, text));
                    last_author = Some(name);
                } else {
                    content.push_str(&format!("}}}}\n{{{{sc|{}|{}", name, text));
                    last_author = Some(name);
                }
            }
            Line::Narration { text } => {
                cleanup_open_tags(&mut content, &mut last_author, &mut false, &mut is_subtitle);
                if is_narration {
                    content.push_str(&format!("<br/>{}", text));
                } else {
                    content.push_str(&format!("{{{{sc|{}", text));
                    is_narration = true;
                }
            }
            Line::Subtitle { text, .. } => {
                let Some(text) = text else {
                    continue;
                };
                cleanup_open_tags(
                    &mut content,
                    &mut last_author,
                    &mut is_narration,
                    &mut false,
                );
                if is_subtitle {
                    content.push_str(&format!("<br/>{}", text));
                } else {
                    content.push_str(&format!("{{{{sc|{}", text));
                    is_subtitle = true;
                }
            }
            Line::Decision { options } => {
                cleanup_open_tags(
                    &mut content,
                    &mut last_author,
                    &mut is_narration,
                    &mut is_subtitle,
                );
                if options.len() > 1 {
                    current_options = options;
                    content.push_str("{{sc|mode=branchstart}}\n");
                } else {
                    let selection = options.iter().next().unwrap().1;
                    content.push_str(&format!("{{{{sc|Doctor|{}}}}}\n", selection));
                }
            }
            Line::Predicate { references } => {
                cleanup_open_tags(
                    &mut content,
                    &mut last_author,
                    &mut is_narration,
                    &mut is_subtitle,
                );
                if references.len() > 1 {
                    content.push_str("{{sc|mode=branchend}}\n");
                    continue;
                }
                if current_options.is_empty() {
                    continue;
                }
                if !content.ends_with("{{sc|mode=branchstart}}\n") {
                    content.push_str("{{sc|mode=branch}}\n");
                }
                let selection = current_options.remove(&references[0]).unwrap();
                content.push_str(&format!("{{{{sc|Doctor|{}}}}}\n", selection));
            }
            Line::Blocker { a, .. } => {
                if a == 1 {
                    cleanup_open_tags(
                        &mut content,
                        &mut last_author,
                        &mut is_narration,
                        &mut is_subtitle,
                    );
                    content.push_str("{{sc|fades out and in|mode=background}}\n");
                }
            }
            //Line::PlaySound { key, .. } => {
            //if last_author.is_some() {
            //content.push_str("}}\n");
            //last_author = None;
            //}
            //if is_subtitle {
            //content.push_str("}}\n");
            //is_subtitle = false;
            //}
            //content.push_str(&format!("{{{{sc|Audio - {}|mode=action}}}}\n", key));
            //}
            _ => {}
        }
    }

    let mut images_header = "|chars = ".to_string();
    for char in characters {
        images_header.push_str(&format!(" {{{{si|mode=char|{}}}}}", char));
    }
    images_header = images_header.trim().to_string();
    images_header.push_str("|\n|bgs = ");
    for (image, id) in backgrounds.iter() {
        images_header.push_str(&format!(" {{{{si|mode=bg|{}|{}}}}}", image, id));
    }
    println!("{}\n\n\n{}", images_header, content);

    Ok(())
}
