fn main() {
    let mut nova = core::new();

    // IO
    nova.add_function("print", native::io::print);
    nova.add_function("println", native::io::println);
    nova.add_function("readln", native::io::readln);

    // random
    nova.add_function("random", native::random::random);

    // list
    nova.add_function("length", native::list::length);
    nova.add_function("push", native::list::push);
    nova.add_function("pop", native::list::pop);
    nova.add_function("last", native::list::last);
    nova.add_function("insert", native::list::insert);
    nova.add_function("remove", native::list::remove);
    
    match std::env::args().nth(1) {
        Some(option) => match option.as_str() {
            "run" => {
                if let Some(filepath) = std::env::args().nth(2) {
                    if let Err(error) = nova.open_file(&filepath) {
                        println!("{:?}", error);
                        return;
                    }
                    nova.run();
                } else {
                    println!("Error: No file path specified");
                }
            }
            "dis" => {
                if let Some(filepath) = std::env::args().nth(2) {
                    if let Err(error) = nova.open_file(&filepath) {
                        println!("{:?}", error);
                        return;
                    }
                    println!("Disassembly:");
                    nova.dis();
                } else {
                    println!("Error: No file path specified");
                }
            }

            "fmt" => {
                if let Some(filepath) = std::env::args().nth(2) {
                    match fmt::format_code(&filepath) {
                        Ok(_) => {
                            println!("Format Complete!")
                        }
                        Err(_) => {
                            println!("Could not format {}", filepath)
                        }
                    }
                } else {
                    println!("Error: No file path specified");
                }
            }
            _ => {
                println!("Error: Unrecognized option {}", option);
            }
        },
        None => {
            let mut input = String::new();
            loop {
                print!("Nova $ ");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                input.clear();
                std::io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();
                match input.to_ascii_lowercase().as_str() {
                    "exit" => std::process::exit(0),
                    _ => {
                        if !input.is_empty() {
                            match nova.eval(input, true) {
                                Ok(_) => {}
                                Err(error) => error.show(),
                            }
                        }
                    }
                }
            }
        }
    }
}
