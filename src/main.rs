mod spec;
mod tree;

use base64::prelude::*;
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use spec::Header;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::{env, fs};
use tree::{build_huff_tree, get_huffe};

const FILE_EXTENSION: &str = ".whar";
const SPEC_VERSION: &str = "0.0.1";

const BASE64_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::BIN_HEX, general_purpose::NO_PAD);

fn huffe(input: &str) -> (Vec<u8>, HashMap<char, String>) {
    // Get count of all chars
    let mut frequencies: HashMap<char, usize> = HashMap::new();

    for c in input.chars() {
        let count = frequencies.entry(c).or_insert(0);
        *count += 1;
    }

    // Sort counts ascending
    let mut frequencies_sorted: Vec<(&char, &usize)> = frequencies.iter().collect();
    frequencies_sorted.sort_by(|a, b| a.1.cmp(b.1));

    // Insert into tree
    let tree = build_huff_tree(frequencies_sorted);

    // Get encodings
    let huff_map: HashMap<char, String> = match tree {
        Some(t) => get_huffe(&t),
        None => {
            println!("ERROR: Failed to encode files");
            panic!("FATAL ERROR");
        }
    };

    // Map input to encodings
    let mut encoded: Vec<u8> = Vec::new();
    let mut byte = 0;
    let mut num_bits = 0;

    for c in input.chars() {
        if let Some(mapping) = huff_map.get(&c) {
            for bit in mapping.chars() {
                byte <<= 1;
                if bit == '1' {
                    byte |= 1;
                }

                num_bits += 1;

                if num_bits == 8 {
                    encoded.push(byte);
                    byte = 0;
                    num_bits = 0;
                }
            }
        }
    }

    if num_bits > 0 {
        byte <<= 8 - num_bits;
        encoded.push(byte);
    }

    (encoded, huff_map)
}

fn huffd(input: Vec<u8>, map: HashMap<char, String>) -> String {
    let mut decoded = String::new();
    let mut current_code = String::new();

    for byte in input {
        current_code.push_str(&format!("{:08b}", byte));
        for (key, val) in &map {
            if current_code.starts_with(val) {
                decoded.push(*key);
                current_code = current_code[val.len()..].to_string();
            }
        }
    }

    decoded
}

