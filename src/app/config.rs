/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use std::collections::HashMap;
use std::fs::{File, create_dir_all};
use std::io::{self, BufReader, Write};
use std::path::{Path, PathBuf};

use mg_settings::{Config, EnumFromStr, Parser, ParseResult};
use mg_settings::errors::ResultExt;

use app::settings::DefaultConfig;
use super::{
    Modes,
    ModesHash,
    COMMAND_MODE,
    COMPLETE_NEXT_COMMAND,
    COMPLETE_PREVIOUS_COMMAND,
    COPY,
    CUT,
    ENTRY_DELETE_NEXT_CHAR,
    ENTRY_DELETE_NEXT_WORD,
    ENTRY_DELETE_PREVIOUS_WORD,
    ENTRY_END,
    ENTRY_NEXT_CHAR,
    ENTRY_NEXT_WORD,
    ENTRY_PREVIOUS_CHAR,
    ENTRY_PREVIOUS_WORD,
    ENTRY_SMART_HOME,
    INSERT_MODE,
    NORMAL_MODE,
    PASTE,
};

/// Create the default config directories and files.
pub fn create_default_config(default_config: Vec<DefaultConfig>) -> Result<(), io::Error> {
    for config_item in default_config {
        match config_item {
            DefaultConfig::Dir(directory) => create_dir_all(directory?)?,
            DefaultConfig::File(name, content) => create_default_config_file(&name?, content)?,
        }
    }
    Ok(())
}

/// Create the config file with its default content if it does not exist.
pub fn create_default_config_file(path: &Path, content: &'static str) -> Result<(), io::Error> {
    if !path.exists() {
        let mut file = File::create(path)?;
        write!(file, "{}", content)?;
    }
    Ok(())
}

/// Parse a configuration file.
pub fn parse_config<P: AsRef<Path>, COMM: EnumFromStr>(filename: P, user_modes: Modes, include_path: Option<PathBuf>)
    -> (Parser<COMM>, ParseResult<COMM>, ModesHash)
{
    let mut parse_result = ParseResult::new();

    let mut modes = HashMap::new();
    for &(key, value) in user_modes {
        let _ = modes.insert(key, value);
    }
    assert!(modes.insert("n", NORMAL_MODE).is_none(), "Duplicate mode prefix n.");
    assert!(modes.insert("c", COMMAND_MODE).is_none(), "Duplicate mode prefix c.");
    assert!(modes.insert("i", INSERT_MODE).is_none(), "Duplicate mode prefix i.");
    let config = Config {
        application_commands: vec![COMPLETE_NEXT_COMMAND, COMPLETE_PREVIOUS_COMMAND, COPY, CUT, ENTRY_DELETE_NEXT_CHAR,
            ENTRY_DELETE_NEXT_WORD, ENTRY_DELETE_PREVIOUS_WORD, ENTRY_END, ENTRY_NEXT_CHAR, ENTRY_NEXT_WORD,
            ENTRY_PREVIOUS_CHAR, ENTRY_PREVIOUS_WORD, ENTRY_SMART_HOME, PASTE],
        mapping_modes: modes.keys().cloned().collect(),
    };
    let mut parser = Parser::new_with_config(config);
    if let Some(include_path) = include_path {
        parser.set_include_path(include_path);
    }

    let file = File::open(&filename)
        .chain_err(|| format!("failed to open settings file `{}`", filename.as_ref().to_string_lossy()));
    let file = rtry_no_return!(parse_result, file, { return (parser, parse_result, modes); });
    let buf_reader = BufReader::new(file);
    let parse_result = parser.parse(buf_reader);
    (parser, parse_result, modes)
}
