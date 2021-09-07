use connection::parse::{frame::Frame, Parse};

/// Represents an "unknown" command. This is not a real `Redis` command.
#[derive(Debug)]
pub struct Unknown {
    command_name: String,
    params: Vec<Frame>,
}

impl Unknown {
    /// Create a new `Unknown` command which responds to unknown commands
    /// issued by clients
    pub fn new(key: &impl ToString, parse: &mut Parse) -> Unknown {
        let mut params = vec![];
        while let Ok(t) = parse.next_frame() {
            params.push(t);
        }
        Unknown {
            command_name: key.to_string(),
            params,
        }
    }

    /// Returns the command name
    pub fn _get_name(&self) -> &str {
        &self.command_name
    }

    /// Responds to the client, indicating the command is not recognized.
    ///
    /// This usually means the command is not yet implemented by `rudis`.
    #[tracing::instrument(skip(self))]
    pub fn apply(self) -> Frame {
        Frame::Error(
            format!(
                "ERR unknown command '{}', params: {:?}",
                self.command_name, self.params
            )
            .as_bytes()
            .into(),
        )
    }
}
