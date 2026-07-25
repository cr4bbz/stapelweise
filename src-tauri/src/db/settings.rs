use super::repository::Repository;
use crate::srs::sm2::Sm2Config;

fn default_learning_animations() -> bool {
    true
}

fn default_show_deck_card_previews() -> bool {
    true
}

fn default_timer_slider_scale() -> String {
    "logarithmic".into()
}

fn default_timer_min_minutes() -> u32 {
    5
}

fn default_timer_max_minutes() -> u32 {
    240
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
    pub theme: String,
    #[serde(default = "default_color_theme")]
    pub color_theme: String,
    #[serde(default = "default_custom_primary_color")]
    pub custom_primary_color: String,
    #[serde(default = "default_custom_secondary_color")]
    pub custom_secondary_color: String,
    #[serde(default = "default_module_color_assignments")]
    pub module_color_assignments: String,
    #[serde(default = "default_module_surface")]
    pub module_surface: String,
    #[serde(default = "default_show_deck_card_previews")]
    pub show_deck_card_previews: bool,
    #[serde(default = "default_pixel_font")]
    pub pixel_font: String,
    pub card_font_family: String,
    pub card_font_size: String,
    #[serde(default = "default_learning_animations")]
    pub learning_animations: bool,
    #[serde(default = "default_learning_animations")]
    pub card_flip_animation: bool,
    #[serde(default = "default_learning_animations")]
    pub control_transition_animation: bool,
    #[serde(default = "default_learning_animations")]
    pub rating_buttons_animation: bool,
    pub session_limit: u32,
    #[serde(default = "default_timer_slider_scale")]
    pub timer_slider_scale: String,
    #[serde(default = "default_timer_min_minutes")]
    pub timer_min_minutes: u32,
    #[serde(default = "default_timer_max_minutes")]
    pub timer_max_minutes: u32,
    pub sm2_initial_ef: f64,
    pub sm2_pass_threshold: u8,
    pub obsidian_vault_path: String,
    pub obsidian_flashcard_tag: String,
}

fn default_ui_language() -> String {
    "de".into()
}

fn default_color_theme() -> String {
    "academy".into()
}

fn default_custom_primary_color() -> String {
    "#2563eb".into()
}

fn default_custom_secondary_color() -> String {
    "#059669".into()
}

fn default_module_color_assignments() -> String {
    r#"{"brand":"primary","deck":"primary","single_card":"primary","timer":"secondary","tags":"primary","settings":"secondary","archive":"secondary","exam":"secondary"}"#.into()
}

fn default_module_surface() -> String {
    "glass".into()
}

fn default_pixel_font() -> String {
    "press-start".into()
}

