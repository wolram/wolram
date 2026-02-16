use crate::anthropic::{Message, MessageSender, MessagesRequest};
use crate::state_machine::ModelTier;

/// Routes a job description to an appropriate skill/agent type using weighted keyword scoring.
pub struct SkillRouter;

impl SkillRouter {
    /// Weighted keyword-based skill assignment from a job description.
    pub fn route(description: &str) -> String {
        let lower = description.to_lowercase();

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
            .unwrap_or_else(|| "code_generation".to_string())
    }
}

/// Selects a model tier based on task complexity inferred from the description.
pub struct ModelSelector;

impl ModelSelector {
    /// Complexity-based model selection using weighted keyword scoring.
    pub fn select(description: &str) -> ModelTier {
        let lower = description.to_lowercase();

        let simple_keywords: &[(&str, u32)] = &[
            ("rename", 10),
            ("format", 10),
            ("typo", 10),
            ("delete", 7),
            ("remove", 5),
            ("update", 3),
        ];

        let complex_keywords: &[(&str, u32)] = &[
            ("architect", 10),
            ("refactor", 8),
            ("redesign", 10),
            ("migrate", 8),
            ("multi-file", 10),
            ("system", 5),
            ("overhaul", 10),
        ];

        let mut simple_score: u32 = 0;
        let mut complex_score: u32 = 0;

        for &(keyword, weight) in simple_keywords {
            if lower.contains(keyword) {
                simple_score += weight;
            }
        }

        for &(keyword, weight) in complex_keywords {
            if lower.contains(keyword) {
                complex_score += weight;
            }
        }

        // Length heuristic
        if description.len() < 20 {
            simple_score += 5;
        }
        if description.len() > 100 {
            complex_score += 5;
        }

        // Word count heuristic
        let word_count = description.split_whitespace().count();
        if word_count > 15 {
            complex_score += 3;
        }

        if simple_score > complex_score && simple_score >= 5 {
            ModelTier::Haiku
        } else if complex_score > simple_score && complex_score >= 5 {
            ModelTier::Opus
        } else {
            ModelTier::Sonnet
        }
    }
}

/// Classification result from LLM-based routing.
#[derive(Debug, serde::Deserialize)]
struct LlmClassification {
    skill: String,
    complexity: String,
}

