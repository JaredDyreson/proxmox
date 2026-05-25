//! This module outlines shared enums across each back-end

/// Services that are checked
#[derive(serde::Deserialize)]
pub enum Backend {
    /// Proxmox Backup Server
    Pbs,
    PiHole,
    PfSense,
}

/// Sub-set of ::std::result::Result
pub enum Status {
    /// Err-variant of the backup check
    Bad {
        /// Contents of the error so the client knows what happened
        message: String,
    },
    /// Ok-variant. Nothing to return here.
    Ok,
}

impl serde::Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let inner = match self {
            Self::Bad { message } => format!("BAD: {message}"),
            Self::Ok => String::from("OK"),
        };
        serde_json::json!({"status": inner}).serialize(serializer)
    }
}

impl Backend {
    /// Run the check for the specific service we're interested in
    pub fn status(&self) -> Status {
        match self {
            Self::Pbs => crate::backends::pbs::check(),
            Self::PiHole => crate::backends::pihole::check(),
            Self::PfSense => crate::backends::pfsense::check(),
        }
        .map_or_else(
            |error| Status::Bad {
                message: error.to_string(),
            },
            |()| Status::Ok,
        )
    }
}
