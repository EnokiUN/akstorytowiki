use std::collections::HashMap;

use anyhow::{Context, Result};

#[derive(Debug)]
pub enum Line {
    Background {
        image: Option<String>,
        screen_adapt: Option<String>,
        block: bool,
    },
    Multiline {
        name: String,
        text: String,
    },
    Line {
        name: String,
        text: String,
    },
    Narration {
        text: String,
    },
    Sticker {
        text: Option<String>,
    },
    Subtitle {
        text: Option<String>,
        // x: Option<u32>,
        // y: Option<u32>,
        // size: Option<u32>,
        // width: Option<u32>,
        // alignment: Option<String>,
        // delay: Option<f32>,
    },
    Decision {
        options: HashMap<String, String>,
    },
    Predicate {
        references: Vec<String>,
    },
    Blocker {
        fade_time: Option<u32>,
        block: bool,
        a: Option<f32>,
        r: Option<f32>,
        g: Option<f32>,
        b: Option<f32>,
    },
    Image {
        image: Option<String>,
    },
    CameraEffect {
        effect: Option<String>,
        amount: f32,
    },
    Other {
        line_type: String,
        arguments: HashMap<String, String>,
    },
}

pub fn parse_line(line: &str) -> Result<Line> {
    let mut line = line.chars();
    let mut parsing_header = true;
    let mut parsing_type = true;
    let mut parsing_arg_name = true;
    let mut quoted = false;
    let mut line_type = String::new();
    let mut arg_name_buffer = String::new();
    let mut arg_value_buffer = String::new();
    let mut args = HashMap::new();
    let mut content = String::new();
    let first = line
        .next()
        .context("Invalid syntax in Arknights story file")?;
    if first != '[' {
        parsing_header = false;
        parsing_type = false;
        parsing_arg_name = false;
        line_type = "narration".to_string();
        content.push(first);
    }
    loop {
        let Some(current) = line.next() else {
            break;
        };
        if !parsing_header {
            content.push(current);
            continue;
        }
        if !parsing_type {
            if current == '"' {
                quoted = !quoted;
                continue;
            }
            if [')', ' '].contains(&current) && !quoted {
                continue;
            }
            if (current == ',' || current == ']') && !quoted {
                if current == ']' {
                    parsing_header = false;
                }
                args.insert(arg_name_buffer.clone(), arg_value_buffer.clone());
                arg_name_buffer.clear();
                arg_value_buffer.clear();
                parsing_arg_name = true;
            } else if !parsing_arg_name {
                arg_value_buffer.push(current)
            } else if current == '=' {
                parsing_arg_name = false;
            } else {
                arg_name_buffer.push(current);
            }
            continue;
        }
        if !current.is_alphabetic() {
            parsing_type = false;
            if line_type == "name" {
                line_type = "line".to_string();
                arg_name_buffer = "name".to_string();
                parsing_arg_name = false;
            }
            continue;
        }
        line_type.push(current);
    }
    line_type = line_type.to_lowercase();

    Ok(match line_type.as_str() {
        "background" => Line::Background {
            image: args.remove("image"),
            screen_adapt: args.remove("screenadapt"),
            block: args
                .remove("block")
                .unwrap_or_else(|| "false".to_string())
                .parse()?,
        },
        "multiline" => Line::Multiline {
            name: args.remove("name").unwrap(),
            text: content,
        },
        "line" => Line::Line {
            name: args.remove("name").unwrap(),
            text: content,
        },
        "narration" => Line::Narration { text: content },
        "sticker" => Line::Sticker {
            text: args.remove("text"),
        },
        "subtitle" => Line::Subtitle {
            text: args.remove("text"),
            //x: args.get("x").map(|d| d.parse().ok()).unwrap_or(None),
            //y: args.get("y").map(|d| d.parse().ok()).unwrap_or(None),
            //size: args.get("size").map(|d| d.parse().ok()).unwrap_or(None),
            //width: args.get("width").map(|d| d.parse().ok()).unwrap_or(None),
            //alignment: args.get("alignment").cloned(),
            //delay: args.get("delay").map(|d| d.parse().ok()).unwrap_or(None),
        },
        "decision" => Line::Decision {
            options: args
                .remove("values")
                .unwrap()
                .split(';')
                .map(|s| s.to_string())
                .zip(
                    args.get("options")
                        .unwrap()
                        .split(';')
                        .map(|s| s.to_string()),
                )
                .collect(),
        },
        "predicate" => Line::Predicate {
            references: args
                .remove("references")
                .unwrap_or_else(|| "".to_string())
                .split(';')
                .map(|s| s.to_string())
                .collect(),
        },
        "blocker" => Line::Blocker {
            fade_time: args
                .remove("fadetime")
                .map(|d| d.parse().ok())
                .unwrap_or(None),
            block: args
                .remove("block")
                .unwrap_or_else(|| "false".to_string())
                .parse()?,
            r: args.remove("r").map(|d| d.parse().ok()).unwrap_or(None),
            g: args.remove("g").map(|d| d.parse().ok()).unwrap_or(None),
            b: args.remove("b").map(|d| d.parse().ok()).unwrap_or(None),
            a: args.remove("a").map(|d| d.parse().ok()).unwrap_or(None),
        },
        "animtext" => Line::Subtitle {
            text: Some(
                content
                    .replace("</>", "")
                    .replace("<p=1>", "")
                    .split("<p=2>")
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<String>>()
                    .join("<br/>"),
            ),
        },
        "image" => Line::Image {
            image: args.remove("image"),
        },
        "cameraeffect" => Line::CameraEffect {
            effect: args.remove("effect").map(|e| e.to_lowercase()),
            amount: args
                .remove("amount")
                .and_then(|f| f.parse().ok())
                .unwrap_or(0.0),
        },
        _ => Line::Other {
            line_type,
            arguments: args,
        },
    })
}

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

