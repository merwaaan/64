use std::io::{self, BufRead, Write};
use std::path::Path;

use n64::{cart::Cart, cpu::CPU};

fn main() {
    let cart = Cart::load(Path::new("sm.n64")).expect("load ROM");

    println!("START:{:02x?}", cart.pc());

    let mut cpu = CPU::new();
    cpu.skip_ipl(&cart);

    println!("Step: [Enter]=1, N=step N, q=quit");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        let _ = stdout.flush();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            break;
        }
        let line = line.trim();

        if line.is_empty() || line == "1" {
            cpu.step();
            continue;
        }
        if line.eq_ignore_ascii_case("q") || line.eq_ignore_ascii_case("quit") {
            break;
        }
        match line.parse::<u32>() {
            Ok(n) if n > 0 => {
                for _ in 0..n {
                    cpu.step();
                }
            }
            _ => println!("? [Enter]=1, N=step N, q=quit"),
        }
    }
}
