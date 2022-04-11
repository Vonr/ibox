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
    pub center: bool,
    pub position: (u16, u16),
    pub query: Vec<String>,
    pub length: u16,
}

impl Config {
    fn new(args: Args) -> Self {
        let mut border = vec!['┌', '─', '┐', '│', '└', '┘'];
        let mut center = false;
        let mut position = crossterm::cursor::position().expect("Could not get cursor position");
        let mut query: Vec<String> = Vec::new();
        let mut length = 8;
        let mut finished = false;

        for arg in args.skip(1) {
            if !finished {
                if arg == "--" {
                    finished = true;
                } else if arg.starts_with('-') {
                    let trimmed = arg.trim_start_matches('-');
                    if trimmed.contains('=') {
                        if let Some(stripped) = trimmed.strip_prefix("b=") {
                            match stripped {
                                "single" => (),
                                "double" => border = vec!['╔', '═', '╗', '║', '╚', '╝'],
                                "thick" => border = vec!['┏', '━', '┓', '┃', '┗', '┛'],
                                "curved" => border = vec!['╭', '─', '╮', '│', '╰', '╯'],
                                _ => {
                                    border = stripped.chars().collect::<Vec<char>>();
                                    if border.len() != 6 {
                                        error(&format!("Invalid border length: {}", border.len()));
                                    }
                                }
                            }
                            continue;
                        } else if let Some(stripped) = trimmed.strip_prefix("l=") {
                            length = stripped.parse::<u16>().unwrap_or(8);
                            continue;
                        } else if let Some(stripped) = trimmed.strip_prefix("p=") {
                            if let Some((x, y)) = stripped.split_once(',') {
                                if let Ok(x) = x.parse::<u16>() {
                                    if let Ok(y) = y.parse::<u16>() {
                                        position = (x, y);
                                        continue;
                                    }
                                }
                            }
                            error("Invalid position");
                        }
                    } else {
                        match trimmed {
                            "c" => {
                                center = true;
                                continue;
                            }
                            "h" => {
                                print_help();
                                continue;
                            }
                            _ => {
                                eprintln!("Invalid argument: {}", arg);
                                exit(1);
                            }
                        }
                    }
                } else {
                    finished = true;
                }
            }

            query.push(arg);
        }

        if query.is_empty() {
            error("No query specified");
        }

        Self {
            border,
            center,
            position,
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
        "Usage: ibox [OPTION]... [QUERY]...\n",
        "Search for QUERY in FILES.\n",
        "Example:\n",
        "    ibox -l=24 'Title' 'Context' 'Question: ?>'\n",
        "Options:\n",
        "    -b=BORDER\n",
        "        Specify the border characters or presets.\n",
        "        Presets: single (default), double, thick, curved\n",
        "        Default: ┌─┐│└┘\n",
        "    -l=LENGTH\n",
        "        Specify the added length of the input space after the longest line.\n",
        "        Default: 8\n",
        "    -p=X,Y\n",
        "        Specify the position of the top left corner of the box.\n",
        "        Default: current cursor position\n",
        "    -c\n",
        "        Center the box on the screen.\n",
        "    -h\n",
        "        Print this help message and exit.\n",
    ));
}

fn main() {
    let config = Config::new(env::args());
    let border = config.border;
    let query = config.query;
    let stderr = &mut stderr();
    let mut positions: Vec<(u16, u16)> = Vec::new();

    let length = query
        .iter()
        .map(|q| q.len() as u16 - q.ends_with("?>") as u16 * 2)
        .max()
        .unwrap()
        + config.length;

    let center = config.center;
    let (sx, sy) = if center {
        let (sx, sy) = crossterm::terminal::size().expect("Failed to get terminal size");
        (sx / 2 - length / 2 - 2, sy / 2 - query.len() as u16 / 2 - 2)
    } else {
        config.position
    };
    cursor(stderr, sx, sy);

    let mut sy = sy + 1;

    eprintln!("{}", top(query.get(0), &border, length));

    if query.len() > 1 {
        for q in query.iter().skip(1) {
            cursor(stderr, sx, sy);
            let (text, new_pos) = mid(q, &border, length);
            if let Some(new_pos) = new_pos {
                positions.push(new_pos);
            }
            eprintln!("{}", text);
            sy += 1;
        }
    }
    cursor(stderr, sx, sy);

    eprintln!("{}", bot(&border, length));

    let mut input = String::new();
    for (x, y) in positions {
        let (ex, ey) = crossterm::cursor::position().expect("Failed to get cursor position");
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

        cursor(stderr, ex, ey);
        input.push('\n');
    }

    print!("{}", input);
}

fn top(title: Option<&String>, border: &[char], length: u16) -> String {
    let mut top = String::new();
    top.push(border[0]);
    top.push(border[1]);
    if let Some(title) = title {
        top.push_str(title);
        for _ in 0..(length - title.len() as u16) {
            top.push(border[1]);
        }
    }

    top.push(border[2]);
    top
}

fn mid(text: &String, border: &[char], length: u16) -> (String, Option<(u16, u16)>) {
    let mut mid = String::new();
    mid.push(border[3]);
    if let Some(question) = text.strip_suffix("?>") {
        if let Ok((x, y)) = crossterm::cursor::position() {
            let question_len = question.len() as u16;
            mid.push_str(question);
            for _ in 0..(length - question_len + 1) {
                mid.push(' ');
            }
            mid.push(border[3]);
            return (mid, Some((x + 1 + question_len as u16, y)));
        }

        error("Cannot get cursor position");
        exit(1);
    } else {
        mid.push_str(text);
        for _ in 0..(length - text.len() as u16 + 1) {
            mid.push(' ');
        }
        mid.push(border[3]);
        (mid, None)
    }
}

fn bot(border: &[char], length: u16) -> String {
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
