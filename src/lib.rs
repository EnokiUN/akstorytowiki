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
    Subtitle {
        text: Option<String>,
        x: Option<u32>,
        y: Option<u32>,
        size: Option<u32>,
        width: Option<u32>,
        alignment: Option<String>,
        delay: Option<f32>,
    },
    Decision {
        options: HashMap<String, String>,
    },
    Predicate {
        references: Vec<String>,
    },
    Blocker {
        fade_time: f32,
        block: bool,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    },
    CharSlot {
        slot: Option<String>,
        name: Option<String>,
        duration: Option<f32>,
    },
    PlayMusic {
        intro: Option<String>,
        key: String,
        volume: f32,
    },
    Header {
        fit_mode: String,
        key: String,
        is_skippable: bool,
    },
    PlaySound {
        key: String,
        volume: f32,
        channel: Option<u8>,
        r#loop: bool,
    },
    StopSound {
        channel: Option<u8>,
        fade_time: Option<f32>,
    },
    CameraShake {
        x_strength: u32,
        y_strength: u32,
        vibrato: u32,
        randomness: u32,
        fadeout: bool,
        block: bool,
        duration: Option<f32>,
    },
    BgEffect {
        name: Option<String>,
        layer: Option<u32>,
    },
    Image {
        image: Option<String>,
        screen_adapt: Option<String>,
        fade_time: Option<f32>,
    },
    ImageTween {
        x_scale_to: f32,
        y_scale_to: f32,
        x_scale_from: f32,
        y_scale_from: f32,
        duration: f32,
        block: bool,
    },
    Delay {
        time: f32,
    },
    StopMusic,
    Dialog,
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
            image: args.get("image").cloned(),
            screen_adapt: args.get("screenadapt").cloned(),
            block: args
                .get("block")
                .cloned()
                .unwrap_or_else(|| "false".to_string())
                .parse()?,
        },
        "multiline" => Line::Multiline {
            name: args.get("name").unwrap().to_string(),
            text: content,
        },
        "line" => Line::Line {
            name: args.get("name").unwrap().to_string(),
            text: content,
        },
        "narration" => Line::Narration { text: content },
        "subtitle" => Line::Subtitle {
            text: args.get("text").cloned(),
            x: args.get("x").map(|d| d.parse().ok()).unwrap_or(None),
            y: args.get("y").map(|d| d.parse().ok()).unwrap_or(None),
            size: args.get("size").map(|d| d.parse().ok()).unwrap_or(None),
            width: args.get("width").map(|d| d.parse().ok()).unwrap_or(None),
            alignment: args.get("alignment").cloned(),
            delay: args.get("delay").map(|d| d.parse().ok()).unwrap_or(None),
        },
        "decision" => Line::Decision {
            options: args
                .get("values")
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
                .get("references")
                .unwrap()
                .split(';')
                .map(|s| s.to_string())
                .collect(),
        },
        "blocker" => Line::Blocker {
            fade_time: args.get("fadetime").unwrap().parse()?,
            block: args.get("block").unwrap().parse()?,
            r: args.get("r").unwrap().parse()?,
            g: args.get("g").unwrap().parse()?,
            b: args.get("b").unwrap().parse()?,
            a: args.get("a").unwrap().parse()?,
        },
        "charslot" => Line::CharSlot {
            slot: args.get("slot").cloned(),
            name: args.get("name").cloned(),
            duration: args.get("duration").map(|d| d.parse().ok()).unwrap_or(None),
        },
        "playmusic" => Line::PlayMusic {
            intro: args.get("intro").cloned(),
            key: args.get("key").unwrap().to_string(),
            volume: args
                .get("time")
                .cloned()
                .unwrap_or_else(|| "1.0".to_string())
                .parse()?,
        },
        "header" => Line::Header {
            fit_mode: args.get("fit_mode").unwrap().to_string(),
            key: args.get("key").unwrap().to_string(),
            is_skippable: args
                .get("is_skippable")
                .cloned()
                .unwrap_or_else(|| "false".to_string())
                .parse()?,
        },
        "delay" => Line::Delay {
            time: args.get("time").unwrap().parse()?,
        },
        "playsound" => Line::PlaySound {
            key: args.get("key").unwrap().to_string(),
            volume: args
                .get("volume")
                .cloned()
                .unwrap_or_else(|| "1.0".to_string())
                .parse()?,
            channel: args.get("channel").map(|d| d.parse().ok()).unwrap_or(None),
            r#loop: args
                .get("loop")
                .cloned()
                .unwrap_or_else(|| "false".to_string())
                .parse()?,
        },
        "stopsound" => Line::StopSound {
            channel: args.get("channel").map(|d| d.parse().ok()).unwrap_or(None),
            fade_time: args.get("fadetime").map(|d| d.parse().ok()).unwrap_or(None),
        },
        "camerashake" => Line::CameraShake {
            x_strength: args.get("xstrength").unwrap().parse()?,
            y_strength: args.get("ystrength").unwrap().parse()?,
            vibrato: args.get("vibrato").unwrap().parse()?,
            randomness: args.get("vibrato").unwrap().parse()?,
            fadeout: args
                .get("fadeout")
                .cloned()
                .unwrap_or_else(|| "false".to_string())
                .parse()?,
            block: args
                .get("block")
                .cloned()
                .unwrap_or_else(|| "false".to_string())
                .parse()?,
            duration: args.get("duration").map(|d| d.parse().ok()).unwrap_or(None),
        },
        "bgeffect" => Line::BgEffect {
            name: args.get("name").cloned(),
            layer: args.get("duration").map(|d| d.parse().ok()).unwrap_or(None),
        },
        "image" => Line::Image {
            image: args.get("image").cloned(),
            screen_adapt: args.get("screenadapt").cloned(),
            fade_time: args.get("fadetime").map(|d| d.parse().ok()).unwrap_or(None),
        },
        "imagetween" => Line::ImageTween {
            x_scale_to: args.get("xScaleTo").unwrap().parse()?,
            y_scale_to: args.get("yScaleTo").unwrap().parse()?,
            x_scale_from: args.get("xScaleFrom").unwrap().parse()?,
            y_scale_from: args.get("xScaleFrom").unwrap().parse()?,
            duration: args.get("duration").unwrap().parse()?,
            block: args
                .get("block")
                .cloned()
                .unwrap_or_else(|| "false".to_string())
                .parse()?,
        },
        "stopmusic" => Line::StopMusic,
        "dialog" => Line::Dialog,
        _ => Line::Other {
            line_type,
            arguments: args,
        },
    })
}
