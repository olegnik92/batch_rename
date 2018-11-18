extern crate regex;

use std::io::prelude::*;
use std::io::{BufReader};
use std::fs;
use std::path::PathBuf;

use regex::Regex;

fn main() {
    let config = read_config();
    let rename_items = get_rename_items(&config);
    rename_all(rename_items);
    pause();
}

struct Config {
    work_dir: String,
    rename_rules: Vec<RenameRule>
}

struct RenameRule {
    search_pattern: Regex,
    replace_pattern: String
}

impl RenameRule {
    fn apply(&self, line: &String) -> String {
        String::from(self.search_pattern.replace(line, &self.replace_pattern as &str))
    }
}

fn read_config() -> Config {
    let mut result = Config {work_dir: String::new(), rename_rules: Vec::new() };
    let mut is_work_dir_readed = false;

    let file = fs::File::open("./config.txt").expect("Failed to open config file");
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let config_string = line.unwrap();
        if is_work_dir_readed {
            result.rename_rules.push(make_rename_rule(config_string));
        } else {
            result.work_dir = config_string;
            is_work_dir_readed = true;
        }
    }

    result
}

fn make_rename_rule(line: String) -> RenameRule {
    let parts: Vec<&str> = line.split("=>").collect();
    RenameRule { search_pattern: Regex::new(parts[0]).expect("Failed to parse Regex"), replace_pattern: String::from(parts[1]) }
}

struct RenameItem {
    from: PathBuf,
    to: PathBuf
}

fn get_rename_items(config: &Config) -> Vec<RenameItem> {
    let mut result = Vec::new();
    let entries = fs::read_dir(config.work_dir.clone()).expect("Failed to read work_dir files");

    for entry in entries {
        let path = entry.unwrap().path();
        let extension = String::from(path.extension().unwrap().to_string_lossy());
        let old_stem = String::from(path.file_stem().unwrap().to_string_lossy());
        let new_stem = get_new_file_stem(&old_stem, config);
        let old_path = path.with_file_name(old_stem + "." + &extension);
        let new_path = path.with_file_name(new_stem + "." + &extension);
        if old_path != new_path {
            let rename_item = RenameItem { from: old_path, to: new_path};
            result.push(rename_item);
        }

    }
    result
}

fn get_new_file_stem(stem: &String, config: &Config) -> String {
    let mut new_stem = stem.clone();
    for rule in &config.rename_rules {
        new_stem = rule.apply(&new_stem);
    }

    new_stem
}


fn rename_all(items: Vec<RenameItem>) {
    for item in items {
        let mut counter = 1;
        let mut new_name = item.to.clone();
        let extension = String::from(item.to.extension().unwrap().to_string_lossy());
        while new_name.exists() {
            new_name = item.to.with_file_name(String::from(item.to.file_stem().unwrap().to_string_lossy()) + "___" + &counter.to_string() + "." + &extension);
            counter = counter + 1;
        }

        println!("{0}  ===> {1}", item.from.display(), new_name.display());
        fs::rename(item.from, new_name).expect("Failed to rename file");
    }
}

fn pause() {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}