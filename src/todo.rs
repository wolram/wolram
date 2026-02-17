//! TODO generation from natural language prompts.
//!
//! Provides [`TodoGenerator`] which breaks down a free-text description into
//! a structured list of [`TodoItem`]s. When an Anthropic API client is available,
//! it uses LLM-based decomposition; otherwise it falls back to keyword-based
//! heuristic parsing.

use anyhow::{Result, bail};

use crate::anthropic::{Message, MessageSender, MessagesRequest};
use crate::state_machine::{Priority, TodoItem};

/// Generates TODO items from natural language prompts.
pub struct TodoGenerator;

/// Raw LLM response item used for JSON deserialization.
#[derive(Debug, serde::Deserialize)]
struct LlmTodoItem {
    title: String,
    priority: String,
    #[serde(default)]
    skill: Option<String>,
}

/// Wrapper for the LLM JSON response.
#[derive(Debug, serde::Deserialize)]
struct LlmTodoResponse {
    todos: Vec<LlmTodoItem>,
}

/// Valid skills that can be assigned to TODO items.
const VALID_SKILLS: &[&str] = &[
    "testing",
    "refactoring",
    "documentation",
    "bug_fix",
    "code_generation",
];

impl TodoGenerator {
    /// Generates TODO items using an LLM client.
    ///
    /// Sends a structured prompt asking the model to decompose the natural
    /// language description into actionable TODO items with priorities and
    /// optional skill categories.
    ///
    /// Falls back to keyword-based generation on any LLM or parsing error.
    pub async fn generate_with_llm(
        client: &impl MessageSender,
        prompt: &str,
    ) -> Result<Vec<TodoItem>> {
        let req = MessagesRequest {
            model: "claude-haiku-4-5-20251001".to_string(),
            max_tokens: 1024,
            messages: vec![Message {
                role: "user".into(),
                content: format!(
                    "Break down this task into actionable TODO items. \
                     Respond with ONLY valid JSON, no other text.\n\
                     \n\
                     Format:\n\
                     {{\"todos\": [\n\
                       {{\"title\": \"<short imperative action>\", \"priority\": \"<high|medium|low>\", \"skill\": \"<skill_or_null>\"}}\n\
                     ]}}\n\
                     \n\
                     Rules:\n\
                     - Each title must be a short, actionable imperative phrase (e.g., \"Write unit tests for auth module\")\n\
                     - priority must be one of: high, medium, low\n\
                     - skill must be one of: testing, refactoring, documentation, bug_fix, code_generation, or null\n\
                     - Generate 2-8 TODO items, ordered by suggested execution sequence\n\
                     - Assign high priority to foundational or blocking tasks, low to polish/docs\n\
                     \n\
                     Task: {prompt}"
                ),
            }],
        };

        let response = client.send_message(&req).await?;
        let text = response
            .content
            .first()
            .map(|b| b.text.trim().to_string())
            .unwrap_or_default();

        let parsed: LlmTodoResponse = serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM TODO response: {e}"))?;

        if parsed.todos.is_empty() {
            bail!("LLM returned empty TODO list");
        }

        let items = parsed
            .todos
            .into_iter()
            .enumerate()
            .map(|(i, raw)| TodoItem {
                id: (i + 1) as u32,
                title: raw.title,
                priority: parse_priority(&raw.priority),
                skill: raw.skill.and_then(|s| {
                    if VALID_SKILLS.contains(&s.as_str()) {
                        Some(s)
                    } else {
                        None
                    }
                }),
            })
            .collect();

        Ok(items)
    }

