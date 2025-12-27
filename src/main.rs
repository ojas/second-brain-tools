// extern crate serde_json;

use clap::{Parser, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;


// fn dump_tree(node: &Value, indent: usize) {
//     if let Some(name) = node.get("name").and_then(|n| n.as_str()) {
//         println!("{}{}", " ".repeat(indent), name);
//     }
//     if let Some(contents) = node.get("contents").and_then(|c| c.as_array()) {
//         for child in contents {
//             dump_tree(child, indent + 2);
//         }
//     }
// }

// CLI Data Structures

#[derive(Parser)]
#[command(name = "second-brain-tools")]
#[command(about = "A CLI tool for managing a second brain file system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse tree JSON and output TSV format (path, size, timestamp)
    Tree {
        /// Path to the tree JSON file (generated with: tree -J --du -D --timefmt "%Y-%m-%d")
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Generate and sync Netscape-style bookmark index files for folders
    Bookmarks {
        /// Path to the folder to generate bookmarks for
        #[arg(value_name = "FOLDER")]
        folder: PathBuf,

        /// Name of the index file (default: index.html)
        #[arg(short, long, default_value = "index.html")]
        index: String,

        /// Include subdirectories recursively
        #[arg(short, long)]
        recursive: bool,
    },
}

// Data Structures

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
    phones: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct TreeList {
    node: TreeNode,
    report: ReportNode,
}

#[derive(Serialize, Deserialize, Debug)]
struct TreeNode {
    // rename "type" to node_type because type is a reserved keyword
    #[serde(rename = "type")]
    node_type: String,
    name: String,
    size: u64,
    time: String,
    contents: Option<Vec<TreeNode>>,
}

#[derive(Serialize, Deserialize)]
struct ReportNode {
    #[serde(rename = "type")]
    node_type: String,
    size: u64,
    directories: u64,
    files: u64,
}

#[test]
fn test_textbook_example() -> Result<(), Box<dyn Error>> {
    // Some JSON input data as a &str. Maybe this comes from the user.
    let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;

    // Parse the string of data into a Person object. This is exactly the
    // same function as the one that produced serde_json::Value above, but
    // now we are asking it for a Person as output.
    let p: Person = serde_json::from_str(data)?;

    // Do things just like with any other Rust data structure.
    println!("Please call {} at the number {}", p.name, p.phones[0]);

    Ok(())
}

#[test]
fn test_typed_example() {
    let data = r#"
    [
      {"type":"link","name":"/Users/ojas/GDrive","size":99984167157,"time":"2023-08-23","contents":[
        {"type":"file","name":"2021-06 IG Posts.gsheet","size":168,"time":"2021-07-07"},
        {"type":"file","name":"2023 Cashflow.gsheet","size":168,"time":"2023-12-04"},
        {"type":"directory","name":"Archive","size":577632,"time":"2019-02-08","contents":[
          {"type":"file","name":"2008 RX-7 Weight.gsheet","size":168,"time":"2019-11-12"},
          {"type":"file","name":"XÚÖ West.gdoc","size":168,"time":"2018-05-08"},
          {"type":"file","name":"XÚÖ.gsite","size":157,"time":"2018-05-10"}
        ]}
      ]}
    ,
      {"type":"report","size":99984167157,"directories":909,"files":29484}
    ]"#;

    // let json_value: Value = serde_json::from_str(data).expect("JSON was not well-formatted");

    // we want to get back a variable that has two elements; the first is a Node, the second is a Report,
    // not using json_value, but using strongly typed structs
    // so we can access fields directly. Let's go.

   let tree: TreeList = serde_json::from_str(data).expect("JSON was not well-formatted");

   // https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/testing.html
   assert_eq!(tree.node.name, "/Users/ojas/GDrive");
   assert_eq!(tree.report.files, 29484);

  // let nodes: Vec<Node> = serde_json::from_str(data).expect("JSON was not well-formatted");
//   println!("{:#?}", tree.report.node_type);
}

fn read_tree_from_file<P: AsRef<Path>>(path: P) -> Result<TreeList,Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `TreeList`.
    let tree: TreeList = serde_json::from_reader(reader)?;

    // Return the `TreeList`.
    Ok(tree)
}

fn walk_tree(node: &TreeNode, indent: usize) {
    println!("{}{}", " ".repeat(indent), node.name);
    if let Some(contents) = &node.contents {
        for child in contents {
            walk_tree(child, indent + 2);
        }
    }
}

