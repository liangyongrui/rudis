use tracing::{instrument};

use crate::{Frame};

/// Represents an "unknown" command. This is not a real `Redis` command.
#[derive(Debug)]
pub struct Unknown {
    command_name: String,
}

impl Unknown {
    /// Create a new `Unknown` command which responds to unknown commands
    /// issued by clients
    pub fn new(key: impl ToString) -> Unknown {
        Unknown {
            command_name: key.to_string(),
        }
    }

    /// Returns the command name
    pub fn _get_name(&self) -> &str {
        &self.command_name
    }

    /// Responds to the client, indicating the command is not recognized.
    ///
    /// This usually means the command is not yet implemented by `rcc`.
    #[instrument(skip(self))]
    pub async fn apply(self) -> crate::Result<Frame> {
        let response = Frame::Error(format!("ERR unknown command '{}'", self.command_name));
        Ok(response)
    }
}
