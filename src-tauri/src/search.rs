use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use tauri::command;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use once_cell::sync::OnceCell;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub name: String,
    pub appid: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SearchResult {
    score: i64,
    index: usize,
}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher scores should come first
        other.score.cmp(&self.score)
    }
}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

static GAME_LIST_CACHE: OnceCell<Vec<Game>> = OnceCell::new();

/// Fetches and caches the game list from the remote source
async fn get_cached_games() -> Result<&'static Vec<Game>, String> {
    match GAME_LIST_CACHE.get() {
        Some(games) => Ok(games),
        None => {
            println!("[Search] Caching game list for the first time");

            let url = "https://raw.githubusercontent.com/0xSovereign/steamapplist/refs/heads/main/data/apps.json";

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

            let games: Vec<Game> = client
                .get(url)
                .send()
                .await
                .map_err(|e| format!("Failed to fetch game list: {}", e))?
                .json()
                .await
                .map_err(|e| format!("Failed to parse game list: {}", e))?;

            println!(
                "[Search] Game list cached successfully ({} games).",
                games.len()
            );

            // Cache the games list
            GAME_LIST_CACHE
                .set(games)
                .map_err(|_| "Failed to cache game list".to_string())?;

            Ok(GAME_LIST_CACHE.get().unwrap())
        }
    }
}

/// Calculates bonus score based on various factors
fn calculate_bonus_score(game_name: &str, query: &str, fuzzy_score: i64) -> i64 {
    let game_lower = game_name.to_lowercase();
    let query_lower = query.to_lowercase();
    let mut bonus = 0i64;

    // Exact match gets highest bonus
    if game_lower == query_lower {
        bonus += 10000;
    }
    // Starts with query gets high bonus
    else if game_lower.starts_with(&query_lower) {
        bonus += 5000;
    }
    // Contains query as whole word gets medium bonus
    else if game_lower.contains(&format!(" {} ", query_lower))
        || game_lower.starts_with(&format!("{} ", query_lower))
        || game_lower.ends_with(&format!(" {}", query_lower))
    {
        bonus += 2000;
    }
    // Contains query gets smaller bonus
    else if game_lower.contains(&query_lower) {
        bonus += 1000;
    }

    // Prefer shorter names when fuzzy scores are similar
    let length_penalty = (game_name.len() as i64).saturating_sub(query.len() as i64);
    bonus -= length_penalty.min(500); // Cap penalty

    // Boost score based on fuzzy match quality
    bonus += (fuzzy_score as f64 * 0.1) as i64;

    bonus
}

/// Search function for retrieving games based on a title query.
#[command]
pub async fn cmd_get_game(title: String, limit: Option<usize>) -> Result<Vec<Game>, String> {
    let title = title.trim();

    // Return empty results for very short queries
    if title.len() < 2 {
        return Ok(Vec::new());
    }

    let games = get_cached_games().await?;
    let limit = limit.unwrap_or(10).min(50);

    let matcher = SkimMatcherV2::default().smart_case();

    let mut results: Vec<SearchResult> = games
        .iter()
        .enumerate()
        .filter_map(|(idx, game)| {
            // First try fuzzy matching
            if let Some(fuzzy_score) = matcher.fuzzy_match(&game.name, title) {
                let bonus = calculate_bonus_score(&game.name, title, fuzzy_score);
                let total_score = fuzzy_score + bonus;

                Some(SearchResult {
                    score: total_score,
                    index: idx,
                })
            } else {
                let title_lower = title.to_lowercase();
                let query_words: Vec<&str> = title_lower.split_whitespace().collect();
                let game_name_lower = game.name.to_lowercase();

                let all_words_present = query_words
                    .iter()
                    .all(|word| game_name_lower.contains(word));

                if all_words_present && !query_words.is_empty() {
                    let fallback_score = 100i64 - (game.name.len() as i64);
                    Some(SearchResult {
                        score: fallback_score,
                        index: idx,
                    })
                } else {
                    None
                }
            }
        })
        .collect();

    results.sort();

    let final_results: Vec<Game> = results
        .into_iter()
        .take(limit)
        .map(|result| games[result.index].clone())
        .collect();

    println!(
        "[Search] Found {} relevant results for '{}' (showing top {})",
        final_results.len(),
        title,
        limit
    );

    Ok(final_results)
}

/// Force refresh the game list cache
#[command]
pub async fn cmd_refresh_game_cache() -> Result<String, String> {
    println!("[Search] Force refreshing game list cache...");

    let url =
        "https://raw.githubusercontent.com/0xSovereign/steamapplist/refs/heads/main/data/apps.json";

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let games: Vec<Game> = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch game list: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse game list: {}", e))?;

    println!("[Search] Ftched {} games.", games.len());

    Ok(format!("Fetched {} games.", games.len()))
}

/// Get cache statistics
#[command]
pub fn cmd_get_cache_stats() -> Result<serde_json::Value, String> {
    match GAME_LIST_CACHE.get() {
        Some(games) => Ok(serde_json::json!({
            "cached": true,
            "game_count": games.len(),
            "status": "ready"
        })),
        None => Ok(serde_json::json!({
            "cached": false,
            "game_count": 0,
            "status": "not_initialized"
        })),
    }
}

/// Search for games by App ID
#[command]
pub async fn cmd_get_game_by_appid(appid: u32) -> Result<Option<Game>, String> {
    let games = get_cached_games().await?;

    let result = games.iter().find(|game| game.appid == appid).cloned();

    match &result {
        Some(game) => println!("[Search] Found game by App ID {}: {}", appid, game.name),
        None => println!("[Search] No game found with App ID {}", appid),
    }

    Ok(result)
}
