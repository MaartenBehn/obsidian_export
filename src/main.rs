use std::fs::{self, DirEntry, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use markdown_meta_parser::MetaData;

fn main() {

    let t = ExportTask {
        tags: vec!["uni".to_string()], 
        notes: "/home/stroby/Notes".to_string(), 
        exclude_notes_folders: vec![".trash".to_string(), ".obsidian".to_string(), "Sources".to_string()],
        source: "/home/stroby/Notes/Sources/".to_string(), 
        destination: "/home/stroby/dev/obsidian_export/quartz/content/".to_string(), 
        destination_source: "/home/stroby/dev/obsidian_export/quartz/content/attachments".to_string(), 
        index_file: "uni Index.md".to_string(),
    };

    let path = Path::new(&t.destination);
    if !path.exists() {
        fs::create_dir_all(&t.destination).unwrap();
    } else {
        fs::remove_dir_all(path).unwrap();
        fs::create_dir(path).unwrap();
    }

    let path = Path::new(&t.destination_source);
    fs::create_dir(path).unwrap();
        
    let path = Path::new(&t.notes);
    let res = search_notes(&t, path);

    if res.is_err() {
        println!("Err: {}", res.unwrap_err());
    }
}

fn search_notes(t: &ExportTask, dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if t.exclude_notes_folders.iter().any(|f| { path.ends_with(f) }) { 
                    continue;
                }

                search_notes(t, &path)?;
            } else {
                parse_tags(t, &entry)?;
            }
        }
    }
    Ok(())
}

fn parse_tags(t: &ExportTask, entry: &DirEntry) -> io::Result<()> {
    // println!("Processing entry: {:?}", entry);
    
    let path = entry.path();
    let file = File::open(&path);

    // println!("Parsing File: {}", path.to_str().unwrap());

    match file {
        Ok(mut file) => {
            let extension = path.extension();

            match extension {
                Some(str) => {
                    if str != "md" {
                        return Ok(());
                    }
                }
                None => {
                    println!("Cant read extension: {:?}", path);
                }
            }

            let mut buffer = Vec::new();
            let content = file.read_to_end(&mut buffer);
            match content {
                Ok(_) => {
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

                                // println!("Tag: {:?}", tag);

                                owned_tags.push(tag);
                            }

                            if !owned_tags.is_empty() {
                                copy_file(t, entry, content, &path, owned_tags)?;  
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

    Ok(()) 
}

pub struct ExportTask {
    tags: Vec<String>,
    notes: String,
    exclude_notes_folders: Vec<String>,
    source: String,
    destination: String,
    destination_source: String,
    index_file: String,
}

fn copy_file(t: &ExportTask, entry: &DirEntry, content: String, path: &PathBuf, tags: Vec<String>) -> io::Result<()> { 
    let mut found = false;
    for tag in tags.iter() {
        for filter_tag in t.tags.iter() {
            if tag.contains(filter_tag) {
                found = true;
                break;
            }
        }
    }

    if !found {
        return Ok(());
    }

    let content = copy_attachments(t, content, path)?;
    let content = content.replace("\n", "  \n");
    
    let filename_os_string = entry.file_name(); 
    let mut filename = filename_os_string.to_str().expect(&format!("Cant parse filename: {:?}", path));
    if filename == t.index_file {
        filename = "index.md";
    }

    let new_path = Path::new(&t.destination).join(filename);
    fs::write(new_path, content.to_owned()).expect(&format!("Unable to write file: {:?}", path));

    Ok(())
}


fn copy_attachments(t: &ExportTask, content: String, path: &PathBuf) -> io::Result<String> {     
    
    let mut new_content = content.clone(); 
    for (start, _) in content.match_indices("[[") {
        let start = start + 2;
        let end = content[start..].find("]]");

        if end.is_none() {
            println!("{:?} has unclosed [[ at end", path);
            continue;
        }
        let end = end.unwrap() + start;

        let next_start = content[start..].find("[[");

        if next_start.is_some() && (next_start.unwrap() + start) < end {
            println!("{:?} has unclosed [[ ", path);
            continue;
        }

        let attachment = &content[start..end];
        if !attachment.contains(".") || attachment.contains(".md") {
            continue;
        }

        let attachment = attachment.split('|').next().unwrap();

        let source_path = Path::new(&t.source);
        let attachment_path = Path::new(attachment);

        let res = find_attachment(source_path, attachment_path)?;
        if res.is_none() {
            println!("Attachment {:?} in {:?} not found.", attachment_path, path);
            continue;
        }
        let attachment_path = res.unwrap();
    
        let new_path = Path::new(&t.destination_source).join(attachment_path.file_name().unwrap());
        let extension = new_path.extension().unwrap().to_str().unwrap();
        let stem = new_path.file_stem().unwrap().to_str().unwrap();

        let stem = stem.replace("~", "-");

        let mut i = 0; 
        let mut new_path = format!("{}-{}.{}", stem, i, extension);
        while fs::exists(&new_path)? {
            i += 1;
            new_path = format!("{}-{}.{}", stem, i, extension);
        }
        
        fs::copy(&attachment_path, &new_path)?;

        new_content = new_content.replace(attachment, &new_path);
    }

    Ok(new_content)
}

fn find_attachment(dir: &Path, sub_path: &Path) -> io::Result<Option<PathBuf>> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() { 
                let res = find_attachment(&path, sub_path)?;
                if res.is_some() {
                    return Ok(res);
                }
            } else {
                let path = entry.path();
                if path.ends_with(sub_path) {
                    return Ok(Some(path));
                }
            }
        }
    }
    Ok(None)
}

