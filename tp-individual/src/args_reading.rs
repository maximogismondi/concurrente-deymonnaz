const ARGS: usize = 3;

pub fn read_args() -> (String, usize, String) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != ARGS + 1 {
        eprintln!("Usage: {} <input_dir> <threads> <output_file>", args[0]);
        std::process::exit(1);
    }

    let input_dir = args[1].to_string();
    let output_file = args[3].to_string();

    let threads = match args[2].parse() {
        Ok(threads) => threads,
        Err(_) => {
            eprintln!("Invalid number of threads");
            std::process::exit(1);
        }
    };

    (input_dir, threads, output_file)
}
