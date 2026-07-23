use super::CommandError;
use crate::db::models::Deck;
use crate::db::DbState;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationImportResult {
    pub deck: Deck,
    pub imported: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone)]
struct ImportedCard {
    front: String,
    back: String,
    tags: Vec<String>,
}

fn http_client() -> Result<Client, CommandError> {
    Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|_| {
            CommandError::import_failed("Netzwerkverbindung konnte nicht eingerichtet werden", None)
        })
}

fn clean_text(value: &str) -> String {
    let with_breaks = value
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</p>", "\n")
        .replace("</div>", "\n")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"");
    let mut text = String::with_capacity(with_breaks.len());
    let mut inside_tag = false;
    for character in with_breaks.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => text.push(character),
            _ => {}
        }
    }
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn import_cards(
    state: State<DbState>,
    deck_name: String,
    cards: Vec<ImportedCard>,
) -> Result<IntegrationImportResult, CommandError> {
    if deck_name.trim().is_empty() {
        return Err(CommandError::validation(
            "Bitte wähle einen Namen für den Zielstapel.",
        ));
    }
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {e}")))?;
    let deck = db.repo.get_or_create_deck(deck_name.trim())?;
    let mut imported = 0;
    let mut skipped = 0;
    for card in cards {
        if card.front.trim().is_empty() || card.back.trim().is_empty() {
            skipped += 1;
            continue;
        }
        if db
            .repo
            .card_exists_with_content(&deck.id, &card.front, &card.back)?
        {
            skipped += 1;
            continue;
        }
        db.repo.create_card(
            &deck.id,
            "basic",
            None,
            None,
            &card.front,
            &card.back,
            card.tags,
        )?;
        imported += 1;
    }
    Ok(IntegrationImportResult {
        deck,
        imported,
        skipped,
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn import_zotero_local(
    state: State<DbState>,
    deck_name: String,
    collection_key: Option<String>,
    limit: Option<u32>,
) -> Result<IntegrationImportResult, CommandError> {
    let limit = limit.unwrap_or(100).clamp(1, 500);
    let endpoint = collection_key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(|key| format!("http://localhost:23119/api/users/0/collections/{key}/items/top"))
        .unwrap_or_else(|| "http://localhost:23119/api/users/0/items/top".to_string());
    let response = http_client()?
        .get(endpoint)
        .query(&[("format", "json"), ("limit", &limit.to_string())])
        .send()
        .map_err(|_| {
            CommandError::import_failed(
                "Zotero Desktop wurde nicht erreicht. Öffne Zotero und aktiviere die lokale API.",
                None,
            )
        })?;
    if !response.status().is_success() {
        return Err(CommandError::import_failed(
            "Zotero konnte die Bibliothek nicht bereitstellen.",
            None,
        ));
    }
    let items: Value = response.json().map_err(|_| {
        CommandError::import_failed("Zotero hat ein unerwartetes Datenformat geliefert.", None)
    })?;
    let cards = items
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let data = item.get("data")?;
            let title = clean_text(data.get("title")?.as_str()?);
            let abstract_note = clean_text(
                data.get("abstractNote")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
            );
            if title.is_empty() || abstract_note.is_empty() {
                return None;
            }
            let mut tags = vec!["zotero".to_string()];
            if let Some(item_type) = data.get("itemType").and_then(Value::as_str) {
                tags.push(format!("zotero-{}", item_type.to_lowercase()));
            }
            Some(ImportedCard {
                front: title,
                back: abstract_note,
                tags,
            })
        })
        .collect();
    import_cards(state, deck_name, cards)
}

fn notion_property_text(property: &Value) -> String {
    let property_type = property.get("type").and_then(Value::as_str).unwrap_or("");
    let rich_text = match property_type {
        "title" => property.get("title"),
        "rich_text" => property.get("rich_text"),
        _ => None,
    };
    if let Some(parts) = rich_text.and_then(Value::as_array) {
        return parts
            .iter()
            .filter_map(|part| part.get("plain_text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("");
    }
    match property_type {
        "number" => property
            .get("number")
            .and_then(Value::as_f64)
            .map(|value| value.to_string()),
        "url" | "email" | "phone_number" => property
            .get(property_type)
            .and_then(Value::as_str)
            .map(str::to_string),
        "select" | "status" => property
            .get(property_type)
            .and_then(|value| value.get("name"))
            .and_then(Value::as_str)
            .map(str::to_string),
        "multi_select" => property
            .get("multi_select")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.get("name").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join(", ")
            }),
        _ => None,
    }
    .unwrap_or_default()
}

#[tauri::command(rename_all = "camelCase")]
pub fn import_notion_data_source(
    state: State<DbState>,
    token: String,
    data_source_id: String,
    deck_name: String,
    front_property: String,
    back_property: String,
) -> Result<IntegrationImportResult, CommandError> {
    if token.trim().is_empty() || data_source_id.trim().is_empty() {
        return Err(CommandError::validation(
            "Notion-Token und Datenquellen-ID werden benötigt.",
        ));
    }
    let mut headers = HeaderMap::new();
    let auth_value = HeaderValue::from_str(&format!("Bearer {}", token.trim()))
        .map_err(|_| CommandError::validation("Der Notion-Token ist ungültig."))?;
    headers.insert(AUTHORIZATION, auth_value);
    headers.insert("Notion-Version", HeaderValue::from_static("2026-03-11"));
    let response = http_client()?
        .post(format!(
            "https://api.notion.com/v1/data_sources/{}/query",
            data_source_id.trim()
        ))
        .headers(headers)
        .json(&serde_json::json!({"page_size": 100}))
        .send()
        .map_err(|_| CommandError::import_failed("Notion wurde nicht erreicht.", None))?;
    if !response.status().is_success() {
        return Err(CommandError::import_failed(
            "Notion hat den Zugriff abgelehnt. Prüfe Token, Freigabe und Datenquellen-ID.",
            None,
        ));
    }
    let body: Value = response.json().map_err(|_| {
        CommandError::import_failed("Notion hat ein unerwartetes Datenformat geliefert.", None)
    })?;
    let cards = body
        .get("results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|page| {
            let properties = page.get("properties")?.as_object()?;
            let front = clean_text(&notion_property_text(
                properties.get(front_property.trim())?,
            ));
            let back = clean_text(&notion_property_text(properties.get(back_property.trim())?));
            (!front.is_empty() && !back.is_empty()).then(|| ImportedCard {
                front,
                back,
                tags: vec!["notion".to_string()],
            })
        })
        .collect();
    import_cards(state, deck_name, cards)
}