fn main() {
    let mut args = env::args().skip(1);

    'out: while let Some(arg) = args.next() {
        match &arg[..] {
            "-h" | "--help" => {
                // TODO: Help
                println!("NOTE: help is unimplemented");
            }
            "a" | "archive" => {
                // Get archive or output path
                let archive_path: String = match args.next() {
                    Some(a) => {
                        if a.ends_with(FILE_EXTENSION) {
                            a
                        } else {
                            a + FILE_EXTENSION
                        }
                    }
                    None => {
                        println!("ERROR: No path to archive providied, aborting");
                        break 'out;
                    }
                };

                // Get input file paths
                let input_files: Vec<String> = args.collect();

                if input_files.len() == 0 {
                    println!("ERROR: No file provided, aborting");
                    break;
                }

                // Store file contents of file paths in Strings
                let mut file_contents: Vec<String> = Vec::new();

                for path in &input_files {
                    let mut file = match File::open(path) {
                        Ok(f) => f,
                        Err(_) => {
                            println!("ERROR: Could not open file <{}>, aborting", path);
                            break 'out;
                        }
                    };

                    // Read base64 encoded strings into file_contents
                    let mut contents = Vec::new();

                    match file.read_to_end(&mut contents) {
                        Ok(_) => {
                            let mut output = String::new();
                            BASE64_ENGINE.encode_string(contents, &mut output);
                            file_contents.push(output);
                        }
                        Err(_) => {
                            println!("ERROR: Could not read file <{}>, aborting", path);
                            break 'out;
                        }
                    }
                }

                // Store file sizes for later
                let mut file_sizes: Vec<usize> = Vec::new();

                for file in &file_contents {
                    file_sizes.push(file.len());
                }

                // Concatenate files into a blob
                let all_file_contents: String = file_contents.join("");

                // Huffman encode files
                let (encoded, map) = huffe(&all_file_contents);

                let header = Header::new(SPEC_VERSION.to_string(), input_files, file_sizes, map);

                // Get serialized data
                let mut result_archive = header.get_serialized();
                result_archive.extend_from_slice(b"$$"); // END OF HEADER
                result_archive.extend(encoded);

                // Write to file
                let mut file = match File::create(&archive_path) {
                    Ok(f) => f,
                    Err(e) => {
                        println!(
                            "ERROR: Could not open file <{}> for writing, {}",
                            archive_path, e
                        );
                        break 'out;
                    }
                };

                if let Err(e) = file.write_all(result_archive.as_slice()) {
                    println!("ERROR: Could not write to file <{}>, {}", archive_path, e);
                };

                break;
            }
            "x" | "extract" => {
                // Get archive path or input path
                let archive_path: String = match args.next() {
                    Some(a) => {
                        if a.ends_with(FILE_EXTENSION) {
                            a
                        } else {
                            a + FILE_EXTENSION
                        }
                    }
                    None => {
                        println!("ERROR: No path to archive providied, aborting");
                        break 'out;
                    }
                };

                // Read archive
                let mut archive_file = match File::open(&archive_path) {
                    Ok(f) => f,
                    Err(e) => {
                        println!("ERROR: Could not open archive <{}̱>, {}", archive_path, e);
                        break 'out;
                    }
                };

                let mut archive_contents: Vec<u8> = Vec::new();

                match archive_file.read_to_end(&mut archive_contents) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("ERROR: Could not read file <{}>, {}", archive_path, e);
                        break 'out;
                    }
                }

                // Split header and encoded files
                let mut header_serialized: Vec<u8> = Vec::new();
                let mut files_encoded: Vec<u8> = Vec::new();

                for i in 0..archive_contents.len() {
                    if i < archive_contents.len() - 2
                        && (archive_contents[i] == b'0' || archive_contents[i] == b'1')
                        && archive_contents[i + 1] == b'$'
                        && archive_contents[i + 2] == b'$'
                    {
                        header_serialized.extend(&archive_contents[..=i]);
                        files_encoded.extend(&archive_contents[i + 3..]);
                    }
                }

                // Deserialize header
                let header = Header::get_deserialized(header_serialized);

                // Decode files
                let all_file_contents = huffd(files_encoded, header.hash_map);

                // Split files by their sizes
                let mut file_contents_b64: Vec<String> = Vec::new();

                let mut start = 0;

                for size in header.file_sizes {
                    let end = start + size;
                    file_contents_b64.push(all_file_contents[start..end].to_string());
                    start = end;
                }

                // Base 64 decode files
                let mut file_contents: Vec<Vec<u8>> = Vec::new();

                for file in file_contents_b64 {
                    let output = match BASE64_ENGINE.decode(file) {
                        Ok(f) => f,
                        Err(e) => {
                            println!("ERROR: Failed to decode base64, {}", e);
                            break 'out;
                        }
                    };
                    file_contents.push(output);
                }

                // Create base directory
                let base_dir = archive_path.replace(FILE_EXTENSION, "");
                if let Err(e) = fs::create_dir(&base_dir) {
                    println!("ERROR: Could not make directory <{}>, {}", base_dir, e);
                    break 'out;
                }

                // Create subdirectories
                for file_name in &header.file_names {
                    let path = Path::new(&base_dir).join(file_name);
                    if let Some(parent) = path.parent() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            println!("ERROR: Failed to create dirs, {}", e);
                            break 'out;
                        }
                    }
                }

                // Write files based off file name
                for (file_name, file_content) in header.file_names.iter().zip(file_contents.iter())
                {
                    let file_path = Path::new(&base_dir).join(&file_name);

                    if let Err(e) = fs::write(&file_path, &file_content) {
                        println!(
                            "ERROR: Failed to write to file <{}>, {}",
                            file_path.display(),
                            e
                        );
                        break 'out;
                    }
                }
            }
            _ => {
                println!("ERROR: Unrecognized command: <{}>", arg);
                println!("Available commands:\narchive, extract");
            }
        }
    }
}
