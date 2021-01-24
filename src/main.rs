extern crate getopts;
extern crate wild;

use std::io::{ self };
use std::env;
use std::thread;
use std::sync::mpsc;
use getopts::Options;
use std::time::Duration;

mod line_src;
use line_src::{ LineSource, LinesFromStdin, LinesFromFiles };

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

#[derive(Debug)]
struct GrepArgs {
    extended_regex: bool,
    ignore_case: bool,
    print_line_nums: bool,
    use_color: bool,
    pattern: Option<String>,
    files: Vec<String>,
}

fn print_error(error: String, force_exit: bool) {
    eprintln!("---");
    eprintln!("Error: {}", error);
    eprintln!("---");

    if force_exit {
        panic!();
    }
}

fn print_usage(opts: &Options) {
    println!("---");
    println!("{}, v{}", DESCRIPTION, VERSION);
    println!("{}", AUTHORS);
    println!("");
    println!("{}", opts.usage("Usage:"));
    println!("---");
}

fn parse_args(args: &[String]) -> Option<GrepArgs> {
    let mut opts = Options::new();
    opts.optflag("E", "", "PATTERNS are extended regular expressions");
    opts.optflag("i", "", "ignore case distinctions in patterns and data");
    opts.optflag("n", "", "print line number with output lines");
    opts.optflag("", "color", "use markers to highlight the matching strings");
    opts.optflag("", "help", "Print this help menu");

    let mut grep_args = GrepArgs {
        extended_regex: false,
        ignore_case: false,
        print_line_nums: false,
        use_color: false,
        pattern: Some("".to_string()),
        files: Vec::<String>::new(),
    };

    let matches = match opts.parse(args) {
        Ok(o) => o,
        Err(_) => {
            print_usage(&opts);
            return None;
        }
    };

    if matches.opt_present("help") {
        print_usage(&opts);
        return None;
    }

    if matches.opt_present("E") {
        grep_args.extended_regex = true;
    }

    if matches.opt_present("i") {
        grep_args.ignore_case = true;
    }

    if matches.opt_present("n") {
        grep_args.print_line_nums = true;
    }

    if matches.opt_present("color") {
        grep_args.use_color = true;
    }

    if matches.free.is_empty() {
        println!("this is wrong!");
    }
    else {
        grep_args.pattern = Some(matches.free[0].to_string());

        for f in &matches.free[1..] {
            grep_args.files.push(f.to_string());
        }
    }

    Some(grep_args)
}

fn main() -> std::io::Result <()> {
    //loop {
    //    let mut input = String::new();
    //    io::stdin()
    //        .read_line(&mut input)
    //        .expect("failed to read from pipe");
    //    input = input.trim().to_string();
    //    if input == "" {
    //        break;
    //    }
    //    println!("Pipe output: {}", input);
    //}
    let args: Vec<String> = wild::args().collect();
    let mut grep_args = parse_args(&args[1..]).unwrap();

    let handle = io::stdin();
    let mut source: Box<dyn LineSource>;
    
    if grep_args.files.is_empty() {
        source = Box::new(LinesFromStdin::new(&handle));
    }
    else {
        source = Box::new(LinesFromFiles::new(&mut grep_args.files));
    }

    for line in source {
        println!("line: {:?}", line);
        thread::sleep(Duration::from_millis(25));
    }

    Ok(())
}
