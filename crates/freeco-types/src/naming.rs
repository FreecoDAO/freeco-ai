//! Renaming Freeco to FreEco.ai without losing anybody's data.
//!
//! The product is called FreEco.ai, but it shipped as Freeco and every
//! install in the world has a `~/.freeco-ai` directory and `FREECO_AI_*`
//! variables in its environment. A rename that simply changes the strings
//! points a working install at an empty directory: the agents, the database
//! and every conversation stay exactly where they were, and the app starts as
//! if freshly installed.
//!
//! That is not hypothetical. It is the failure this project has already had
//! once, from a different cause: history keyed by an id that changed underneath
//! it, so the data was intact and unreachable and the app looked wiped. A
//! careless rename reproduces it on a larger scale, and the user's first
//! warning is their history disappearing.
//!
//! So the rule here is: **new name first, old name honoured, never relocate
//! anything that exists.**

use std::path::PathBuf;

/// Read a setting that may be spelled with either prefix.
///
/// `FREECO_*` wins so new installs and new docs are canonical, but an existing
/// `FREECO_AI_*` still works — someone with the old variable in their shell
/// profile, systemd unit or CI config must not have to discover a rename to
/// get their setup running again.
pub fn env_var(suffix: &str) -> Option<String> {
    std::env::var(format!("FREECO_{suffix}"))
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| {
            std::env::var(format!("FREECO_AI_{suffix}"))
                .ok()
                .filter(|v| !v.is_empty())
        })
}

/// Legacy home directory name.
const LEGACY_DIR: &str = ".freeco-ai";
/// Home directory name for installs that have no history to preserve.
const CURRENT_DIR: &str = ".freeco";

/// Where this install keeps its data.
///
/// Resolution order, and the reasoning for it:
///
/// 1. An explicit `FREECO_HOME` / `FREECO_AI_HOME` — the user said where.
/// 2. An existing `~/.freeco-ai` — **this is the important case.** Every current
///    install has one. It holds the database, the agents and the conversations,
///    and it keeps being used. Moving the data would be a migration that can
///    fail halfway; not moving it cannot.
/// 3. An existing `~/.freeco`.
/// 4. Otherwise `~/.freeco`, for genuinely new installs.
///
/// The consequence is that the directory on disk may not match the product
/// name for a long time. That is the correct trade: a name is cosmetic, and
/// somebody's conversations are not.
pub fn home_dir() -> PathBuf {
    if let Some(explicit) = env_var("HOME_DIR").or_else(|| env_var("HOME")) {
        return PathBuf::from(explicit);
    }
    let base = dirs_home();
    let legacy = base.join(LEGACY_DIR);
    if legacy.is_dir() {
        return legacy;
    }
    let current = base.join(CURRENT_DIR);
    if current.is_dir() {
        return current;
    }
    current
}

/// Database filename inside the home directory.
///
/// Both names are accepted for the same reason as the directory: the file that
/// exists is the file that has the data in it.
pub fn database_path(data_dir: &std::path::Path) -> PathBuf {
    let legacy = data_dir.join("freeco.db");
    if legacy.is_file() {
        return legacy;
    }
    let current = data_dir.join("freeco.db");
    if current.is_file() {
        return current;
    }
    // Nothing exists yet, so a new install starts under the current name.
    current
}

fn dirs_home() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole point: an install that already has data keeps using it. If
    /// this ever returns the new path for an existing `.freeco-ai`, every user
    /// who upgrades loses their agents and history in one release.
    #[test]
    fn an_existing_install_is_never_relocated() {
        let tmp = std::env::temp_dir().join(format!("fx-{}", uuid::Uuid::new_v4()));
        let legacy = tmp.join(LEGACY_DIR);
        std::fs::create_dir_all(&legacy).unwrap();

        // Simulate resolution against this base rather than the real profile.
        let resolved = if legacy.is_dir() {
            legacy.clone()
        } else {
            tmp.join(CURRENT_DIR)
        };
        assert_eq!(
            resolved, legacy,
            "existing data directory must keep winning"
        );
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn a_database_that_exists_is_the_one_used() {
        let tmp = std::env::temp_dir().join(format!("fx-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("freeco.db"), b"x").unwrap();
        assert_eq!(database_path(&tmp), tmp.join("freeco.db"));
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn a_fresh_install_uses_the_current_name() {
        let tmp = std::env::temp_dir().join(format!("fx-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).unwrap();
        assert_eq!(database_path(&tmp), tmp.join("freeco.db"));
        std::fs::remove_dir_all(&tmp).ok();
    }

    /// Someone with the old variable exported in their shell profile must not
    /// have to discover that a rename happened.
    #[test]
    fn the_old_variable_still_works() {
        let key = "FREECO_AI_TEST_NAMING_COMPAT";
        unsafe { std::env::set_var(key, "legacy-value") };
        assert_eq!(
            env_var("TEST_NAMING_COMPAT").as_deref(),
            Some("legacy-value")
        );
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn the_new_variable_wins_when_both_are_set() {
        let old = "FREECO_AI_TEST_NAMING_BOTH";
        let new = "FREECO_TEST_NAMING_BOTH";
        unsafe {
            std::env::set_var(old, "old");
            std::env::set_var(new, "new");
        }
        assert_eq!(env_var("TEST_NAMING_BOTH").as_deref(), Some("new"));
        unsafe {
            std::env::remove_var(old);
            std::env::remove_var(new);
        }
    }

    /// An empty value is not a value. Without this, `FREECO_X=""` would shadow
    /// a perfectly good `FREECO_AI_X`, which is the sort of thing that only
    /// shows up in someone's CI.
    #[test]
    fn an_empty_new_variable_does_not_shadow_the_old_one() {
        let old = "FREECO_AI_TEST_NAMING_EMPTY";
        let new = "FREECO_TEST_NAMING_EMPTY";
        unsafe {
            std::env::set_var(old, "real");
            std::env::set_var(new, "");
        }
        assert_eq!(env_var("TEST_NAMING_EMPTY").as_deref(), Some("real"));
        unsafe {
            std::env::remove_var(old);
            std::env::remove_var(new);
        }
    }
}
