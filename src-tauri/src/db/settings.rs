use super::repository::Repository;
use crate::srs::sm2::Sm2Config;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub card_font_family: String,
    pub card_font_size: String,
    pub session_limit: u32,
    pub sm2_initial_ef: f64,
    pub sm2_pass_threshold: u8,
    pub obsidian_vault_path: String,
    pub obsidian_flashcard_tag: String,
}

impl AppSettings {
    pub fn defaults() -> Self {
        Self {
            theme: "auto".into(),
            card_font_family: "serif".into(),
            card_font_size: "medium".into(),
            session_limit: 50,
            sm2_initial_ef: 2.5,
            sm2_pass_threshold: 3,
            obsidian_vault_path: "".into(),
            obsidian_flashcard_tag: "#flashcard".into(),
        }
    }

    /// Load settings from DB using a single batch query.
    pub fn load(repo: &Repository) -> Result<Self, rusqlite::Error> {
        let mut s = Self::defaults();
        let rows = repo.get_all_settings()?;
        for (key, value) in rows {
            match key.as_str() {
                "theme" => s.theme = value,
                "card_font_family" => s.card_font_family = value,
                "card_font_size" => s.card_font_size = value,
                "session_limit" => s.session_limit = value.parse().unwrap_or(s.session_limit),
                "sm2_initial_ef" => s.sm2_initial_ef = value.parse().unwrap_or(s.sm2_initial_ef),
                "sm2_pass_threshold" => s.sm2_pass_threshold = value.parse().unwrap_or(s.sm2_pass_threshold),
                "obsidian_vault_path" => s.obsidian_vault_path = value,
                "obsidian_flashcard_tag" => s.obsidian_flashcard_tag = value,
                _ => {}
            }
        }
        Ok(s)
    }

    pub fn save(&self, repo: &Repository) -> Result<(), rusqlite::Error> {
        repo.set_setting("theme", &self.theme)?;
        repo.set_setting("card_font_family", &self.card_font_family)?;
        repo.set_setting("card_font_size", &self.card_font_size)?;
        repo.set_setting("session_limit", &self.session_limit.to_string())?;
        repo.set_setting("sm2_initial_ef", &self.sm2_initial_ef.to_string())?;
        repo.set_setting("sm2_pass_threshold", &self.sm2_pass_threshold.to_string())?;
        repo.set_setting("obsidian_vault_path", &self.obsidian_vault_path)?;
        repo.set_setting("obsidian_flashcard_tag", &self.obsidian_flashcard_tag)?;
        Ok(())
    }

    pub fn to_sm2_config(&self) -> Sm2Config {
        Sm2Config {
            initial_ease_factor: self.sm2_initial_ef,
            min_ease_factor: 1.3,
            ease_penalty: 0.2,
            pass_threshold: self.sm2_pass_threshold,
            intervals: [1, 6],
        }
    }
}
