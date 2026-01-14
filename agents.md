# Distorted (ted for short)

<high level of what we are building>

## Project Structure

- `src/cli/` - CLI parsing and command handling
- `src/commands/` - Command implementations (add, adopt, apply, dots, edit, find, clean)
- `src/core/` - Core functionality (config management)
- `src/domain/` - Domain models and types
- `src/infrastructure/` - Infrastructure code
- `src/internal/` - Internal utilities (color, constants)
- `src/error.rs` - Error types

## Key Commands

The CLI uses clap for argument parsing with these main subcommands:
- `apply` - Apply configuration (default)
- `dots` - List dotfiles
- `add` - Add packages
- `adopt` - Adopt existing packages
- `find` - Find packages or files
- `edit` - Edit dotfiles or config
- `config-check` - Check configuration
- `config-host` - Show host configuration
- `clean` - Clean up files

## Global Flags

- `-v, --verbose` - Enable verbose output
- `--dry-run` - Perform a dry run without making changes
- `-y, --non-interactive` - Run in non-interactive mode