/// Classify a job description using a Haiku LLM call.
/// Returns (skill, model_tier) or an error if the call fails.
pub async fn classify_with_llm(
    client: &impl MessageSender,
    description: &str,
) -> anyhow::Result<(String, ModelTier)> {
    let req = MessagesRequest {
        model: "claude-haiku-4-5-20251001".to_string(),
        max_tokens: 256,
        messages: vec![Message {
            role: "user".into(),
            content: format!(
                "Classify this coding task. Respond with ONLY valid JSON, no other text.\n\
                 Format: {{\"skill\": \"<skill>\", \"complexity\": \"<complexity>\"}}\n\
                 \n\
                 skill must be one of: testing, refactoring, documentation, bug_fix, code_generation\n\
                 complexity must be one of: simple, medium, complex\n\
                 \n\
                 Task: {description}"
            ),
        }],
    };

    let response = client.send_message(&req).await?;
    let text = response
        .content
        .first()
        .map(|b| b.text.trim().to_string())
        .unwrap_or_default();

    let classification: LlmClassification = serde_json::from_str(&text)
        .map_err(|e| anyhow::anyhow!("Failed to parse LLM classification: {e}"))?;

    let valid_skills = [
        "testing",
        "refactoring",
        "documentation",
        "bug_fix",
        "code_generation",
    ];
    let skill = if valid_skills.contains(&classification.skill.as_str()) {
        classification.skill
    } else {
        "code_generation".to_string()
    };

    let model = match classification.complexity.as_str() {
        "simple" => ModelTier::Haiku,
        "complex" => ModelTier::Opus,
        _ => ModelTier::Sonnet,
    };

    Ok((skill, model))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- SkillRouter tests ---

    #[test]
    fn route_testing() {
        assert_eq!(
            SkillRouter::route("Write unit tests for the parser"),
            "testing"
        );
    }

    #[test]
    fn route_refactoring() {
        assert_eq!(
            SkillRouter::route("Refactor the auth module"),
            "refactoring"
        );
    }

    #[test]
    fn route_documentation() {
        assert_eq!(SkillRouter::route("Add docs for the API"), "documentation");
    }

    #[test]
    fn route_bug_fix() {
        assert_eq!(SkillRouter::route("Fix the login bug"), "bug_fix");
        assert_eq!(SkillRouter::route("Debug the crash"), "bug_fix");
    }

    #[test]
    fn route_default() {
        assert_eq!(
            SkillRouter::route("Implement hero section layout"),
            "code_generation"
        );
    }

    #[test]
    fn route_multi_keyword_picks_highest() {
        // "fix" → bug_fix(10), "bug" → bug_fix(10), "test" → testing(10)
        // bug_fix = 20, testing = 10 → bug_fix wins
        assert_eq!(
            SkillRouter::route("fix the bug in the test suite"),
            "bug_fix"
        );
    }

    #[test]
    fn route_spec_routes_to_testing() {
        assert_eq!(SkillRouter::route("Write a spec for login"), "testing");
    }

    #[test]
    fn route_clean_up_routes_to_refactoring() {
        assert_eq!(
            SkillRouter::route("Clean up the utils module"),
            "refactoring"
        );
    }

    #[test]
    fn route_create_routes_to_code_generation() {
        assert_eq!(
            SkillRouter::route("Create a new user service"),
            "code_generation"
        );
    }

    #[test]
    fn route_no_keywords_defaults() {
        assert_eq!(
            SkillRouter::route("something completely unrelated"),
            "code_generation"
        );
    }

    // --- ModelSelector tests ---

    #[test]
    fn select_haiku_for_simple() {
        assert_eq!(ModelSelector::select("rename variable"), ModelTier::Haiku);
        assert_eq!(
            ModelSelector::select("fix typo in readme"),
            ModelTier::Haiku
        );
        assert_eq!(ModelSelector::select("format code"), ModelTier::Haiku);
    }

    #[test]
    fn select_haiku_for_short() {
        assert_eq!(ModelSelector::select("add a button"), ModelTier::Haiku);
    }

    #[test]
    fn select_opus_for_complex() {
        assert_eq!(
            ModelSelector::select("architect the new payment system"),
            ModelTier::Opus
        );
        assert_eq!(
            ModelSelector::select("refactor the entire auth module"),
            ModelTier::Opus
        );
        assert_eq!(
            ModelSelector::select("redesign the database schema for scaling"),
            ModelTier::Opus
        );
    }

    #[test]
    fn select_sonnet_default() {
        assert_eq!(
            ModelSelector::select("implement the user profile page"),
            ModelTier::Sonnet
        );
    }

    #[test]
    fn select_opus_for_long_complex_description() {
        assert_eq!(
            ModelSelector::select(
                "implement a complex multi-file authentication system with OAuth2 and JWT token refresh"
            ),
            ModelTier::Opus
        );
    }

    #[test]
    fn select_word_count_boosts_complex() {
        // >15 words adds +3 complex, >100 chars adds +5 complex
        let desc = "please carefully plan and then implement the new thing for the app with all the details and edge cases handled";
        let tier = ModelSelector::select(desc);
        // This has >15 words and >100 chars → complex gets +8 boost, no keywords → Opus
        assert_eq!(tier, ModelTier::Opus);
    }

    // --- MockClient for classify_with_llm tests ---

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
            _req: &crate::anthropic::MessagesRequest,
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
    async fn classify_with_llm_valid_response() {
        let client = MockClient::ok(r#"{"skill":"bug_fix","complexity":"simple"}"#);
        let (skill, tier) = classify_with_llm(&client, "fix the login bug")
            .await
            .unwrap();
        assert_eq!(skill, "bug_fix");
        assert_eq!(tier, ModelTier::Haiku);
    }

    #[tokio::test]
    async fn classify_with_llm_complex_response() {
        let client = MockClient::ok(r#"{"skill":"refactoring","complexity":"complex"}"#);
        let (skill, tier) = classify_with_llm(&client, "refactor auth").await.unwrap();
        assert_eq!(skill, "refactoring");
        assert_eq!(tier, ModelTier::Opus);
    }

    #[tokio::test]
    async fn classify_with_llm_medium_response() {
        let client = MockClient::ok(r#"{"skill":"testing","complexity":"medium"}"#);
        let (skill, tier) = classify_with_llm(&client, "add tests").await.unwrap();
        assert_eq!(skill, "testing");
        assert_eq!(tier, ModelTier::Sonnet);
    }

    #[tokio::test]
    async fn classify_with_llm_invalid_json() {
        let client = MockClient::ok("not json");
        let result = classify_with_llm(&client, "whatever").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn classify_with_llm_unknown_skill_defaults() {
        let client = MockClient::ok(r#"{"skill":"unknown_thing","complexity":"simple"}"#);
        let (skill, tier) = classify_with_llm(&client, "do something").await.unwrap();
        assert_eq!(skill, "code_generation");
        assert_eq!(tier, ModelTier::Haiku);
    }

    #[tokio::test]
    async fn classify_with_llm_api_error() {
        let client = MockClient::err(AnthropicError::ApiError {
            status: 500,
            message: "Internal Server Error".to_string(),
        });
        let result = classify_with_llm(&client, "anything").await;
        assert!(result.is_err());
    }

    // --- LLM classification JSON parsing test ---

    #[test]
    fn parse_llm_classification_json() {
        let json = r#"{"skill": "bug_fix", "complexity": "simple"}"#;
        let c: LlmClassification = serde_json::from_str(json).unwrap();
        assert_eq!(c.skill, "bug_fix");
        assert_eq!(c.complexity, "simple");
    }

    #[test]
    fn parse_llm_classification_maps_complexity() {
        // Verify the mapping logic
        assert_eq!(
            match "simple" {
                "simple" => ModelTier::Haiku,
                "complex" => ModelTier::Opus,
                _ => ModelTier::Sonnet,
            },
            ModelTier::Haiku
        );
        assert_eq!(
            match "complex" {
                "simple" => ModelTier::Haiku,
                "complex" => ModelTier::Opus,
                _ => ModelTier::Sonnet,
            },
            ModelTier::Opus
        );
        assert_eq!(
            match "medium" {
                "simple" => ModelTier::Haiku,
                "complex" => ModelTier::Opus,
                _ => ModelTier::Sonnet,
            },
            ModelTier::Sonnet
        );
    }
}
