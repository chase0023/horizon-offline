//! Offline profiles: local accounts that need no Microsoft sign-in. A profile is a name, the
//! Minecraft offline UUID that name derives (vanilla's `UUID.nameUUIDFromBytes("OfflinePlayer:"
//! + name)`, the same one the game computes), and when it was made. Nothing here authenticates:
//! an offline profile never gets a session file, a token or access to the Minecraft services.
//!
//! The game already plays without a session (`--name`, its offline UUID), so an offline launch
//! only hands it the profile's name. Single-player and LAN work; online-mode servers still need
//! a Microsoft account.

use std::time::{SystemTime, UNIX_EPOCH};

/// A local profile saved in `accounts.json`. Never a Microsoft identity.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OfflineProfile {
    /// The Minecraft offline UUID, without dashes: what the game uses for this name.
    pub uuid: String,
    pub name: String,
    /// Unix seconds, for sorting and for the UI ("made …").
    #[serde(default)]
    pub created_at: u64,
}

/// What a name the game would refuse costs the player, in one line.
pub const NAME_RULE: &str = "Use 3 to 16 letters, digits or underscores for an offline name.";
/// Another profile (or a signed-in Microsoft account) already has the name.
pub const NAME_TAKEN: &str = "That name is already used. Pick another one.";

impl OfflineProfile {
    /// A profile for `name`; the caller checks it is free first.
    pub fn new(name: &str) -> Result<Self, String> {
        validate_name(name)?;
        Ok(Self {
            uuid: uuid_for(name),
            name: name.to_owned(),
            created_at: now(),
        })
    }
}

/// Vanilla's offline UUID for `name`, as the game computes it.
pub fn uuid_for(name: &str) -> String {
    format!("{:032x}", horizon_platform::protocol::offline_uuid(name))
}

/// StringUtil.isValidPlayerName's rule, as launchers know it: 3 to 16 letters, digits or
/// underscores. The game's own offline names are looser; this keeps a profile usable on servers
/// and distinct from another profile's.
pub fn validate_name(name: &str) -> Result<(), String> {
    let ok = (3..=16).contains(&name.len())
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_');
    if ok { Ok(()) } else { Err(NAME_RULE.into()) }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_checked_like_minecraft_names() {
        for good in ["Builder", "a_b", "ABC_123", "x".repeat(16).as_str()] {
            assert!(validate_name(good).is_ok(), "{good}");
        }
        for bad in [
            "",
            "ab",
            "has space",
            "dot.name",
            "dash-name",
            "été",
            "line\nbreak",
            "x".repeat(17).as_str(),
        ] {
            assert_eq!(validate_name(bad).unwrap_err(), NAME_RULE, "{bad:?}");
        }
    }

    #[test]
    fn a_profiles_uuid_is_vanillas_offline_uuid_of_its_name() {
        let profile = OfflineProfile::new("Builder").unwrap();
        assert_eq!(profile.name, "Builder");
        assert_eq!(profile.uuid, uuid_for("Builder"));
        assert_eq!(profile.uuid.len(), 32);
        assert!(profile.uuid.bytes().all(|b| b.is_ascii_hexdigit()));
        // The game computes the same id from the name it is given.
        assert_eq!(
            profile.uuid,
            format!(
                "{:032x}",
                horizon_platform::protocol::offline_uuid("Builder")
            )
        );
        // Different names, different identities.
        assert_ne!(profile.uuid, OfflineProfile::new("Builder2").unwrap().uuid);
        assert_ne!(profile.uuid, OfflineProfile::new("builder").unwrap().uuid);
    }
}
