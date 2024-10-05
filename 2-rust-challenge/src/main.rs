use std::io;

use eval::eval;

fn main() {
    loop {
        println!("Please, enter the expression below (enter empty expression to exit):");
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();

                if input.len() == 0 {
                    break;
                }

                let result = eval(input.trim());

                match result {
                    Ok(result) => println!("Result: {result}"),
                    Err(err) => println!("Error: {err:?}"),
                }
            }
            Err(error) => panic!("Error: {error}"),
        }

        println!();
    }

    println!("Have a good day!");
}
