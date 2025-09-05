#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub fn red(s: &str) -> String {
    format!("\x1b[31m{}\x1b[0m", s)
}

pub fn green(s: &str) -> String {
    format!("\x1b[32m{}\x1b[0m", s)
}

pub fn yellow(s: &str) -> String {
    format!("\x1b[33m{}\x1b[0m", s)
}

pub fn blue(s: &str) -> String {
    format!("\x1b[34m{}\x1b[0m", s)
}

pub fn magenta(s: &str) -> String {
    format!("\x1b[35m{}\x1b[0m", s)
}

pub fn cyan(s: &str) -> String {
    format!("\x1b[36m{}\x1b[0m", s)
}

pub fn white(s: &str) -> String {
    format!("\x1b[37m{}\x1b[0m", s)
}

pub fn bg_red(s: &str) -> String {
    format!("\x1b[41m{}\x1b[0m", s)
}

pub fn bg_green(s: &str) -> String {
    format!("\x1b[42m{}\x1b[0m", s)
}

pub fn bg_yellow(s: &str) -> String {
    format!("\x1b[43m{}\x1b[0m", s)
}

pub fn bg_blue(s: &str) -> String {
    format!("\x1b[44m{}\x1b[0m", s)
}

pub fn bg_magenta(s: &str) -> String {
    format!("\x1b[45m{}\x1b[0m", s)
}

pub fn bg_cyan(s: &str) -> String {
    format!("\x1b[46m{}\x1b[0m", s)
}

pub fn bg_white(s: &str) -> String {
    format!("\x1b[47m{}\x1b[0m", s)
}

pub fn bold(s: &str) -> String {
    format!("\x1b[1m{}\x1b[0m", s)
}

pub fn italic(s: &str) -> String {
    format!("\x1b[3m{}\x1b[0m", s)
}

pub fn underline(s: &str) -> String {
    format!("\x1b[4m{}\x1b[0m", s)
}

pub fn dim(s: &str) -> String {
    format!("\x1b[2m{}\x1b[0m", s)
}