fn walk_tree_fullpath(node: &TreeNode, prefix: &str, indent: usize) {
    let _ = indent; // suppress unused variable warning
    let full_path = format!("{}{}", prefix, &node.name);
    println!("{}\t{}\t{}", full_path, node.size, node.time);
    if let Some(contents) = &node.contents {
        for child in contents {
            walk_tree_fullpath(child, &format!("{}/", full_path), indent + 2);
        }
    }
}

fn handle_tree_command(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let tree: TreeList = read_tree_from_file(path)?;
    //walk_tree(&tree.node, 0);
    walk_tree_fullpath(&tree.node, "", 0);

    println!("{:#?}", tree.report.files);

//    let data = fs::read_to_string(filename).expect("Unable to read file");
    // let json_value: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
    // println!("{:#}", json_value);

    // dump_tree(&json_value[0], 0);
    Ok(())
}

// Bookmarks functionality

#[derive(Debug, Clone)]
struct BookmarkEntry {
    name: String,
    href: String,
    add_date: u64,
    last_modified: u64,
    description: Option<String>,
}

#[derive(Debug, Clone)]
struct BookmarkFolder {
    name: String,
    last_modified: u64,
    entries: Vec<BookmarkItem>,
}

#[derive(Debug, Clone)]
enum BookmarkItem {
    Link(BookmarkEntry),
    Folder(BookmarkFolder),
}

fn system_time_to_unix_timestamp(time: SystemTime) -> u64 {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn get_file_metadata(path: &Path) -> Result<(u64, u64), Box<dyn Error>> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    let created = metadata.created().unwrap_or(modified);

    Ok((
        system_time_to_unix_timestamp(created),
        system_time_to_unix_timestamp(modified),
    ))
}

fn scan_directory(
    dir_path: &Path,
    index_filename: &str,
    recursive: bool,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>), Box<dyn Error>> {
    let mut files = Vec::new();
    let mut dirs = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Skip the index file itself
        if file_name_str == index_filename {
            continue;
        }

        // Skip hidden files (starting with .)
        if file_name_str.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            if recursive {
                dirs.push(path);
            }
        } else if path.is_file() {
            files.push(path);
        }
    }

    // Sort alphabetically
    files.sort();
    dirs.sort();

    Ok((files, dirs))
}

fn parse_existing_bookmarks(content: &str) -> Vec<BookmarkItem> {
    let mut items = Vec::new();

    // Find the main DL block
    let main_dl_re = Regex::new(r"(?s)<H1>.*?</H1>\s*<DL><p>(.*)</DL><p>\s*$").unwrap();
    let main_content = match main_dl_re.captures(content) {
        Some(cap) => cap.get(1).map(|m| m.as_str()).unwrap_or(""),
        None => return items,
    };

    // Parse top-level entries
    let mut pos = 0;
    let lines: Vec<&str> = main_content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Check for link entry
        if line.starts_with("<DT><A ") {
            let link_re = Regex::new(
                r#"<DT><A\s+HREF="([^"]+)"(?:\s+ADD_DATE="(\d+)")?(?:\s+LAST_MODIFIED="(\d+)")?>([^<]+)</A>"#
            ).unwrap();

            if let Some(cap) = link_re.captures(line) {
                let mut entry = BookmarkEntry {
                    href: cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
                    add_date: cap.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0),
                    last_modified: cap.get(3).and_then(|m| m.as_str().parse().ok()).unwrap_or(0),
                    name: cap.get(4).map(|m| m.as_str().to_string()).unwrap_or_default(),
                    description: None,
                };

                // Check next line for description
                if i + 1 < lines.len() && lines[i + 1].trim().starts_with("<DD>") {
                    let desc = lines[i + 1].trim().strip_prefix("<DD>").unwrap_or("").trim();
                    entry.description = Some(desc.to_string());
                    i += 1;
                }

                items.push(BookmarkItem::Link(entry));
            }
        }
        // Check for folder entry
        else if line.starts_with("<DT><H3") {
            let folder_re = Regex::new(
                r#"<DT><H3(?:\s+ADD_DATE="(\d+)")?(?:\s+LAST_MODIFIED="(\d+)")?>([^<]+)</H3>"#
            ).unwrap();

            if let Some(cap) = folder_re.captures(line) {
                let folder_name = cap.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();
                let last_modified = cap.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);

                // Find the nested DL block for this folder
                let mut folder_entries = Vec::new();
                i += 1;

                if i < lines.len() && lines[i].trim() == "<DL><p>" {
                    i += 1;

                    // Parse folder contents until we hit </DL>
                    while i < lines.len() && !lines[i].trim().starts_with("</DL>") {
                        let folder_line = lines[i].trim();

                        if folder_line.starts_with("<DT><A ") {
                            let link_re = Regex::new(
                                r#"<DT><A\s+HREF="([^"]+)"(?:\s+ADD_DATE="(\d+)")?(?:\s+LAST_MODIFIED="(\d+)")?>([^<]+)</A>"#
                            ).unwrap();

                            if let Some(cap) = link_re.captures(folder_line) {
                                let mut entry = BookmarkEntry {
                                    href: cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
                                    add_date: cap.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0),
                                    last_modified: cap.get(3).and_then(|m| m.as_str().parse().ok()).unwrap_or(0),
                                    name: cap.get(4).map(|m| m.as_str().to_string()).unwrap_or_default(),
                                    description: None,
                                };

                                // Check next line for description
                                if i + 1 < lines.len() && lines[i + 1].trim().starts_with("<DD>") {
                                    let desc = lines[i + 1].trim().strip_prefix("<DD>").unwrap_or("").trim();
                                    entry.description = Some(desc.to_string());
                                    i += 1;
                                }

                                folder_entries.push(BookmarkItem::Link(entry));
                            }
                        }

                        i += 1;
                    }
                }

                items.push(BookmarkItem::Folder(BookmarkFolder {
                    name: folder_name,
                    last_modified,
                    entries: folder_entries,
                }));
            }
        }

        i += 1;
    }

    items
}

