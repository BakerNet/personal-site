#[derive(Debug)]
pub enum CommandRes {
    EmptyErr,
    Err(String),
    Redirect(String),
    Output(String),
    Nothing,
}

impl From<&str> for CommandRes {
    fn from(value: &str) -> Self {
        let mut parts = value.split_whitespace();
        let cmd_text = if let Some(word) = parts.next() {
            word
        } else {
            return Self::EmptyErr;
        };
        let cmd = Command::from(cmd_text);
        match cmd {
            Command::Help => todo!(),
            Command::Pwd => todo!(),
            Command::Ls => todo!(),
            Command::Cd => todo!(),
            Command::Cat => todo!(),
            Command::Clear => CommandRes::Nothing,
            Command::Mines => Self::Redirect("https://mines.hansbaker.com".to_string()),
            Command::MinesScript => todo!(),
            Command::Relative => {
                if cmd_text == "./" {
                    Self::Nothing
                } else if &cmd_text[2..] == "index.html" {
                    Self::Err(format!("permission denied: {}", cmd_text))
                } else {
                    Self::Err(format!("no such file or directory: {}", cmd_text))
                }
            }
            Command::Unknown => Self::Err(format!("command not found: {}", cmd_text)),
        }
    }
}

enum Command {
    Help,
    Pwd,
    Ls,
    Cd,
    Cat,
    Clear,
    Mines,
    MinesScript,
    Relative,
    Unknown,
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value {
            "help" => Self::Help,
            "pwd" => Self::Pwd,
            "ls" => Self::Ls,
            "cd" => Self::Cd,
            "cat" => Self::Cat,
            "clear" => Self::Clear,
            "mines" => Self::Mines,
            "mines.sh" => Self::Mines,
            x if x.starts_with("./") || x.starts_with("../") || x == ".." => Self::Relative,
            _ => Self::Unknown,
        }
    }
}