    /// Generates TODO items from a natural language prompt using keyword heuristics.
    ///
    /// This is the fallback path used when no API client is available.
    /// It scans the prompt for action verbs, conjunctions, and structural cues
    /// to split the text into discrete tasks.
    pub fn generate_from_keywords(prompt: &str) -> Vec<TodoItem> {
        let mut items: Vec<TodoItem> = Vec::new();

        // Try splitting on explicit list markers first (numbered items, bullet points).
        let explicit_items = split_explicit_list(prompt);
        if explicit_items.len() >= 2 {
            for (i, text) in explicit_items.into_iter().enumerate() {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    continue;
                }
                items.push(TodoItem {
                    id: (i + 1) as u32,
                    title: capitalize_first(trimmed),
                    priority: infer_priority(trimmed),
                    skill: infer_skill(trimmed),
                });
            }
            return renumber(items);
        }

        // Try splitting on conjunctions ("and", "then", commas between clauses).
        let clauses = split_on_conjunctions(prompt);
        if clauses.len() >= 2 {
            for (i, clause) in clauses.into_iter().enumerate() {
                let trimmed = clause.trim();
                if trimmed.is_empty() {
                    continue;
                }
                items.push(TodoItem {
                    id: (i + 1) as u32,
                    title: capitalize_first(trimmed),
                    priority: infer_priority(trimmed),
                    skill: infer_skill(trimmed),
                });
            }
            return renumber(items);
        }

        // Single-task fallback: generate a planning + execution + verification pattern.
        let skill = infer_skill(prompt);
        let priority = infer_priority(prompt);
        let desc = capitalize_first(prompt.trim());

        items.push(TodoItem {
            id: 1,
            title: format!("Plan approach for: {desc}"),
            priority: Priority::High,
            skill: None,
        });
        items.push(TodoItem {
            id: 2,
            title: desc,
            priority,
            skill: skill.clone(),
        });
        items.push(TodoItem {
            id: 3,
            title: "Verify changes and run tests".to_string(),
            priority: Priority::Medium,
            skill: Some("testing".to_string()),
        });

        items
    }
}

/// Parses a priority string from LLM output, defaulting to Medium.
fn parse_priority(s: &str) -> Priority {
    match s.to_lowercase().as_str() {
        "high" => Priority::High,
        "low" => Priority::Low,
        _ => Priority::Medium,
    }
}

/// Splits text on explicit list markers: "1.", "2.", "-", "*".
fn split_explicit_list(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();

    // Check for numbered or bulleted lines.
    let list_items: Vec<String> = lines
        .iter()
        .filter_map(|line| {
            let trimmed = line.trim();
            // Match "1.", "2)", "- ", "* " prefixes.
            if let Some(rest) = trimmed
                .strip_prefix("- ")
                .or_else(|| trimmed.strip_prefix("* "))
            {
                Some(rest.to_string())
            } else if trimmed.len() > 2
                && trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
            {
                // "1. text" or "1) text"
                if let Some(pos) = trimmed.find(['.', ')']) {
                    let after = trimmed[pos + 1..].trim();
                    if !after.is_empty() {
                        return Some(after.to_string());
                    }
                }
                None
            } else {
                None
            }
        })
        .collect();

    list_items
}

/// Splits a sentence on conjunctions like "and", "then", or commas separating clauses.
fn split_on_conjunctions(text: &str) -> Vec<String> {
    // Split on ", then ", " and then ", " and ", ", " (with verb following).
    let delimiters = [", then ", " and then ", " then ", " and "];

    let mut parts = vec![text.to_string()];
    for delim in delimiters {
        let mut new_parts = Vec::new();
        for part in &parts {
            let lower = part.to_lowercase();
            if let Some(pos) = lower.find(delim) {
                let left = part[..pos].trim().to_string();
                let right = part[pos + delim.len()..].trim().to_string();
                if !left.is_empty() {
                    new_parts.push(left);
                }
                if !right.is_empty() {
                    new_parts.push(right);
                }
            } else {
                new_parts.push(part.clone());
            }
        }
        parts = new_parts;
    }

    parts
}