fn generate_bookmark_html(
    folder_name: &str,
    items: &[BookmarkItem],
) -> String {
    let mut html = String::new();

    html.push_str("<!DOCTYPE NETSCAPE-Bookmark-file-1>\n");
    html.push_str("<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n");
    html.push_str("<!-- This is an automatically generated file. It will be read and modified by automated tools. Edit only if you understand the risks -->\n");
    html.push_str("<TITLE>Bookmarks</TITLE>\n");
    html.push_str(&format!("<H1>{}</H1>\n", folder_name));
    html.push_str("<DL><p>\n");

    for item in items {
        match item {
            BookmarkItem::Link(entry) => {
                html.push_str(&format!(
                    "    <DT><A HREF=\"{}\" ADD_DATE=\"{}\" LAST_MODIFIED=\"{}\">{}</A>\n",
                    entry.href, entry.add_date, entry.last_modified, entry.name
                ));
                if let Some(desc) = &entry.description {
                    html.push_str(&format!("    <DD>{}\n", desc));
                }
            }
            BookmarkItem::Folder(folder) => {
                html.push_str(&format!(
                    "    <DT><H3 LAST_MODIFIED=\"{}\">{}</H3>\n",
                    folder.last_modified, folder.name
                ));
                html.push_str("    <DL><p>\n");
                for sub_item in &folder.entries {
                    if let BookmarkItem::Link(entry) = sub_item {
                        html.push_str(&format!(
                            "        <DT><A HREF=\"{}\" ADD_DATE=\"{}\" LAST_MODIFIED=\"{}\">{}</A>\n",
                            entry.href, entry.add_date, entry.last_modified, entry.name
                        ));
                        if let Some(desc) = &entry.description {
                            html.push_str(&format!("        <DD>{}\n", desc));
                        }
                    }
                }
                html.push_str("    </DL><p>\n");
            }
        }
    }

    html.push_str("</DL><p>\n");
    html
}

