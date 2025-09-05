use crate::colo;

pub fn print_usage() {
    eprintln!("{}", colo::bold("Usage: owl [OPTIONS] <COMMAND>"));
    eprintln!("{}", colo::green("Commands:"));
    eprintln!("  apply");
    eprintln!("  edit {} {}", colo::dim("<type>"), colo::dim("<argument>"));
    eprintln!("    de {}    {}", colo::dim("<argument>"), colo::dim("(alias for edit dots)"));
    eprintln!("    ce {}    {}", colo::dim("<argument>"), colo::dim("(alias for edit config)"));
    eprintln!("  add {}", colo::dim("<items...>"));
    eprintln!("{}", colo::blue("Options:"));
    eprintln!(
        "  {}   {}",
        colo::bold("-v, --verbose"),
        colo::dim(":Enable verbose logging")
    );
}