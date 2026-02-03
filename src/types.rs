use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub tag: String,
    #[serde(rename = "contentType")]
    pub content_type: ContentType,
    pub content: QuestionContent,
    #[serde(rename = "type")]
    pub question_type: QuestionType,
    pub answers: Vec<Answer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Text,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QuestionContent {
    Text(String),
    Image(String),
    Select(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentPart {
    Text(String),
    SelectPlaceholder {
        #[serde(rename = "selectId")]
        select_id: usize,
        #[serde(rename = "visibleContent")]
        visible_content: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuestionType {
    Single,
    Select,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    pub id: usize,
    #[serde(rename = "type")]
    pub answer_type: AnswerType,
    pub content: AnswerContent,
    #[serde(rename = "isCorrect", skip_serializing_if = "Option::is_none")]
    pub is_correct: Option<bool>,
    #[serde(rename = "correctOptionId", skip_serializing_if = "Option::is_none")]
    pub correct_option_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<AnswerOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnswerType {
    Text,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnswerContent {
    Single {
        content: String,
    },
    Select {
        #[serde(rename = "correctOptionId")]
        correct_option_id: usize,
        options: Vec<AnswerOption>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerOption {
    pub id: usize,
    #[serde(rename = "type")]
    pub option_type: AnswerType,
    pub content: String,
    #[serde(rename = "isCorrect")]
    pub is_correct: bool,
}
