use std::{
    env::{self, Args},
    io::{stderr, Stderr},
    process::exit,
};

use crossterm::{
    event::{read, Event, KeyCode},
    ExecutableCommand, QueueableCommand,
};

struct Config {
    pub border: Vec<char>,
    pub query: Vec<String>,
    pub length: usize,
}

impl Config {
    fn new(args: Args) -> Self {
        let mut border = vec!['┌', '─', '┐', '│', '└', '┘'];
        let mut query: Vec<String> = Vec::new();
        let mut length = 8;
        let mut finished = false;

        for arg in args.skip(1) {
            if !finished {
                if arg == "--" {
                    finished = true;
                } else if arg.starts_with('-') {
                    let trimmed = arg.trim_start_matches('-');
                    if trimmed.contains("=") {
                        if let Some(stripped) = trimmed.strip_prefix("b=") {
                            border = stripped.chars().collect::<Vec<char>>();
                            if border.len() != 6 {
                                eprintln!("Invalid border length: {}", border.len());
                                exit(1);
                            }
                            continue;
                        } else if let Some(stripped) = trimmed.strip_prefix("l=") {
                            length = stripped.parse::<usize>().unwrap_or(8);
                            continue;
                        }
                    } else if trimmed == "h" {
                        print_help();
                        continue;
                    } else {
                        finished = true;
                    }
                }
            }

            query.push(arg);
        }

        if query.is_empty() {
            error("No query specified");
        }

        Self {
            border,
            query,
            length,
        }
    }
}

fn error(msg: &str) {
    eprintln!("error: {}", msg);
    print_help();
    exit(1);
}

fn print_help() {
    eprintln!(concat!(
        "Usage: ibox [OPTION]... QUERY\n",
        "Search for QUERY in FILES.\n",
        "Example:\n",
        "    ibox -l=24 'Title' 'Context' 'Question?>'\n",
        "Options:\n",
        "    -b=BORDER, --border=BORDER\n",
        "        Specify the border characters.\n",
        "        Default: ┌─┐│└┘\n",
        "    -l=LENGTH, --length=LENGTH\n",
        "        Specify the max length of the input.\n",
        "        Default: 8\n",
        "    -h, --help\n",
        "        Print this help message and exit.\n",
    ));
}

fn main() {
    let config = Config::new(env::args());
    let border = config.border;
    let query = config.query;
    let mut positions: Vec<(u16, u16)> = Vec::new();

    let length = query.iter().map(|q| q.len()).max().unwrap() + config.length;

    eprintln!("{}", top(query.get(0), &border, length));

    if query.len() > 1 {
        for q in query.iter().skip(1) {
            let (text, new_pos) = mid(q, &border, length);
            if let Some(new_pos) = new_pos {
                positions.push(new_pos);
            }
            eprintln!("{}", text);
        }
    }

    eprintln!("{}", bot(&border, length));

    let mut input = String::new();
    let stderr = &mut stderr();
    for (x, y) in positions {
        let end_pos = crossterm::cursor::position().expect("Failed to get cursor position");
        loop {
            force_cursor(stderr, x, y);
            match read().expect("Failed to read input") {
                Event::Key(event) => {
                    if event.code == crossterm::event::KeyCode::Enter {
                        break;
                    }
                    if let KeyCode::Char(c) = event.code {
                        input.push(c);
                    }
                }
                _ => break,
            }
        }

        cursor(stderr, end_pos.0, end_pos.1);
        input.push('\n');
    }

    println!("{}", input);
}

fn top(title: Option<&String>, border: &Vec<char>, length: usize) -> String {
    let mut top = String::new();
    top.push(border[0]);
    top.push(border[1]);
    if let Some(title) = title {
        top.push_str(title);
        for _ in 0..(length - title.len()) {
            top.push(border[1]);
        }
    }

    top.push(border[2]);
    top
}

fn mid(text: &String, border: &Vec<char>, length: usize) -> (String, Option<(u16, u16)>) {
    let mut mid = String::new();
    mid.push(border[3]);
    if let Some(question) = text.strip_suffix("?>") {
        if let Ok((x, y)) = crossterm::cursor::position() {
            mid.push_str(question);
            for _ in 0..(length - question.len() + 1) {
                mid.push(' ');
            }
            mid.push(border[3]);
            return (mid, Some((x + 1 + question.len() as u16, y)));
        }

        error("Cannot get cursor position");
        exit(1);
    } else {
        mid.push_str(text);
        for _ in 0..(length - text.len() + 1) {
            mid.push(' ');
        }
        mid.push(border[3]);
        (mid, None)
    }
}

fn bot(border: &Vec<char>, length: usize) -> String {
    let mut bot = String::new();
    bot.push(border[4]);
    bot.push(border[1]);
    for _ in 0..length {
        bot.push(border[1]);
    }
    bot.push(border[5]);
    bot
}

fn cursor(stderr: &mut Stderr, x: u16, y: u16) {
    stderr
        .queue(crossterm::cursor::MoveTo(x, y))
        .expect("Failed to move cursor");
}

fn force_cursor(stderr: &mut Stderr, x: u16, y: u16) {
    stderr
        .execute(crossterm::cursor::MoveTo(x, y))
        .expect("Failed to move cursor");
}
