use tracing::{debug, instrument};

use crate::{Connection, Frame};

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
    #[instrument(skip(self, dst))]
    pub async fn apply(self, dst: &mut Connection) -> crate::Result<()> {
        let response = Frame::Error(format!("ERR unknown command '{}'", self.command_name));

        debug!(?response);

        dst.write_frame(&response).await?;
        Ok(())
    }
}
