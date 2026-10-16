use akstorytowiki::story_to_wiki;

use anyhow::{Context, Result};
use std::{
    env, fs,
    io::{self, IsTerminal, Read},
};

fn main() -> Result<()> {
    let mut stdin = io::stdin();
    let file = if stdin.is_terminal() {
        let args = env::args()
            .nth(1)
            .context("You must pass the name of the Arknights story file")?;
        fs::read_to_string(args).context("Couldn't open the supplied file")?
    } else {
        let mut content = String::new();
        stdin
            .read_to_string(&mut content)
            .context("Couldn't read from stdin")?;
        content
    };

    println!("{}", story_to_wiki(file));

    Ok(())
}