#[tauri::command(rename_all = "camelCase")]
pub fn import_moodle_glossary(
    state: State<DbState>,
    base_url: String,
    token: String,
    glossary_id: u32,
    deck_name: String,
) -> Result<IntegrationImportResult, CommandError> {
    if base_url.trim().is_empty() || token.trim().is_empty() || glossary_id == 0 {
        return Err(CommandError::validation(
            "Moodle-URL, Token und Glossar-Aktivitäts-ID werden benötigt.",
        ));
    }
    let endpoint = format!(
        "{}/webservice/rest/server.php",
        base_url.trim().trim_end_matches('/')
    );
    let response = http_client()?
        .post(endpoint)
        .form(&[
            ("wstoken", token.trim()),
            ("wsfunction", "mod_glossary_get_entries_by_letter"),
            ("moodlewsrestformat", "json"),
            ("cmid", &glossary_id.to_string()),
            ("letter", "ALL"),
            ("sortorder", "ASC"),
            ("sortkey", "CREATION"),
            ("from", "0"),
            ("limit", "1000"),
        ])
        .send()
        .map_err(|_| CommandError::import_failed("Moodle wurde nicht erreicht.", None))?;
    if !response.status().is_success() {
        return Err(CommandError::import_failed(
            "Moodle hat die Anfrage abgelehnt.",
            None,
        ));
    }
    let body: Value = response.json().map_err(|_| {
        CommandError::import_failed("Moodle hat ein unerwartetes Datenformat geliefert.", None)
    })?;
    if body.get("exception").is_some() {
        return Err(CommandError::import_failed(
            "Moodle konnte das Glossar nicht lesen. Prüfe Token und Aktivitäts-ID.",
            None,
        ));
    }
    let cards = body
        .get("entries")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let front = clean_text(entry.get("concept")?.as_str()?);
            let back = clean_text(entry.get("definition")?.as_str()?);
            (!front.is_empty() && !back.is_empty()).then(|| ImportedCard {
                front,
                back,
                tags: vec!["moodle".to_string(), "moodle-glossar".to_string()],
            })
        })
        .collect();
    import_cards(state, deck_name, cards)
}

#[cfg(test)]
mod tests {
    use super::{clean_text, notion_property_text};
    use serde_json::json;

    #[test]
    fn cleans_common_html_from_external_sources() {
        assert_eq!(
            clean_text("<p>Eine <b>Antwort</b><br>mit Zeile</p>"),
            "Eine Antwort\nmit Zeile"
        );
    }

    #[test]
    fn reads_notion_rich_text_properties() {
        let value = json!({
            "type": "rich_text",
            "rich_text": [{"plain_text": "Erste "}, {"plain_text": "Antwort"}]
        });
        assert_eq!(notion_property_text(&value), "Erste Antwort");
    }
}
