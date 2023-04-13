fn read_lines<P>(filename: P) -> std::io::Result<std::io::Lines<std::io::BufReader<std::fs::File>>>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(filename)?;
    Ok(std::io::BufRead::lines(std::io::BufReader::new(file)))
}

fn print_line(line: usize, row: Option<usize>, file: &str, msg: &str) {
    if let Ok(lines) = read_lines(file) {
        // Consumes the iterator, returns an (Optional) String
        let mut linenumber = 0;
        for l in lines {
            linenumber += 1;
            if linenumber == line {
                let spaces = line.to_string().chars().count();
                if let Some(mut row) = row {
                    row = (row + 1) - spaces;
                    if let Ok(ip) = l {
                        for _ in 0..spaces {
                            print!(" ")
                        }
                        println!(" |");
                        println!("{} |  {} ", line, ip);

                        for _ in 0..spaces {
                            print!(" ")
                        }
                        print!(" |");

                        for _ in 0..=row {
                            print!(" ")
                        }
                        print!("^ {}", msg);

                        println!();
                    }
                } else if let Ok(ip) = l {
                    for _ in 0..spaces {
                        print!(" ")
                    }
                    println!(" |");
                    println!("{} |  {} ", line, ip);

                    for _ in 0..spaces {
                        print!(" ")
                    }
                    print!(" |");

                    println!();
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ErrorType {
    File,
    Lexing,
    Parsing,
    Compiler,
    Runtime,
}

#[derive(Debug)]
pub struct NovaError {
    error: ErrorType,
    msg: String,
    note: String,
    line: usize,
    row: usize,
    filepath: String,
}

impl NovaError {
    #[inline(always)]
    pub fn show(&self) {
        match self.error {
            ErrorType::File => {
                println!("File Error: {}", self.msg)
            }
            ErrorType::Lexing => {
                println!("Lexing Error in: {}", self.filepath);
                print_line(self.line, Some(self.row), &self.filepath, &self.msg);
                println!("Note: {}", self.note);
            }
            ErrorType::Parsing => todo!(),
            ErrorType::Runtime => {
                println!("Runtime Error: {}", self.msg)
            }
            ErrorType::Compiler => {
                println!("Compiler Error in: {}", self.filepath);
                print_line(self.line, None, &self.filepath, &self.msg);
                println!("Note: {}", self.note);
            }
        }
    }
}

pub fn file_error(msg: String) -> NovaError {
    NovaError {
        error: ErrorType::File,
        msg,
        note: String::new(),
        line: 0,
        filepath: String::new(),
        row: 0,
    }
}

pub fn lexer_error(
    msg: String,
    note: String,
    line: usize,
    row: usize,
    filepath: String,
) -> NovaError {
    NovaError {
        error: ErrorType::Lexing,
        msg,
        note,
        line,
        row,
        filepath,
    }
}

pub fn runetime_error(msg: String) -> NovaError {
    NovaError {
        error: ErrorType::Runtime,
        msg,
        note: String::new(),
        line: 0,
        filepath: String::new(),
        row: 0,
    }
}

pub fn compiler_error(note: String, line: usize, filepath: String) -> NovaError {
    NovaError {
        error: ErrorType::Compiler,
        msg: String::new(),
        note,
        line: line + 1,
        filepath,
        row: 0,
    }
}
