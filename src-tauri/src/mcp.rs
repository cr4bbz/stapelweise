use crate::db::Database;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

pub fn run_mcp_loop(db: Database) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        if let Ok(req) = serde_json::from_str::<Value>(&line) {
            let id = req["id"].clone();
            let method = req["method"].as_str().unwrap_or("");

            match method {
                "initialize" => {
                    let res = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "protocolVersion": "2024-11-05",
                            "capabilities": {
                                "tools": {}
                            },
                            "serverInfo": {
                                "name": "stapelweise-mcp",
                                "version": "0.1.0"
                            }
                        }
                    });
                    writeln!(stdout, "{}", res.to_string()).unwrap();
                    stdout.flush().unwrap();
                }
                "notifications/initialized" => {
                    // ignore
                }
                "tools/list" => {
                    let res = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "tools": [
                                {
                                    "name": "list_decks",
                                    "description": "List all learning decks in Stapelweise",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {}
                                    }
                                },
                                {
                                    "name": "search_cards",
                                    "description": "Search flashcards by query string",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "query": { "type": "string" }
                                        },
                                        "required": ["query"]
                                    }
                                },
                                {
                                    "name": "get_deck_stats",
                                    "description": "Get statistics and progress for a specific deck",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "deck_id": { "type": "string" }
                                        },
                                        "required": ["deck_id"]
                                    }
                                },
                                {
                                    "name": "add_card",
                                    "description": "Add a new flashcard to a deck",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "deck_id": { "type": "string" },
                                            "front": { "type": "string" },
                                            "back": { "type": "string" },
                                            "reasoning": { "type": "string", "description": "Optional 'Why?' explanation" }
                                        },
                                        "required": ["deck_id", "front", "back"]
                                    }
                                }
                            ]
                        }
                    });
                    writeln!(stdout, "{}", res.to_string()).unwrap();
                    stdout.flush().unwrap();
                }
                "tools/call" => {
                    let tool_name = req["params"]["name"].as_str().unwrap_or("");
                    let content = if tool_name == "list_decks" {
                        match db.repo.list_decks() {
                            Ok(decks) => json!(decks).to_string(),
                            Err(e) => format!("Error: {}", e),
                        }
                    } else if tool_name == "search_cards" {
                        let query = req["params"]["arguments"]["query"].as_str().unwrap_or("");
                        match db.repo.search_cards(query) {
                            Ok(cards) => json!(cards).to_string(),
                            Err(e) => format!("Error: {}", e),
                        }
                    } else if tool_name == "get_deck_stats" {
                        let deck_id = req["params"]["arguments"]["deck_id"].as_str().unwrap_or("");
                        let today_start =
                            chrono::Local::now().format("%Y-%m-%dT00:00:00").to_string();
                        match db.repo.get_deck_stats(deck_id, &today_start) {
                            Ok(stats) => json!(stats).to_string(),
                            Err(e) => format!("Error: {}", e),
                        }
                    } else if tool_name == "add_card" {
                        let deck_id = req["params"]["arguments"]["deck_id"].as_str().unwrap_or("");
                        let front = req["params"]["arguments"]["front"].as_str().unwrap_or("");
                        let back = req["params"]["arguments"]["back"].as_str().unwrap_or("");
                        let reasoning = req["params"]["arguments"]["reasoning"].as_str();

                        match db.repo.create_card(
                            deck_id,
                            "basic",
                            None,
                            reasoning,
                            front,
                            back,
                            vec![],
                        ) {
                            Ok(card) => json!(card).to_string(),
                            Err(e) => format!("Error: {}", e),
                        }
                    } else {
                        "Unknown tool".to_string()
                    };

                    let res = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": content
                                }
                            ]
                        }
                    });
                    writeln!(stdout, "{}", res.to_string()).unwrap();
                    stdout.flush().unwrap();
                }
                _ => {
                    // Send error or ignore
                }
            }
        }
    }
}