pub fn story_to_wiki(content: String) -> String {
    let lines: Vec<Line> = content.lines().map(|l| parse_line(l).unwrap()).collect();

    let mut content = String::new();
    let mut backgrounds = HashMap::new();
    let mut last_background = String::new();
    let mut characters = vec![];
    let mut last_author = None;
    let mut is_narration = false;
    let mut is_subtitle = false;
    let mut current_options = HashMap::new();
    let mut options_len = 0;

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
                    content.push_str(&format!("<br/>{}", text.trim()));
                } else if last_author.is_none() {
                    content.push_str(&format!("{{{{sc|{}|{}", name, text.trim()));
                    last_author = Some(name);
                } else {
                    content.push_str(&format!("}}}}\n{{{{sc|{}|{}", name, text.trim()));
                    last_author = Some(name);
                }
            }
            Line::Narration { text } | Line::Sticker { text: Some(text) } => {
                cleanup_open_tags(&mut content, &mut last_author, &mut false, &mut is_subtitle);
                let text = text.trim().replace("\\n", "");
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
                    content.push_str(&format!("<br/>{}", text.trim()));
                } else {
                    content.push_str(&format!("{{{{sc|{}", text.trim()));
                    is_subtitle = true;
                }
            }
            Line::Decision { options } => {
                if !characters.contains(&"Doctor") {
                    characters.push("Doctor".to_string());
                }
                cleanup_open_tags(
                    &mut content,
                    &mut last_author,
                    &mut is_narration,
                    &mut is_subtitle,
                );
                current_options = options.clone();
                options_len = options.len();
                if options.len() > 1 {
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
                if references.is_empty() {
                    continue;
                }
                if references.len() > 1 {
                    if current_options.len() == references.len() {
                        if content.ends_with("{{sc|mode=branchstart}}\n") {
                            content = content
                                .strip_suffix("{{sc|mode=branchstart}}\n")
                                .unwrap()
                                .to_string();
                            let mut line_content = current_options
                                .clone()
                                .into_iter()
                                .map(|(i, v)| (i.parse().unwrap(), v))
                                .collect::<Vec<(u32, String)>>();
                            line_content.sort_by(|a, b| a.0.cmp(&b.0));
                            let line_content = line_content
                                .into_iter()
                                .map(|(_, v)| v)
                                .collect::<Vec<String>>()
                                .join(" / ");
                            content.push_str(&format!("{{{{sc|Doctor|{}}}}}\n", line_content));
                        }
                    } else {
                        content.push_str("{{sc|mode=branchend}}\n");
                    }
                    continue;
                }
                if current_options.is_empty() {
                    continue;
                }
                if options_len > 1 {
                    if let Some(selection) = current_options.remove(&references[0]) {
                        if !content.ends_with("{{sc|mode=branchstart}}\n") {
                            content.push_str("{{sc|mode=branch}}\n");
                        }
                        content.push_str(&format!("{{{{sc|Doctor|{}}}}}\n", selection));
                    }
                }
            }
            Line::Blocker { a, .. } => {
                if a == Some(1.0) {
                    cleanup_open_tags(
                        &mut content,
                        &mut last_author,
                        &mut is_narration,
                        &mut is_subtitle,
                    );
                    content.push_str("{{sc|fades out and in|mode=background}}\n");
                }
            }
            Line::Image { image: Some(image) } => {
                cleanup_open_tags(
                    &mut content,
                    &mut last_author,
                    &mut is_narration,
                    &mut is_subtitle,
                );
                content.push_str(&format!("{{{{sc|{}|mode=image}}}}\n", image));
            }
            Line::CameraEffect {
                effect: Some(effect),
                amount,
            } => {
                if effect == "grayscale" {
                    cleanup_open_tags(
                        &mut content,
                        &mut last_author,
                        &mut is_narration,
                        &mut is_subtitle,
                    );
                    if amount == 0.0 {
                        content.push_str("{{sc|mode=flashbackend}}\n");
                    } else {
                        content.push_str("{{sc|mode=flashbackstart}}\n");
                    }
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
            //content.push_str(&format!("{{{{sc|SFX|{}|mode=action}}}}\n", key));
            //}
            _ => {}
        }
    }
    cleanup_open_tags(
        &mut content,
        &mut last_author,
        &mut is_narration,
        &mut is_subtitle,
    );

    let mut images_header = "|chars = ".to_string();
    for char in characters {
        images_header.push_str(&format!("{{{{si|mode=char|{}}}}}", char));
    }
    images_header = images_header.trim().to_string();
    images_header.push_str("\n|bgs = ");
    let mut backgrounds_vec = backgrounds.into_iter().collect::<Vec<(String, usize)>>();
    backgrounds_vec.sort_by(|t, o| t.1.cmp(&o.1));
    for (image, id) in backgrounds_vec {
        images_header.push_str(&format!("{{{{si|mode=bg|{}|{}}}}}", image, id));
    }
    format!("{}\n\n\n{}", images_header, content.trim())
}
