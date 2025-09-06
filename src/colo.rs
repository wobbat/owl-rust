#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

fn colorize(s: &str, code: &str) -> String {
    format!("\x1b[{}m{}\x1b[0m", code, s)
}

pub fn red(s: &str) -> String {
    colorize(s, "31")
}

pub fn green(s: &str) -> String {
    colorize(s, "32")
}

pub fn yellow(s: &str) -> String {
    colorize(s, "33")
}

pub fn orange(s: &str) -> String {
    colorize(s, "38;5;208")
}

pub fn env_orange(s: &str) -> String {
    colorize(s, "38;5;166")
}

pub fn system_purple(s: &str) -> String {
    colorize(s, "38;5;97")
}

pub fn blue(s: &str) -> String {
    colorize(s, "34")
}

pub fn magenta(s: &str) -> String {
    colorize(s, "35")
}

pub fn cyan(s: &str) -> String {
    colorize(s, "36")
}

pub fn teal(s: &str) -> String {
    colorize(s, "38;5;37")
}

pub fn white(s: &str) -> String {
    colorize(s, "37")
}

pub fn bg_red(s: &str) -> String {
    colorize(s, "41")
}

pub fn bg_green(s: &str) -> String {
    colorize(s, "42")
}

pub fn bg_yellow(s: &str) -> String {
    colorize(s, "43")
}

pub fn bg_blue(s: &str) -> String {
    colorize(s, "44")
}

pub fn bg_magenta(s: &str) -> String {
    colorize(s, "45")
}

pub fn bg_cyan(s: &str) -> String {
    colorize(s, "46")
}

pub fn bg_white(s: &str) -> String {
    colorize(s, "47")
}

pub fn bold(s: &str) -> String {
    colorize(s, "1")
}

pub fn italic(s: &str) -> String {
    colorize(s, "3")
}

pub fn underline(s: &str) -> String {
    colorize(s, "4")
}

pub fn dim(s: &str) -> String {
    colorize(s, "2")
}

pub fn highlight(s: &str) -> String {
    colorize(s, "1;36") // Bold cyan
}

pub fn success(s: &str) -> String {
    colorize(s, "1;32") // Bold green
}

pub fn warning(s: &str) -> String {
    colorize(s, "1;33") // Bold yellow
}

pub fn repository(s: &str) -> String {
    colorize(s, "1;35") // Bold magenta
}

pub fn description(s: &str) -> String {
    colorize(s, "2;37") // Dim white
}
