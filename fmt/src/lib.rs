use std::fs::{OpenOptions};
use std::io::{BufRead, BufReader, Result, Seek, SeekFrom, Write};

pub fn format_code(filepath: &str) -> Result<()> {
    let mut file = OpenOptions::new().read(true).write(true).open(filepath)?;
    let reader = BufReader::new(&file);
    let mut contents = String::new();
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        let mut prev_char = ' ';
        let mut formatted_line = String::new();
        for c in trimmed.chars() {
            if c.is_whitespace() && prev_char.is_whitespace() {
                continue;
            }
            formatted_line.push(c);
            prev_char = c;
        }
        contents.push_str(&formatted_line);
        contents.push('\n');
    }

    let mut indentation = 0;
    let mut formatted = String::new();
    let mut empty_line = false;

    for line in contents.lines() {
        let trimmed = line.trim();
    
        if trimmed.is_empty() {
            if !empty_line {
                formatted.push('\n');
                empty_line = true;
            }
            continue;
        }
    
        empty_line = false;
    
        if trimmed.ends_with('{') {
            formatted.push_str(&"    ".repeat(indentation));
            formatted.push_str(trimmed);
            formatted.push('\n');
            indentation += 1;
        } else if trimmed.starts_with('}') {
            indentation -= 1;
            formatted.push_str(&"    ".repeat(indentation));
            formatted.push_str(trimmed);
            formatted.push('\n');
        } else {
            formatted.push_str(&"    ".repeat(indentation));
            formatted.push_str(trimmed);
            formatted.push('\n');
        }
    }

    file.seek(SeekFrom::Start(0))?;
    file.write_all(formatted.as_bytes())?;
    file.set_len(formatted.len() as u64)?;

    Ok(())
}
