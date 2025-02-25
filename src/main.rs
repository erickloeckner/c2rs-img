use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process;

#[derive(Eq, PartialEq)]
enum Parse {
    Octal,
    Ascii,
    EscapedAscii,
    Empty,
}

fn main() {
    let image_path_string = env::args().nth(1).unwrap_or_else(|| {
        println!("Error: provide a path to c image file");
        process::exit(1);
    });
    
    let mut image_path = PathBuf::new();
    image_path.push(&image_path_string);
    
    let image = fs::read_to_string(&image_path).unwrap_or_else(|err| {
        println!("Error opening file: {}", err);
        process::exit(1);
    });

    image_path.set_extension("rs");
    match fs::File::open(&image_path) {
        Ok(_) => { 
            println!("Error: output file {} exists", image_path.display());
            process::exit(1);
        },
        Err(_) => {},
    }

    let var_name = match env::args().nth(2) {
        Some(v) => String::from(&v.to_lowercase()),
        None    => String::from("image"),
    };
    
    let var_name_all_cap = var_name.to_uppercase();
    
    let mut var_name_cap = String::new();
    for (index, char) in var_name.chars().enumerate() {
        if index == 0 {
            var_name_cap.push(char.to_ascii_uppercase());
        } else {
            var_name_cap.push(char);
        }
    }

    let mut gimp_match = None;

    for (index, line) in image.lines().enumerate() {
        if line.contains("gimp_image") {
            gimp_match = Some(index);
        }
    }

    if let Some(i) = gimp_match {
        let dimensions: Vec<&str> = image
            .lines()
            .nth(i + 1)
            .unwrap()
            .split_terminator(",")
            .collect();
        let w = usize::from_str_radix(dimensions.get(0).unwrap().trim(), 10).unwrap();
        let h = usize::from_str_radix(dimensions.get(1).unwrap().trim(), 10).unwrap();
        let bpp = usize::from_str_radix(dimensions.get(2).unwrap().trim(), 10).unwrap();
        let mut output_vec = Vec::new();

        for line in image.lines().skip(i + 2) {
			let mut next_value = 0;
            let mut current_parsed = Parse::Empty;
            let quotes: Vec<_> = line.match_indices("\"").collect();
            if quotes.len() < 2 {
                break
            }
            let start = quotes[0].0 + 1;
            let end = quotes[quotes.len() - 1].0;
            let line_sub = &line[start..end];
            
            for (index, c) in line_sub.chars().enumerate() {
                match c {
                    '\\' => { 
                        if line_sub[index + 1..index + 2].chars().nth(0).unwrap().is_digit(8) {
                            next_value = index + 4; 
                            current_parsed = Parse::Octal;
                        } else {
                            next_value = index + 2;
                            current_parsed = Parse::EscapedAscii;
                        }
                    }
                    _ => {
                        if index == next_value {
                            next_value = index + 1;
                            current_parsed = Parse::Ascii;
                        }
                    }
                }
                
                match current_parsed {
                    Parse::Empty => {}
                    Parse::Octal => {
                        let octal = &line_sub[index + 1..index + 4];
                        current_parsed = Parse::Empty;
                        output_vec.push(u8::from_str_radix(&octal, 8).unwrap());
                    }
                    Parse::Ascii => {
                        let value = line_sub.chars().nth(index).unwrap();
                        output_vec.push(u32::from(value) as u8);
                    }
                    Parse::EscapedAscii => {
                        let value = line_sub.chars().nth(index + 1).unwrap();
                        current_parsed = Parse::Empty;
                        output_vec.push(u32::from(value) as u8);
                    }
                }
            }
        }

        assert_eq!((w * h * bpp), output_vec.len());

        let output_struct = format!(
            "pub struct {} {{\n    pub width: usize,\n    pub height: usize,\n    pub bytes_per_pixel: u8,\n    pub pixel_data: [u8; {}]\n}}\n\n", 
            var_name_cap, output_vec.len()
        );

        let output_data = format!(
            "pub static {}: {} = {} {{\n    width: {},\n    height: {},\n    bytes_per_pixel: {},\n    pixel_data: {:?},\n}};\n", 
            var_name_all_cap, var_name_cap, var_name_cap, w, h, bpp, output_vec
        );

        if let Ok(mut file) = fs::File::create(&image_path) {
            file.write_all(&output_struct.as_bytes()).unwrap();
            file.write_all(&output_data.as_bytes()).unwrap();

            println!("Success: file written to {}", image_path.display());
        }

    } else {
        println!("Error: invalid file")
    }
}
