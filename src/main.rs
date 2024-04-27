use clap::Parser;
use regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    root_dir: String,
    #[clap(long, short, use_value_delimiter = true)]
    ignore_dirs: Vec<String>,
}

fn main() {
    let args = Cli::parse();

    println!(
        "Visiting {}. Ignoring folders that contain ({}) ",
        args.root_dir,
        args.ignore_dirs.join("|")
    );

    let re = regex::Regex::new(r"\[\[(.*?)\]\]").expect("Failed to compile regex");

    let mapping = retrieve_record(Path::new(&args.root_dir), HashMap::new(), &args.ignore_dirs)
        .unwrap_or_else(|err| panic!("Failed to retrieve record: {}", err));

    let canonical_root_dir = std::fs::canonicalize(&args.root_dir).unwrap();
    let canonical_root_dir_string = canonical_root_dir.to_str().unwrap();

    println!(
        "Calculated a canonical file path of {}",
        canonical_root_dir_string
    );

    let file_list: Vec<String> = mapping.values().cloned().collect();
    println!("List of files:");
    for file in file_list.iter() {
        let file_path = std::fs::canonicalize(file).unwrap();
        let relative_path_to_root_dir = file_path
            .to_str()
            .unwrap()
            .replace(&canonical_root_dir_string, "");
        let file_content = fs::read_to_string(&file_path).expect("Failed to read file");

        for cap in re.captures_iter(&file_content) {
            if let Some(file_path_in_mapping) = mapping.get(&cap[1]) {
                let mapping_file_path = std::fs::canonicalize(file_path_in_mapping).unwrap();
                let sanitized_mapping_file_path = mapping_file_path
                    .to_str()
                    .unwrap()
                    .replace(canonical_root_dir_string, "")
                    .replace(" ", "%20");

                let new_link = format!("[{}]({})", &cap[1], sanitized_mapping_file_path);
                println!(
                    "Replaced link: {} in file {}",
                    new_link, relative_path_to_root_dir
                );
                let original_file_content =
                    fs::read_to_string(file).expect("Failed to read file for replacement");
                let replaced_content = original_file_content.replace(&cap[0], &new_link);
                fs::write(file, replaced_content)
                    .expect("Failed to write replaced content to file");
            }
        }
    }
}

fn is_invalid_path(path: &Path, ignore_dirs: &Vec<String>) -> bool {
    let path_str = path.to_string_lossy();
    let is_dir = path.is_dir();
    let ends_with_md = path_str.ends_with(".md");

    let is_valid_dir_or_file = is_dir || ends_with_md;

    !is_valid_dir_or_file
        || ignore_dirs
            .iter()
            .any(|ignore_dir| path_str.contains(ignore_dir))
}

fn retrieve_record(
    path: &Path,
    mut acc: HashMap<String, String>,
    ignore_dirs: &Vec<String>,
) -> Result<HashMap<String, String>, String> {
    if is_invalid_path(path, ignore_dirs) {
        return Ok(acc);
    }

    println!("Exploring {}", path.to_str().unwrap());
    if path.is_file() {
        let mut new_acc = acc.clone();
        new_acc.insert(
            path.file_stem().unwrap().to_string_lossy().to_string(),
            path.to_string_lossy().to_string(),
        );
        return Ok(new_acc);
    }

    let entries = fs::read_dir(path).map_err(|err| err.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        acc = retrieve_record(&entry.path(), acc, ignore_dirs)?;
    }

    Ok(acc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_markdown_file() {
        let path = Path::new("example.md");
        let ignore_dirs = vec!["temp".to_string()];
        assert!(!is_invalid_path(path, &ignore_dirs));
    }

    #[test]
    fn test_invalid_non_markdown_file() {
        let path = Path::new("example.txt");
        let ignore_dirs = vec!["temp".to_string()];
        assert!(is_invalid_path(path, &ignore_dirs));
    }

    #[test]
    fn test_ignored_directory_file() {
        let path = Path::new("temp/example.md");
        let ignore_dirs = vec!["temp".to_string()];
        assert!(is_invalid_path(path, &ignore_dirs));
    }

    #[test]
    fn test_ignored_directory() {
        let path = Path::new("./fixtures/ignored_dir");
        let ignore_dirs = vec!["ignored_dir".to_string()];
        assert!(is_invalid_path(path, &ignore_dirs));
    }

    #[test]
    fn test_non_ignored_directory() {
        let path = Path::new("./src/example.md");
        let ignore_dirs = vec!["temp".to_string()];
        assert!(!is_invalid_path(path, &ignore_dirs));
    }

    #[test]
    fn test_directory() {
        let path = Path::new("./src");
        let ignore_dirs = vec!["temp".to_string()];
        assert!(path.is_dir(), "The path should point to a valid directory");
        assert!(!is_invalid_path(path, &ignore_dirs));
    }
}
