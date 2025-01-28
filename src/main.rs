use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, DirEntry, File};
use std::io::{self, Read};
use std::path::Path;
use markdown_meta_parser::MetaData;


const NODES_DIR: &str = "/home/stroby/Notes/";


// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with(".trash") || path.ends_with(".obsidian") || path.ends_with("Sources") {
                    continue;
                }

                visit_dirs(&path)?;
            } else {
                parse_tags(&entry);
            }
        }
    }
    Ok(())
}

fn parse_tags(entry: &DirEntry) {
    // println!("Processing entry: {:?}", entry);
    
    let path = entry.path();
    let file = File::open(&path);

    println!("{}", path.to_str().unwrap());

    match file {
        Ok(mut file) => {
            let extension = path.extension();

            match extension {
                Some(str) => {
                    if str != "md" {
                        return;
                    }
                }
                None => {
                    println!("Cant read extension: {:?}", path);
                }
            }

            let mut buffer = Vec::new();
            let content = file.read_to_end(&mut buffer);
            match content {
                Ok(sz) => {
                    //println!("  got {} bytes", sz);

                    let content = String::from_utf8(buffer).expect("Bytes should be valid utf8");
                                
                    let tags_index = content.find("tags:");
                    if let Some(tags_index) = tags_index {
                        let line_end = content[tags_index..].find("\n");

                        if let Some(line_end) = line_end {
                            let mut line_end = line_end + tags_index;

                            let tag_line = &content[(tags_index + 5)..line_end];
                                
                            //println!("Tag line: {:?}", tag_line);

                            let mut tags: Vec<&str> = tag_line.split([' ', ',']).filter(|tag| (*tag != "" && *tag != ":")).collect();
                                
                            if tags.is_empty() {

                                // Search for tags:
                                // - tag
                                // - sagfas
                                // - saf
                                if &content[tags_index..(tags_index + 5)] == "tags:" {
                                    let next_line_end_option = content[(line_end + 1)..].find("\n");
                                    
                                    if let Some(next_line_end) = next_line_end_option {
                                        let mut next_line_end = next_line_end + line_end + 1;

                                        loop {
                                            let minus_index = content[(line_end + 1)..next_line_end].find("-");

                                            if let Some(minus_index) = minus_index {
                                                let minus_index = minus_index + line_end + 1;

                                                tags.push(&content[(minus_index + 2)..next_line_end]);        
                                                
                                                let next_line_end_option = content[(next_line_end + 1)..].find("\n");
                                                if next_line_end_option.is_some() {
                                                    line_end = next_line_end;
                                                    next_line_end = next_line_end_option.unwrap() + next_line_end + 1;
                                                } else {
                                                    break;
                                                }
                                            } else {
                                                break;
                                            }
                                        }
                                        
                                    } else {
                                        //println!("No next line after tag: {:?}", path);
                                    }
                                }
                            }

                            let mut owned_tags = vec![];
                            for tag in tags {
                                let tag: String = tag.chars().filter(|c| (*c != '#' && *c != '\"' && *c != '[' && *c != ']')).collect();

                                println!("Tag: {:?}", tag);

                                owned_tags.push(tag);
                            }

                            if !owned_tags.is_empty() {
                                copy_file(entry, content, owned_tags);  
                            } else {
                                println!("No Tags: {:?}", path)
                            }

                        } else {  
                            println!("Tags found but not return statement: {:?}", path)
                        }
                    } else {
                        println!("No Tags: {:?}", path)
                    }
                }
                    Err(e) => {
                        println!("Read error: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("Open error: {:?}", e);
            }
        }
     
}

fn copy_file(entry: &DirEntry, content: String, tags: Vec<String>) {
    let destinations = vec![
        (["uni"], "/home/stroby/dev/obsidian_export/uni_notes", "uni MOC.md")
    ];

    for (filter_tags, destination, index_name) in destinations {
        let mut found = false;
        for tag in tags.iter() {
            for filter_tag in filter_tags.iter() {
                if tag.contains(filter_tag) {
                    found = true;
                    break;
                }
            }
        }

        if !found {
            continue;
        }

        let mut path = Path::new(destination);
        if !path.exists() {
            fs::create_dir_all(destination).unwrap();
        }
        
        let filename_os_string = entry.file_name(); 
        let mut filename = filename_os_string.to_str().expect(&format!("Cant parse filename: {:?}", path));
        if filename == index_name {
            filename = "index.md";
        }

        let path = path.join(filename);
        fs::write(path.to_owned(), content.to_owned()).expect(&format!("Unable to write file: {:?}", path));
    }
    

}

fn main() {
    let path = Path::new(NODES_DIR);
    
    visit_dirs(path);
}
