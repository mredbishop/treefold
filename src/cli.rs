use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Gui,
    Tui,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArgs {
    pub mode: Mode,
    pub path: Option<PathBuf>,
    pub help: bool,
}

pub fn parse_args<I, S>(args: I) -> Result<ParsedArgs, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut mode = Mode::Gui;
    let mut path: Option<PathBuf> = None;
    let mut help = false;

    for raw in args {
        let arg = raw.as_ref();
        match arg {
            "-h" | "--help" => help = true,
            "-t" | "--tui" => mode = Mode::Tui,
            _ if arg.starts_with('-') => {
                return Err(format!("unknown option: {arg}"));
            }
            _ => {
                if path.is_some() {
                    return Err("multiple paths provided; expected at most one".to_string());
                }
                path = Some(PathBuf::from(arg));
            }
        }
    }

    Ok(ParsedArgs { mode, path, help })
}

pub fn help_text() -> &'static str {
    "treefold [OPTIONS] [PATH]

Default mode: GUI

Options:
  -t, --tui    Run in terminal UI mode
  -h, --help   Show this help

Args:
  PATH         Optional starting path (GUI default: home, TUI default: current working directory)"
}
