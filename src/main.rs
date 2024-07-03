use akstorytowiki::story_to_wiki;

use anyhow::{Context, Result};
use std::{env, fs};

fn main() -> Result<()> {
    let args = env::args()
        .nth(1)
        .context("You must pass the name of the Arknights story file")?;
    let file = fs::read_to_string(args).context("Couldn't open the supplied file")?;

    println!("{}", story_to_wiki(file));

    Ok(())
}
