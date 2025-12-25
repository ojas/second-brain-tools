// extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;


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

fn serde_json_dump() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let path = &args[1];

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

// i'm missing external create for serde_json - help me fix

fn main() -> Result<(), Box<dyn Error>> {
    // get filename from args - there's just one arg
    // use serde_json to read the file into a serde_json::Value
    // the JSON structure looks like this;
    //
    // ```json
    
    // ```
    // there are always two top-level elements; the first is the root directory,
    // the second is a report summary
    // we want to recursively walk the tree starting from the first element
    // and print out the names of all files and directories

    // typed_example();
    // test_example();
    // serde_json_dump();
    // let args: Vec<String> = env::args().collect();
    // if args.len() != 2 {
    //     eprintln!("Usage: {} <filename>", args[0]);
    //     std::process::exit(1);
    // }
    // let filename = &args[1];

    serde_json_dump()?;
    
    Ok(())
}