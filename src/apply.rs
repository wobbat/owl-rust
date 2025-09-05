pub fn run() {
    println!("[{}]", crate::colo::red("info"));
    println!("  host: {}", crate::colo::bold("gengar"));
    println!("  packages: {} ({}, {}, {})", crate::colo::bold("68"), crate::colo::green("install +1"), crate::colo::yellow("upgrade 7"), crate::colo::red("remove 0"));
    println!("  dotfiles: {}", crate::colo::bold("7"));
    println!();
    println!("[{}]", crate::colo::yellow("packages"));
    println!("  checking for package upgrades... {}", crate::colo::dim("(602ms)"));
    println!("  {}", crate::colo::blue("nothing to do"));
    println!();
    println!("[{}]", crate::colo::green("config"));
    println!("  {} packages -> {}", crate::colo::bold("7"), crate::colo::blue("up to date"));
    println!("  dotfiles {}", crate::colo::dim("(0ms)"));
    println!();
    println!("[{}]", crate::colo::magenta("system"));
    println!("  services {} {}", crate::colo::blue("verified"), crate::colo::dim("(121ms)"));
    println!("  environment {} ({} unchanged)", crate::colo::blue("maintained"), crate::colo::bold("5"));
}