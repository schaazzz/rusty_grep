extern crate getopts;
extern crate wild;

use std::io::{ self };
use std::env;
use getopts::{ Options };
use std::thread::{ self, JoinHandle };
use std::sync::{ atomic::{ Ordering, AtomicBool }, mpsc::{ channel, Sender, Receiver }, Arc, Mutex };

mod line_grep;
use line_grep::{ LineGrep };
mod line_src;
use line_src::{ LineSource, LinesFromStdin, LinesFromFiles };

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

#[derive(Debug)]
struct Config {
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

fn parse_args(args: &[String]) -> Option<Config> {
    let mut opts = Options::new();
    opts.optflag("E", "", "PATTERNS are extended regular expressions");
    opts.optflag("i", "", "ignore case distinctions in patterns and data");
    opts.optflag("n", "", "print line number with output lines");
    opts.optflag("", "color", "use markers to highlight the matching strings");
    opts.optflag("", "help", "Print this help menu");

    let mut config = Config {
        extended_regex: false,
        ignore_case: false,
        print_line_nums: false,
        use_color: false,
        pattern: None,
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
        config.extended_regex = true;
    }

    if matches.opt_present("i") {
        config.ignore_case = true;
    }

    if matches.opt_present("n") {
        config.print_line_nums = true;
    }

    if matches.opt_present("color") {
        config.use_color = true;
    }

    if matches.free.is_empty() {
        println!("this is wrong!");
    }
    else {
        config.pattern = Some(matches.free[0].to_string());

        for f in &matches.free[1..] {
            config.files.push(f.to_string());
        }
    }

    Some(config)
}

fn  process(source: impl LineSource, pattern: String) {
    let mut grep = LineGrep::new(pattern);
    let flag_done = Arc::new(AtomicBool::new(false));
    let flag_exit = Arc::clone(&flag_done);

    let (main_tx_channel, main_rx_channel): (Sender<String>, Receiver<String>) = channel();
    let (thread_tx_channel, thread_rx_channel): (Sender<String>, Receiver<String>) = channel();
    let join_handle: JoinHandle<()> = thread::spawn(move || {
        loop {
            if let Ok(line) = thread_rx_channel.try_recv() {
                grep.feed(line);
                main_tx_channel.send("yakitori".to_string()).expect("Error: Sending from grep to main thread failed!");
            }

            if flag_exit.load(Ordering::Acquire) {
                break;
            }

        }
    });

    for line in source {
        thread_tx_channel.send(line).expect("Error: Sending from grep to main thread failed!");
        let result = main_rx_channel.recv().expect("Error: Receiving in main thread failed!");
        println!("result: {}", result);
    }

    flag_done.store(true, Ordering::Release);
    join_handle.join().unwrap();
}

fn main() -> std::io::Result <()>{
    let args: Vec<String> = wild::args().collect();
    let mut config = parse_args(&args[1..]).unwrap();

    let stdin_handle = io::stdin();

    if config.files.is_empty() {
        process(LinesFromStdin::new(&stdin_handle), config.pattern.unwrap());
    }
    else {
        process(LinesFromFiles::new(&mut config.files), config.pattern.unwrap());
    }

    /*
    let mut grep: LineGrep;
    if let Some(pattern) = config.pattern {
        grep = LineGrep::new(pattern);
    }
    else {
        panic!("lakht pakht");
    }

    let flag_done = Arc::new(AtomicBool::new(false));
    let flag_exit = Arc::clone(&flag_done);

    let (main_tx_channel, main_rx_channel): (Sender<String>, Receiver<String>) = channel();
    let (thread_tx_channel, thread_rx_channel): (Sender<String>, Receiver<String>) = channel();
    let join_handle: JoinHandle<()> = thread::spawn(move || {
        loop {
            if let Ok(line) = thread_rx_channel.try_recv() {
                grep.feed(line);
                main_tx_channel.send("yakitori".to_string()).expect("Error: Sending from grep to main thread failed!");
            }

            if flag_exit.load(Ordering::Acquire) {
                break;
            }

        }
    });

    for line in source {
        thread_tx_channel.send(line).expect("Error: Sending from grep to main thread failed!");
        let result = main_rx_channel.recv().expect("Error: Receiving in main thread failed!");
        println!("result: {}", result);
    }

    flag_done.store(true, Ordering::Release);
    join_handle.join().unwrap();
    */
    Ok(())
}
