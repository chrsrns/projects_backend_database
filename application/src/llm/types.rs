use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct GenerateContentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contents: Vec<Content>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<ToolConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GenerateContentResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<Candidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_feedback: Option<PromptFeedback>,
    pub usage_metadata: UsageMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[default]
    User,
    Model,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(untagged, rename_all = "camelCase")]
pub enum Part {
    Text {
        text: String,
    },
    Thought {
        text: String,
        #[serde(default)]
        thought: bool,
    },
    InlineData {
        #[serde(rename = "inlineData")]
        data: InlineData,
    },
    FileData {
        #[serde(rename = "fileData")]
        data: FileData,
    },
    FunctionCall {
        #[serde(rename = "functionCall")]
        call: FunctionCall,
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        response: FunctionResponse,
    },
    ExecutableCode {
        #[serde(rename = "executableCode")]
        code: ExecutableCode,
    },
    CodeExecutionResult {
        #[serde(rename = "codeExecutionResult")]
        result: Value,
    },
    ThoughtSignature {
        #[serde(rename = "thoughtSignature")]
        signature: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InlineData {
    pub mime_type: String,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileData {
    pub mime_type: String,
    pub file_uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCall {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default, rename = "args")]
    pub arguments: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FunctionResponse {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub response: FunctionResponsePayload,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FunctionResponsePayload {
    pub content: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutableCode {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(untagged, rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Tool {
    #[serde(rename = "function_declaration")]
    FunctionDeclaration(ToolConfigFunctionDeclaration),
    DynamicRetrieval {
        google_search_retrieval: DynamicRetrieval,
    },
    GoogleSearch {
        google_search: Value,
    },
    UrlContext {
        url_context: Value,
    },
    CodeExecution {
        code_execution: Value,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolConfig {
    pub function_calling_config: FunctionCallingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCallingConfig {
    pub mode: FunctionCallingMode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_function_names: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FunctionCallingMode {
    #[default]
    ModeUnspecified,
    Auto,
    Any,
    None,
    Validated,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolConfigFunctionDeclaration {
    pub function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Schema>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(no_recursion)]
pub struct Schema {
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<std::collections::HashMap<String, Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SchemaType {
    #[default]
    TypeUnspecified,
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<Schema>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "_responseJsonSchema"
    )]
    pub response_json_schema: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub response_modalities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_enhanced_civic_answers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_config: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_config: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_resolution: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThinkingConfig {
    pub include_thoughts: bool,
    pub thinking_budget: Option<u32>,
    pub thinking_level: Option<ThinkingLevel>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThinkingLevel {
    #[default]
    ThinkingLevelUnspecified,
    Minimal,
    Low,
    Medium,
    High,
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DynamicRetrieval {
    pub dynamic_retrieval_config: DynamicRetrievalConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DynamicRetrievalConfig {
    pub mode: String,
    pub dynamic_threshold: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citation_metadata: Option<CitationMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_metadata: Option<GroundingMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_logprobs: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FinishReason {
    #[default]
    FinishReasonUnspecified,
    Stop,
    MaxTokens,
    Safety,
    Recitation,
    Language,
    Other,
    Blocklist,
    ProhibitedContent,
    Spii,
    MalformedFunctionCall,
    ImageSafety,
    UnexpectedToolCall,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CitationMetadata {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citation_sources: Vec<CitationSource>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CitationSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PromptFeedback {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_reason: Option<BlockReason>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub safety_ratings: Vec<SafetyRating>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockReason {
    #[default]
    BlockReasonUnspecified,
    Safety,
    Other,
    Blocklist,
    ProhibitedContent,
    ImageSafety,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_prompt_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thoughts_token_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prompt_tokens_details: Vec<ModalityTokenCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cache_tokens_details: Vec<ModalityTokenCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates_tokens_details: Vec<ModalityTokenCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_use_prompt_tokens_details: Vec<ModalityTokenCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traffic_type: Option<TrafficType>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModalityTokenCount {
    pub modality: Modality,
    pub token_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Modality {
    #[default]
    ModalityUnspecified,
    Text,
    Image,
    Video,
    Audio,
    Document,
    Thoughts,
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrafficType {
    #[default]
    TrafficTypeUnspecified,
    OnDemand,
    ProvisionedThroughput,
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SafetyRating {
    pub category: HarmCategory,
    pub probability: HarmProbability,
    pub blocked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmProbability {
    #[default]
    HarmProbabilityUnspecified,
    Negligible,
    Low,
    Medium,
    High,
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmCategory {
    #[default]
    HarmCategoryUnspecified,
    Derogatory,
    Toxicity,
    Violence,
    Sexual,
    Medical,
    Dangerous,
    Harassment,
    HateSpeech,
    SexuallyExplicit,
    DangerousContent,
    CivicIntegrity,
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroundingMetadata {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grounding_chunks: Vec<GroundingChunk>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grounding_supports: Vec<GroundingSupport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub web_search_queries: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_entry_point: Option<SearchEntryPoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_metadata: Option<RetrievalMetadata>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum GroundingChunk {
    Web(Web),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Web {
    pub uri: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroundingSupport {
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub grounding_chunk_indices: std::collections::BTreeSet<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub confidence_scores: Vec<f64>,
    pub segment: Segment,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchEntryPoint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendered_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdk_blob: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search_dynamic_retrieval_score: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_simple_generate_request() {
        let json = serde_json::json!({
            "contents": [{"parts": [{"text": "hello"}]}]
        });
        let request: GenerateContentRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.contents.len(), 1);
        assert_eq!(request.contents[0].parts.len(), 1);
    }

    #[test]
    fn reject_unknown_model_field() {
        let json = serde_json::json!({
            "model": "gemini-1.5-flash",
            "contents": [{"parts": [{"text": "hello"}]}]
        });
        assert!(serde_json::from_value::<GenerateContentRequest>(json).is_err());
    }
}