fn merge_bookmarks(
    existing: Vec<BookmarkItem>,
    files: &[PathBuf],
    dirs: &[(PathBuf, Vec<PathBuf>)],
    base_path: &Path,
) -> Result<Vec<BookmarkItem>, Box<dyn Error>> {
    let mut items = Vec::new();
    let mut existing_file_hrefs = HashSet::new();
    let mut existing_folder_names: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    // Collect existing entries and preserve them
    for (idx, item) in existing.into_iter().enumerate() {
        match &item {
            BookmarkItem::Link(entry) => {
                existing_file_hrefs.insert(entry.href.clone());
            }
            BookmarkItem::Folder(folder) => {
                existing_folder_names.insert(folder.name.clone(), items.len());
            }
        }
        items.push(item);
    }

    // Add new file entries
    for file_path in files {
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let encoded_name = urlencoding::encode(file_name);

        if !existing_file_hrefs.contains(encoded_name.as_ref()) {
            let (add_date, last_modified) = get_file_metadata(file_path)?;
            items.push(BookmarkItem::Link(BookmarkEntry {
                name: file_name.to_string(),
                href: encoded_name.to_string(),
                add_date,
                last_modified,
                description: None,
            }));
        }
    }

    // Add or update directory entries
    for (dir_path, dir_files) in dirs {
        let dir_name = dir_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if let Some(&folder_idx) = existing_folder_names.get(dir_name) {
            // Folder exists, update its entries
            if let Some(BookmarkItem::Folder(ref mut folder)) = items.get_mut(folder_idx) {
                // Get existing hrefs in this folder
                let mut existing_folder_hrefs = HashSet::new();
                for entry in &folder.entries {
                    if let BookmarkItem::Link(link) = entry {
                        existing_folder_hrefs.insert(link.href.clone());
                    }
                }

                // Add new files to this folder
                for file_path in dir_files {
                    let file_name = file_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    let relative_path = file_path.strip_prefix(base_path)
                        .unwrap_or(file_path);
                    let encoded_path = relative_path
                        .to_str()
                        .map(|s| urlencoding::encode(s).to_string())
                        .unwrap_or_default();

                    if !existing_folder_hrefs.contains(&encoded_path) {
                        let (add_date, file_last_modified) = get_file_metadata(file_path)?;
                        folder.entries.push(BookmarkItem::Link(BookmarkEntry {
                            name: file_name.to_string(),
                            href: encoded_path,
                            add_date,
                            last_modified: file_last_modified,
                            description: None,
                        }));
                    }
                }
            }
        } else {
            // New folder, create it
            let (_, last_modified) = get_file_metadata(dir_path)?;
            let mut folder_entries = Vec::new();

            for file_path in dir_files {
                let file_name = file_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                let relative_path = file_path.strip_prefix(base_path)
                    .unwrap_or(file_path);
                let encoded_path = relative_path
                    .to_str()
                    .map(|s| urlencoding::encode(s).to_string())
                    .unwrap_or_default();

                let (add_date, file_last_modified) = get_file_metadata(file_path)?;
                folder_entries.push(BookmarkItem::Link(BookmarkEntry {
                    name: file_name.to_string(),
                    href: encoded_path,
                    add_date,
                    last_modified: file_last_modified,
                    description: None,
                }));
            }

            items.push(BookmarkItem::Folder(BookmarkFolder {
                name: dir_name.to_string(),
                last_modified,
                entries: folder_entries,
            }));
        }
    }

    Ok(items)
}

fn handle_bookmarks_command(
    folder: &PathBuf,
    index_name: &str,
    recursive: bool,
) -> Result<(), Box<dyn Error>> {
    // Validate folder exists
    if !folder.is_dir() {
        return Err(format!("Path is not a directory: {}", folder.display()).into());
    }

    let index_path = folder.join(index_name);

    // Read existing bookmarks if file exists
    let existing_content = if index_path.exists() {
        fs::read_to_string(&index_path)?
    } else {
        String::new()
    };

    let existing_items = if !existing_content.is_empty() {
        parse_existing_bookmarks(&existing_content)
    } else {
        Vec::new()
    };

    // Scan directory for files and subdirectories
    let (files, dirs) = scan_directory(folder, index_name, recursive)?;

    // Process subdirectories if recursive
    let mut dir_contents = Vec::new();
    if recursive {
        for dir in &dirs {
            let (dir_files, _) = scan_directory(dir, index_name, false)?;
            dir_contents.push((dir.clone(), dir_files));
        }
    }

    // Merge existing bookmarks with filesystem
    let merged_items = merge_bookmarks(existing_items, &files, &dir_contents, folder)?;

    // Generate HTML
    let folder_name = folder
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Bookmarks Menu");
    let html = generate_bookmark_html(folder_name, &merged_items);

    // Write to file
    let mut file = File::create(&index_path)?;
    file.write_all(html.as_bytes())?;

    println!("Bookmark index generated: {}", index_path.display());
    println!("Total entries: {}", merged_items.len());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Tree { file } => handle_tree_command(file)?,
        Commands::Bookmarks { folder, index, recursive } => {
            handle_bookmarks_command(folder, index, *recursive)?
        }
    }

    Ok(())
}