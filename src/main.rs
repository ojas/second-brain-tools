// extern crate serde_json;

use clap::{Parser, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
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
    /// Process photo albums: copy bookmarked files and generate resized versions
    Pixie {
        /// Path to pixie.yaml config file (default: ./pixie.yaml)
        #[arg(short, long, default_value = "pixie.yaml")]
        config: PathBuf,
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

// Pixie Data Structures

#[derive(Serialize, Deserialize, Debug)]
struct PixieConfig {
    input_folder: String,
    #[serde(default)]
    folder_depth: Option<u32>,  // 0-64, None means unlimited
    output_folder: String,
    index_file_name: String,
    resize_args: HashMap<String, String>,  // e.g. {"rs": "800x800>", "thumb": "200x200"}
    #[serde(default)]
    index_transform: String,  // unused for now
}

#[derive(Debug)]
struct AlbumFolder {
    path: PathBuf,
    index_path: PathBuf,
    album_name: String,  // Just the folder name
    depth: u32,
}

// Index YAML/JSON data structures

#[derive(Serialize, Deserialize, Debug)]
struct AlbumIndex {
    title: String,
    photos: Vec<PhotoInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PhotoInfo {
    filename: String,
    w: Option<u32>,
    h: Option<u32>,
    caption: Option<String>,
    sizes: HashMap<String, SizeInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SizeInfo {
    filename: String,
    w: Option<u32>,
    h: Option<u32>,
}

#[derive(Debug, Clone)]
struct BookmarkFile {
    href: String,
    name: String,
    caption: Option<String>,
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

fn encode_path_preserving_slashes(path: &Path) -> String {
    path.components()
        .map(|component| {
            let s = component.as_os_str().to_string_lossy();
            urlencoding::encode(&s).to_string()
        })
        .collect::<Vec<_>>()
        .join("/")
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

fn extract_title_from_bookmarks(content: &str) -> String {
    let title_re = Regex::new(r"<H1>(.*?)</H1>").unwrap();
    if let Some(cap) = title_re.captures(content) {
        cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_else(|| "Untitled".to_string())
    } else {
        "Untitled".to_string()
    }
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
                    let encoded_path = encode_path_preserving_slashes(relative_path);

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
                let encoded_path = encode_path_preserving_slashes(relative_path);

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

// Pixie functionality

fn log_command(cmd: &Command) {
    let program = cmd.get_program().to_string_lossy();
    let args: Vec<String> = cmd.get_args()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect();

    if args.is_empty() {
        eprintln!("[CMD] {}", program);
    } else {
        eprintln!("[CMD] {} {}", program, args.join(" "));
    }
}

fn expand_tilde_path(path: &str) -> Result<PathBuf, Box<dyn Error>> {
    let expanded = shellexpand::tilde(path);
    Ok(PathBuf::from(expanded.as_ref()))
}

fn read_pixie_config(path: &Path) -> Result<PixieConfig, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let config: PixieConfig = serde_yaml::from_str(&content)?;

    // Validate folder_depth
    if let Some(depth) = config.folder_depth {
        if depth > 64 {
            return Err(format!("folder_depth must be between 0 and 64, got {}", depth).into());
        }
    }

    Ok(config)
}

fn find_album_folders(
    input_folder: &Path,
    index_filename: &str,
    max_depth: Option<u32>
) -> Result<Vec<AlbumFolder>, Box<dyn Error>> {
    let mut albums = Vec::new();
    let mut queue = VecDeque::new();

    // Start BFS with input folder at depth 0
    queue.push_back((input_folder.to_path_buf(), 0u32));

    while let Some((current_path, current_depth)) = queue.pop_front() {
        // Check if this folder contains the index file
        let index_path = current_path.join(index_filename);
        if index_path.exists() && index_path.is_file() {
            let album_name = current_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            albums.push(AlbumFolder {
                path: current_path.clone(),
                index_path,
                album_name,
                depth: current_depth,
            });
        }

        // Continue traversing if within depth limit
        if max_depth.is_none() || current_depth < max_depth.unwrap() {
            // Read subdirectories
            if let Ok(entries) = fs::read_dir(&current_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_dir() {
                            // Skip hidden directories
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                if !name.starts_with('.') {
                                    queue.push_back((path, current_depth + 1));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(albums)
}

fn extract_bookmark_files(items: &[BookmarkItem]) -> Vec<BookmarkFile> {
    let mut files = Vec::new();

    for item in items {
        match item {
            BookmarkItem::Link(entry) => {
                files.push(BookmarkFile {
                    href: entry.href.clone(),
                    name: entry.name.clone(),
                    caption: entry.description.clone(),
                });
            }
            BookmarkItem::Folder(folder) => {
                // Recursively extract from folder
                files.extend(extract_bookmark_files(&folder.entries));
            }
        }
    }

    files
}

fn run_imagemagick_resize(
    file_path: &Path,
    suffix: &str,
    resize_spec: &str
) -> Result<(), Box<dyn Error>> {
    // Build the ImageMagick command
    // magick input.jpg -resize "{resize_spec}" -set filename:f "%t.{suffix}.%e" "%[filename:f]"
    // The %e in filename:f gets evaluated to the extension, then %[filename:f] outputs that value
    let filename_pattern = format!("%t.{}.%e", suffix);
    let output_pattern = "%[filename:f]";

    // Get the directory where the file is located to set as working directory
    let file_dir = file_path.parent()
        .ok_or("Failed to get parent directory of file")?;

    // Get just the filename to pass to ImageMagick (since we're setting current_dir)
    let filename = file_path.file_name()
        .ok_or("Failed to get filename")?;

    let mut cmd = Command::new("magick");
    cmd.arg(filename)  // Use just the filename since current_dir is set
        .arg("-auto-orient")  // Rotate pixels to match EXIF, strip orientation tag
        .arg("-resize")
        .arg(&resize_spec)
        .arg("-set")
        .arg("filename:f")
        .arg(&filename_pattern)
        .arg(&output_pattern)
        .current_dir(file_dir);  // Set working directory to output folder

    log_command(&cmd);
    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ImageMagick resize failed for {}: {}", file_path.display(), stderr).into());
    }

    Ok(())
}

fn copy_and_resize_files(
    bookmark_files: &[BookmarkFile],
    source_folder: &Path,
    dest_folder: &Path,
    resize_args: &HashMap<String, String>
) -> Result<(), Box<dyn Error>> {
    // Create destination folder
    fs::create_dir_all(dest_folder)?;

    let mut copied_count = 0;
    let mut copied_files = Vec::new();

    // Copy each file
    for file in bookmark_files {
        // URL-decode the href
        let decoded = match urlencoding::decode(&file.href) {
            Ok(s) => s.to_string(),
            Err(e) => {
                eprintln!("Warning: Failed to decode '{}': {}", file.href, e);
                continue;
            }
        };

        let source_path = source_folder.join(&decoded);

        // Extract just the filename for destination
        let filename = source_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&decoded);
        let dest_path = dest_folder.join(filename);

        // Copy file
        match fs::copy(&source_path, &dest_path) {
            Ok(_) => {
                copied_count += 1;
                copied_files.push(dest_path);
            }
            Err(e) => {
                eprintln!("Warning: Failed to copy '{}': {}", source_path.display(), e);
            }
        }
    }

    println!("  Copied {} files", copied_count);

    // Run ImageMagick resize operations on each copied file
    if !resize_args.is_empty() && !copied_files.is_empty() {
        for (suffix, resize_spec) in resize_args {
            let mut resize_count = 0;
            for file_path in &copied_files {
                match run_imagemagick_resize(file_path, suffix, resize_spec) {
                    Ok(_) => {
                        resize_count += 1;
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to create '{}' version of '{}': {}",
                                  suffix, file_path.display(), e);
                    }
                }
            }
            println!("  Created {} '{}' resized versions", resize_count, suffix);
        }
    }

    Ok(())
}

fn get_image_dimensions(folder: &Path) -> Result<HashMap<String, (u32, u32)>, Box<dyn Error>> {
    // Run: magick identify -format "%f,%w,%h\n" folder/*
    let pattern = folder.join("*");
    let pattern_str = pattern.to_str()
        .ok_or("Failed to convert path to string")?;

    let mut cmd = Command::new("magick");
    cmd.arg("identify")
        .arg("-format")
        .arg("%f,%w,%h\\n")
        .arg(pattern_str);

    log_command(&cmd);
    let output = cmd.output()?;

    // Parse stdout regardless of exit code - ImageMagick may partially succeed
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut dimensions = HashMap::new();

    // Parse CSV output: filename,width,height
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 3 {
            if let (Some(filename), Some(width), Some(height)) = (
                parts.get(0),
                parts.get(1).and_then(|s| s.parse::<u32>().ok()),
                parts.get(2).and_then(|s| s.parse::<u32>().ok()),
            ) {
                dimensions.insert(filename.to_string(), (width, height));
            }
        }
    }

    // Log warnings if there were errors, but don't fail if we got some results
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            eprintln!("Warning: magick identify had errors (but may have partial results): {}", stderr);
        }

        // Only fail if we got no results at all
        if dimensions.is_empty() {
            return Err(format!("magick identify failed to process any files: {}", stderr).into());
        }
    }

    Ok(dimensions)
}

fn build_album_index(
    title: String,
    bookmark_files: &[BookmarkFile],
    dimensions: &HashMap<String, (u32, u32)>,
    resize_args: &HashMap<String, String>,
) -> AlbumIndex {
    // Note on orientation handling:
    // - Original files: Copied as-is, preserve EXIF orientation data
    //   Dimensions reported are physical (may not match visual if rotated)
    // - Resized files: Auto-oriented during resize (-auto-orient flag)
    //   Pixels are rotated to match EXIF, orientation tag is stripped
    //   Dimensions reported match visual appearance
    // This ensures resized images display correctly on the web while preserving originals

    let mut photos = Vec::new();

    for file in bookmark_files {
        // Decode href to get the actual filename
        let decoded = urlencoding::decode(&file.href)
            .map(|s| s.to_string())
            .unwrap_or_else(|_| file.href.clone());

        // Extract just the filename
        let filename = Path::new(&decoded)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&decoded)
            .to_string();

        // Get dimensions for original file
        let (w, h) = dimensions.get(&filename)
            .map(|(w, h)| (Some(*w), Some(*h)))
            .unwrap_or((None, None));

        // Build sizes map
        let mut sizes = HashMap::new();
        for (suffix, _) in resize_args {
            let resized_filename = format!(
                "{}.{}.{}",
                Path::new(&filename).file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(""),
                suffix,
                Path::new(&filename).extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
            );

            let (size_w, size_h) = dimensions.get(&resized_filename)
                .map(|(w, h)| (Some(*w), Some(*h)))
                .unwrap_or((None, None));

            sizes.insert(suffix.clone(), SizeInfo {
                filename: resized_filename,
                w: size_w,
                h: size_h,
            });
        }

        photos.push(PhotoInfo {
            filename,
            w,
            h,
            caption: file.caption.clone(),
            sizes,
        });
    }

    AlbumIndex { title, photos }
}

fn process_album(
    album: &AlbumFolder,
    config: &PixieConfig
) -> Result<(), Box<dyn Error>> {
    println!("Processing album: {} (depth {})", album.album_name, album.depth);

    // Read and parse index file
    let index_content = fs::read_to_string(&album.index_path)?;

    // Extract title from H1
    let title = extract_title_from_bookmarks(&index_content);

    let bookmark_items = parse_existing_bookmarks(&index_content);

    // Extract all file hrefs with captions
    let bookmark_files = extract_bookmark_files(&bookmark_items);
    println!("  Found {} files in bookmarks", bookmark_files.len());

    if bookmark_files.is_empty() {
        println!("  Skipping: no files to process");
        return Ok(());
    }

    // Create output folder using just the album name
    let output_folder_path = expand_tilde_path(&config.output_folder)?;
    let output_album_path = output_folder_path.join(&album.album_name);

    // Copy files and create resized versions
    copy_and_resize_files(&bookmark_files, &album.path, &output_album_path, &config.resize_args)?;

    // Copy index file to output
    let output_index_path = output_album_path.join(&config.index_file_name);
    fs::copy(&album.index_path, &output_index_path)?;
    println!("  Copied index file");

    // Get image dimensions for all files in the output folder
    println!("  Getting image dimensions...");
    let dimensions = match get_image_dimensions(&output_album_path) {
        Ok(dims) => dims,
        Err(e) => {
            eprintln!("Warning: Failed to get image dimensions: {}", e);
            HashMap::new()
        }
    };

    // Build album index
    let album_index = build_album_index(title, &bookmark_files, &dimensions, &config.resize_args);

    // Serialize to YAML
    let yaml_content = serde_yaml::to_string(&album_index)?;
    let yaml_path = output_album_path.join("index.yaml");
    fs::write(&yaml_path, yaml_content)?;
    println!("  Generated index.yaml with {} photos", album_index.photos.len());

    Ok(())
}

fn handle_pixie_command(config_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    // Check if ImageMagick is available
    let mut version_cmd = Command::new("magick");
    version_cmd.arg("--version");

    log_command(&version_cmd);
    match version_cmd.output() {
        Ok(output) if output.status.success() => {
            // ImageMagick is available
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ImageMagick check failed: {}", stderr).into());
        }
        Err(e) => {
            return Err(format!("ImageMagick not found: {}. Please install ImageMagick to use the pixie command.", e).into());
        }
    }

    // Load config
    println!("Loading config from: {}", config_path.display());
    let config = read_pixie_config(config_path)?;

    // Expand paths
    let input_folder = expand_tilde_path(&config.input_folder)?;

    // Validate input folder exists
    if !input_folder.exists() || !input_folder.is_dir() {
        return Err(format!("Input folder does not exist: {}", input_folder.display()).into());
    }

    // Find all album folders
    println!("Searching for albums in: {}", input_folder.display());
    let albums = find_album_folders(&input_folder, &config.index_file_name, config.folder_depth)?;

    println!("Found {} album folders\n", albums.len());

    if albums.is_empty() {
        println!("No albums found with index file '{}'", config.index_file_name);
        return Ok(());
    }

    // Process each album
    let mut success_count = 0;
    let mut failure_count = 0;

    for album in &albums {
        match process_album(album, &config) {
            Ok(_) => {
                success_count += 1;
            }
            Err(e) => {
                eprintln!("Error processing album '{}': {}", album.album_name, e);
                failure_count += 1;
            }
        }
        println!();  // Blank line between albums
    }

    // Print summary
    println!("Summary:");
    println!("  Total albums: {}", albums.len());
    println!("  Successful: {}", success_count);
    println!("  Failed: {}", failure_count);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Tree { file } => handle_tree_command(file)?,
        Commands::Bookmarks { folder, index, recursive } => {
            handle_bookmarks_command(folder, index, *recursive)?
        }
        Commands::Pixie { config } => handle_pixie_command(config)?,
    }

    Ok(())
}