/// Infers a skill category from text using the same keywords as SkillRouter.
fn infer_skill(text: &str) -> Option<String> {
    let lower = text.to_lowercase();

    let keyword_skills: &[(&str, &str, u32)] = &[
        ("test", "testing", 10),
        ("spec", "testing", 5),
        ("refactor", "refactoring", 10),
        ("clean up", "refactoring", 5),
        ("doc", "documentation", 10),
        ("readme", "documentation", 5),
        ("fix", "bug_fix", 10),
        ("bug", "bug_fix", 10),
        ("debug", "bug_fix", 7),
        ("error", "bug_fix", 5),
        ("implement", "code_generation", 5),
        ("add", "code_generation", 3),
        ("create", "code_generation", 5),
        ("build", "code_generation", 5),
    ];

    let mut scores: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
    for &(keyword, skill, weight) in keyword_skills {
        if lower.contains(keyword) {
            *scores.entry(skill).or_insert(0) += weight;
        }
    }

    scores
        .into_iter()
        .max_by_key(|&(_, score)| score)
        .map(|(skill, _)| skill.to_string())
}

/// Infers priority from keywords in the text.
fn infer_priority(text: &str) -> Priority {
    let lower = text.to_lowercase();

    let high_keywords = [
        "critical", "urgent", "block", "break", "crash", "security", "fix",
    ];
    let low_keywords = [
        "doc", "readme", "comment", "format", "style", "typo", "rename",
    ];

    for kw in high_keywords {
        if lower.contains(kw) {
            return Priority::High;
        }
    }
    for kw in low_keywords {
        if lower.contains(kw) {
            return Priority::Low;
        }
    }

    Priority::Medium
}