impl AppSettings {
    pub fn defaults() -> Self {
        Self {
            ui_language: default_ui_language(),
            theme: "light".into(),
            color_theme: default_color_theme(),
            custom_primary_color: default_custom_primary_color(),
            custom_secondary_color: default_custom_secondary_color(),
            module_color_assignments: default_module_color_assignments(),
            module_surface: default_module_surface(),
            show_deck_card_previews: default_show_deck_card_previews(),
            pixel_font: default_pixel_font(),
            card_font_family: "serif".into(),
            card_font_size: "medium".into(),
            learning_animations: true,
            card_flip_animation: true,
            control_transition_animation: true,
            rating_buttons_animation: true,
            session_limit: 50,
            timer_slider_scale: default_timer_slider_scale(),
            timer_min_minutes: default_timer_min_minutes(),
            timer_max_minutes: default_timer_max_minutes(),
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
                "ui_language" => s.ui_language = value,
                "theme" => s.theme = "light".into(),
                "color_theme" => s.color_theme = value,
                "custom_primary_color" => s.custom_primary_color = value,
                "custom_secondary_color" => s.custom_secondary_color = value,
                "module_color_assignments" => s.module_color_assignments = value,
                "module_surface" => s.module_surface = "glass".into(),
                "show_deck_card_previews" => {
                    s.show_deck_card_previews = value.parse().unwrap_or(s.show_deck_card_previews)
                }
                "pixel_font" => s.pixel_font = value,
                "card_font_family" => s.card_font_family = value,
                "card_font_size" => s.card_font_size = value,
                "learning_animations" => {
                    s.learning_animations = value.parse().unwrap_or(s.learning_animations)
                }
                "card_flip_animation" => {
                    s.card_flip_animation = value.parse().unwrap_or(s.card_flip_animation)
                }
                "control_transition_animation" => {
                    s.control_transition_animation =
                        value.parse().unwrap_or(s.control_transition_animation)
                }
                "rating_buttons_animation" => {
                    s.rating_buttons_animation = value.parse().unwrap_or(s.rating_buttons_animation)
                }
                "session_limit" => s.session_limit = value.parse().unwrap_or(s.session_limit),
                "timer_slider_scale" => {
                    if matches!(value.as_str(), "linear" | "logarithmic") {
                        s.timer_slider_scale = value;
                    }
                }
                "timer_min_minutes" => {
                    s.timer_min_minutes = value.parse().unwrap_or(s.timer_min_minutes)
                }
                "timer_max_minutes" => {
                    s.timer_max_minutes = value.parse().unwrap_or(s.timer_max_minutes)
                }
                "sm2_initial_ef" => s.sm2_initial_ef = value.parse().unwrap_or(s.sm2_initial_ef),
                "sm2_pass_threshold" => {
                    s.sm2_pass_threshold = value
                        .parse()
                        .map(|threshold: u8| threshold.clamp(1, 4))
                        .unwrap_or(s.sm2_pass_threshold)
                }
                "obsidian_vault_path" => s.obsidian_vault_path = value,
                "obsidian_flashcard_tag" => s.obsidian_flashcard_tag = value,
                _ => {}
            }
        }
        s.timer_min_minutes = s.timer_min_minutes.clamp(1, 475);
        s.timer_max_minutes = s.timer_max_minutes.clamp(s.timer_min_minutes + 5, 480);
        Ok(s)
    }

    pub fn save(&self, repo: &Repository) -> Result<(), rusqlite::Error> {
        repo.set_setting("ui_language", &self.ui_language)?;
        repo.set_setting("theme", "light")?;
        repo.set_setting("color_theme", &self.color_theme)?;
        repo.set_setting("custom_primary_color", &self.custom_primary_color)?;
        repo.set_setting("custom_secondary_color", &self.custom_secondary_color)?;
        repo.set_setting("module_color_assignments", &self.module_color_assignments)?;
        repo.set_setting("module_surface", "glass")?;
        repo.set_setting(
            "show_deck_card_previews",
            &self.show_deck_card_previews.to_string(),
        )?;
        repo.set_setting("pixel_font", &self.pixel_font)?;
        repo.set_setting("card_font_family", &self.card_font_family)?;
        repo.set_setting("card_font_size", &self.card_font_size)?;
        repo.set_setting("learning_animations", &self.learning_animations.to_string())?;
        repo.set_setting("card_flip_animation", &self.card_flip_animation.to_string())?;
        repo.set_setting(
            "control_transition_animation",
            &self.control_transition_animation.to_string(),
        )?;
        repo.set_setting(
            "rating_buttons_animation",
            &self.rating_buttons_animation.to_string(),
        )?;
        repo.set_setting("session_limit", &self.session_limit.to_string())?;
        repo.set_setting("timer_slider_scale", &self.timer_slider_scale)?;
        repo.set_setting("timer_min_minutes", &self.timer_min_minutes.to_string())?;
        repo.set_setting("timer_max_minutes", &self.timer_max_minutes.to_string())?;
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

#[cfg(test)]
mod tests {
    use super::AppSettings;
    use crate::db::migrations;
    use crate::db::repository::Repository;
    use rusqlite::Connection;

    #[test]
    fn dashboard_appearance_settings_persist() {
        let conn = Connection::open_in_memory().unwrap();
        migrations::run_migrations(&conn).unwrap();
        let repo = Repository::new(conn);
        let mut settings = AppSettings::defaults();
        settings.module_surface = "glass".to_string();
        settings.show_deck_card_previews = false;
        settings.timer_slider_scale = "linear".to_string();
        settings.timer_min_minutes = 10;
        settings.timer_max_minutes = 180;

        settings.save(&repo).unwrap();

        let loaded = AppSettings::load(&repo).unwrap();
        assert_eq!(loaded.module_surface, "glass");
        assert!(!loaded.show_deck_card_previews);
        assert_eq!(loaded.timer_slider_scale, "linear");
        assert_eq!(loaded.timer_min_minutes, 10);
        assert_eq!(loaded.timer_max_minutes, 180);
    }
}
