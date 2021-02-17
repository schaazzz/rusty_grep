extern crate getopts;
extern crate wild;

use std::env;
use getopts::{ Options };
use std::io::{ self, Write };
use std::thread::{ self, JoinHandle };
use termcolor::{ Color, ColorChoice, ColorSpec, StandardStream, WriteColor };
use std::sync::{ atomic::{ Ordering, AtomicBool }, mpsc::{ channel, Sender, Receiver }, Arc };

mod line_grep;
use line_grep::{ LineGrep };
mod line_src;
use line_src::{ LineSource, LinesFromStdin, LinesFromFiles };

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

#[derive(Debug)]
struct Flags {
    ignore_case: bool,
    print_line_nums: bool,
    use_color: bool,
}

#[derive(Debug)]
struct Config {
    flags: Flags,
    pattern: Option<String>,
    files: Vec<String>,
}

#[allow(dead_code)]
fn print_error(error: String, force_exit: bool) {
    eprintln!("---");
    eprintln!("Error: {}", error);
    eprintln!("---");

    if force_exit {
        panic!();
    }
}

fn print_usage(opts: &Options, print_about: bool) {
    println!("---");

    if print_about {
        println!("{}, v{}", DESCRIPTION, VERSION);
        println!("{}", AUTHORS);
        println!("");
    }

    println!("{}", opts.usage("Usage: rusty_grep [OPTION]... PATTERNS [FILE]..."));
    println!("---");
}

fn parse_args(args: &[String]) -> Option<Config> {
    let mut opts = Options::new();
    opts.optflag("i", "", "ignore case distinctions in patterns and data");
    opts.optflag("n", "", "print line number with output lines");
    opts.optflag("", "color", "use markers to highlight the matching strings");
    opts.optflag("", "help", "Print this help menu");

    let mut config = Config {
        flags: Flags {
            ignore_case: false,
            print_line_nums: false,
            use_color: false,
        },
        pattern: None,
        files: Vec::<String>::new(),
    };

    let matches = match opts.parse(args) {
        Ok(o) => o,
        Err(_) => {
            print_usage(&opts, true);
            return None;
        }
    };

    if matches.opt_present("help") {
        print_usage(&opts, true);
        return None;
    }

    if matches.opt_present("i") {
        config.flags.ignore_case = true;
    }

    if matches.opt_present("n") {
        config.flags.print_line_nums = true;
    }

    if matches.opt_present("color") {
        config.flags.use_color = true;
    }

    if matches.free.is_empty() {
        println!("Error: Pattern cannot be empty!");
        print_usage(&opts, false);
        return None;
    }
    else {
        config.pattern = Some(matches.free[0].to_string());

        for f in &matches.free[1..] {
            config.files.push(f.to_string());
        }
    }

    Some(config)
}

fn print_matched_line(flags: &Flags, prefix: String, index: u32, line: String, start: usize, end: usize) -> std::io::Result <()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.reset()?;

    if !prefix.is_empty() {
        if flags.use_color {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))?;
        }
        write!(&mut stdout, "{}", prefix)?;

        if flags.use_color {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
        }
        write!(&mut stdout, ":")?;
    }

    if flags.print_line_nums {
        if flags.use_color {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        }
        write!(&mut stdout, "{}", index)?;

        if flags.use_color {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
        }
        write!(&mut stdout, ":")?;
    }

    if flags.use_color {
        stdout.reset()?;
        write!(&mut stdout, "{}", line[0..start].to_string())?;
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        write!(&mut stdout, "{}", line[start..end].to_string())?;
        stdout.reset()?;
        writeln!(&mut stdout, "{}", line[end..line.len()].to_string())?;
    }
    else {
        writeln!(&mut stdout, "{}", line)?;
    }

    Ok(())
}

fn  process(source: impl LineSource, flags: &Flags, pattern: String) -> Result<(), String> {
    let mut grep = match LineGrep::new(pattern, flags.ignore_case) {
        Ok(o) => o,
        Err(_) => return Err("Inavild regular expression!".to_owned())
    };
    
    let flag_done = Arc::new(AtomicBool::new(false));
    let flag_exit = Arc::clone(&flag_done);

    let (main_tx_channel, main_rx_channel): (Sender<(usize, usize)>, Receiver<(usize, usize)>) = channel();
    let (thread_tx_channel, thread_rx_channel): (Sender<String>, Receiver<String>) = channel();
    let join_handle: JoinHandle<()> = thread::spawn(move || {
        loop {
            if let Ok(line) = thread_rx_channel.try_recv() {
                if let Some((start, end)) = grep.search(&line) {
                    main_tx_channel.send((start, end)).expect("Error: Sending from grep to main thread failed!");
                }
                else{
                    main_tx_channel.send((0, 0)).expect("Error: Sending from grep to main thread failed!");
                }
            }

            if flag_exit.load(Ordering::Acquire) {
                break;
            }
        }
    });

    for (prefix, index, line) in source {
        thread_tx_channel.send(line.clone()).expect("Error: Sending from grep to main thread failed!");
        let (start, end) = main_rx_channel.recv().expect("Error: Receiving in main thread failed!");
        
        if start != end {
            match print_matched_line(flags, prefix, index , line, start as usize, end as usize) {
                Ok(_) => (),
                Err(e) => return Err(format!("Error: {}", e.to_string()))
            }
        }
    }

    flag_done.store(true, Ordering::Release);
    join_handle.join().unwrap();
    
    Ok(())
}

fn main() -> std::io::Result <()>{
    let args: Vec<String> = wild::args().collect();
    
    let mut config = match parse_args(&args[1..]) {
        Some(s) => s,
        None => std::process::exit(0)
    };

    let stdin_handle = io::stdin();
    if config.files.is_empty() {
        let _ = process(
                    LinesFromStdin::new(&stdin_handle),
                    &config.flags,
                    config.pattern.as_ref().unwrap().to_string()).map_err(|e|  return e);
    }
    else {
        let _ = process(
                    LinesFromFiles::new(&mut config.files),
                    &config.flags,
                    config.pattern.as_ref().unwrap().to_string()).map_err(|e|  return e);
    }

    Ok(())
}