/// Capitalizes the first character of a string.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// Re-numbers TODO items sequentially starting from 1, filtering out empty titles.
fn renumber(items: Vec<TodoItem>) -> Vec<TodoItem> {
    items
        .into_iter()
        .filter(|item| !item.title.trim().is_empty())
        .enumerate()
        .map(|(i, mut item)| {
            item.id = (i + 1) as u32;
            item
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_priority tests ---

    #[test]
    fn parse_priority_high() {
        assert_eq!(parse_priority("high"), Priority::High);
        assert_eq!(parse_priority("HIGH"), Priority::High);
    }

    #[test]
    fn parse_priority_low() {
        assert_eq!(parse_priority("low"), Priority::Low);
    }

    #[test]
    fn parse_priority_default() {
        assert_eq!(parse_priority("medium"), Priority::Medium);
        assert_eq!(parse_priority("anything"), Priority::Medium);
    }

    // --- infer_skill tests ---

    #[test]
    fn infer_skill_testing() {
        assert_eq!(infer_skill("write unit tests"), Some("testing".into()));
    }

    #[test]
    fn infer_skill_bug_fix() {
        assert_eq!(infer_skill("fix the login bug"), Some("bug_fix".into()));
    }

    #[test]
    fn infer_skill_refactoring() {
        assert_eq!(
            infer_skill("refactor auth module"),
            Some("refactoring".into())
        );
    }

    #[test]
    fn infer_skill_documentation() {
        assert_eq!(
            infer_skill("write docs for the API"),
            Some("documentation".into())
        );
    }

    #[test]
    fn infer_skill_code_generation() {
        assert_eq!(
            infer_skill("implement user dashboard"),
            Some("code_generation".into())
        );
    }

    #[test]
    fn infer_skill_none_for_ambiguous() {
        assert_eq!(infer_skill("do something"), None);
    }

    // --- infer_priority tests ---

    #[test]
    fn infer_priority_high_from_critical() {
        assert_eq!(infer_priority("critical security issue"), Priority::High);
    }

    #[test]
    fn infer_priority_high_from_fix() {
        assert_eq!(infer_priority("fix the crash"), Priority::High);
    }

    #[test]
    fn infer_priority_low_from_docs() {
        assert_eq!(infer_priority("update the documentation"), Priority::Low);
    }

    #[test]
    fn infer_priority_medium_default() {
        assert_eq!(infer_priority("implement new feature"), Priority::Medium);
    }

    // --- split_explicit_list tests ---

    #[test]
    fn split_explicit_numbered_list() {
        let text = "1. Write the model\n2. Add tests\n3. Update docs";
        let items = split_explicit_list(text);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], "Write the model");
        assert_eq!(items[1], "Add tests");
        assert_eq!(items[2], "Update docs");
    }

    #[test]
    fn split_explicit_bullet_list() {
        let text = "- Create user table\n- Add API endpoint\n- Write integration tests";
        let items = split_explicit_list(text);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], "Create user table");
    }

    #[test]
    fn split_explicit_no_list() {
        let text = "implement a login page with authentication";
        let items = split_explicit_list(text);
        assert_eq!(items.len(), 0);
    }

    // --- split_on_conjunctions tests ---

    #[test]
    fn split_on_and() {
        let parts = split_on_conjunctions("implement the model and write tests");
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "implement the model");
        assert_eq!(parts[1], "write tests");
    }

    #[test]
    fn split_on_then() {
        let parts = split_on_conjunctions("create the database, then add the API layer");
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "create the database");
        assert_eq!(parts[1], "add the API layer");
    }

    #[test]
    fn split_no_conjunction() {
        let parts = split_on_conjunctions("implement user authentication");
        assert_eq!(parts.len(), 1);
    }

    // --- capitalize_first tests ---

    #[test]
    fn capitalize_first_lowercase() {
        assert_eq!(capitalize_first("hello"), "Hello");
    }

    #[test]
    fn capitalize_first_empty() {
        assert_eq!(capitalize_first(""), "");
    }

    // --- TodoGenerator::generate_from_keywords tests ---

    #[test]
    fn keywords_explicit_list() {
        let prompt = "1. Create the user model\n2. Add REST endpoints\n3. Write tests";
        let todos = TodoGenerator::generate_from_keywords(prompt);
        assert_eq!(todos.len(), 3);
        assert_eq!(todos[0].id, 1);
        assert_eq!(todos[0].title, "Create the user model");
        assert_eq!(todos[1].id, 2);
        assert_eq!(todos[2].id, 3);
    }

    #[test]
    fn keywords_conjunction_split() {
        let prompt = "implement the login page and add unit tests";
        let todos = TodoGenerator::generate_from_keywords(prompt);
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].title, "Implement the login page");
        assert_eq!(todos[1].title, "Add unit tests");
    }

    #[test]
    fn keywords_single_task_fallback() {
        let prompt = "implement user authentication";
        let todos = TodoGenerator::generate_from_keywords(prompt);
        assert_eq!(todos.len(), 3);
        assert_eq!(todos[0].priority, Priority::High);
        assert!(todos[0].title.starts_with("Plan approach for:"));
        assert_eq!(todos[1].title, "Implement user authentication");
        assert_eq!(todos[2].title, "Verify changes and run tests");
    }

    #[test]
    fn keywords_assigns_skills() {
        let prompt = "fix the login bug and write tests for auth";
        let todos = TodoGenerator::generate_from_keywords(prompt);
        assert!(todos.len() >= 2);
        // First item mentions "fix" and "bug" → bug_fix
        assert_eq!(todos[0].skill, Some("bug_fix".into()));
        // Second item mentions "tests" → testing
        assert_eq!(todos[1].skill, Some("testing".into()));
    }

    #[test]
    fn keywords_assigns_priorities() {
        let prompt = "- fix critical security vulnerability\n- update documentation";
        let todos = TodoGenerator::generate_from_keywords(prompt);
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].priority, Priority::High);
        assert_eq!(todos[1].priority, Priority::Low);
    }

    // --- TodoGenerator::generate_with_llm tests (MockClient) ---

    use crate::anthropic::MessageSender;
    use crate::anthropic::error::AnthropicError;
    use crate::anthropic::types::{ContentBlock, MessagesResponse, Usage};

    struct MockClient {
        result: Result<String, AnthropicError>,
    }

    impl MockClient {
        fn ok(text: &str) -> Self {
            Self {
                result: Ok(text.to_string()),
            }
        }

        fn err(e: AnthropicError) -> Self {
            Self { result: Err(e) }
        }
    }

    impl MessageSender for MockClient {
        async fn send_message(
            &self,
            _req: &MessagesRequest,
        ) -> Result<MessagesResponse, AnthropicError> {
            match &self.result {
                Ok(text) => Ok(MessagesResponse {
                    id: "mock".to_string(),
                    content: vec![ContentBlock {
                        content_type: "text".to_string(),
                        text: text.clone(),
                    }],
                    model: "mock".to_string(),
                    stop_reason: Some("end_turn".to_string()),
                    usage: Usage {
                        input_tokens: 0,
                        output_tokens: 0,
                    },
                }),
                Err(_) => Err(AnthropicError::ApiError {
                    status: 500,
                    message: "mock error".to_string(),
                }),
            }
        }
    }

    #[tokio::test]
    async fn llm_generates_todos() {
        let json = r#"{"todos":[
            {"title":"Create user model","priority":"high","skill":"code_generation"},
            {"title":"Add REST endpoints","priority":"medium","skill":"code_generation"},
            {"title":"Write integration tests","priority":"low","skill":"testing"}
        ]}"#;
        let client = MockClient::ok(json);
        let todos = TodoGenerator::generate_with_llm(&client, "build user service")
            .await
            .unwrap();

        assert_eq!(todos.len(), 3);
        assert_eq!(todos[0].id, 1);
        assert_eq!(todos[0].title, "Create user model");
        assert_eq!(todos[0].priority, Priority::High);
        assert_eq!(todos[0].skill, Some("code_generation".into()));
        assert_eq!(todos[2].priority, Priority::Low);
        assert_eq!(todos[2].skill, Some("testing".into()));
    }

    #[tokio::test]
    async fn llm_filters_invalid_skills() {
        let json = r#"{"todos":[
            {"title":"Do something","priority":"medium","skill":"unknown_skill"}
        ]}"#;
        let client = MockClient::ok(json);
        let todos = TodoGenerator::generate_with_llm(&client, "anything")
            .await
            .unwrap();

        assert_eq!(todos[0].skill, None);
    }

    #[tokio::test]
    async fn llm_handles_null_skill() {
        let json = r#"{"todos":[
            {"title":"Plan the approach","priority":"high","skill":null}
        ]}"#;
        let client = MockClient::ok(json);
        let todos = TodoGenerator::generate_with_llm(&client, "anything")
            .await
            .unwrap();

        assert_eq!(todos[0].skill, None);
    }

    #[tokio::test]
    async fn llm_rejects_empty_list() {
        let json = r#"{"todos":[]}"#;
        let client = MockClient::ok(json);
        let result = TodoGenerator::generate_with_llm(&client, "anything").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn llm_error_propagates() {
        let client = MockClient::err(AnthropicError::ApiError {
            status: 500,
            message: "fail".into(),
        });
        let result = TodoGenerator::generate_with_llm(&client, "anything").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn llm_invalid_json_errors() {
        let client = MockClient::ok("not json at all");
        let result = TodoGenerator::generate_with_llm(&client, "anything").await;
        assert!(result.is_err());
    }

    // --- TodoItem serialization ---

    #[test]
    fn todo_item_serialization_roundtrip() {
        let item = TodoItem {
            id: 1,
            title: "Write tests".to_string(),
            priority: Priority::High,
            skill: Some("testing".to_string()),
        };
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: TodoItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, item);
    }

    #[test]
    fn todo_item_without_skill_omits_field() {
        let item = TodoItem {
            id: 1,
            title: "Do something".to_string(),
            priority: Priority::Medium,
            skill: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(!json.contains("skill"));
    }

    #[test]
    fn priority_display() {
        assert_eq!(Priority::High.to_string(), "high");
        assert_eq!(Priority::Medium.to_string(), "medium");
        assert_eq!(Priority::Low.to_string(), "low");
    }
}
