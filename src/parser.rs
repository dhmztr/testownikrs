use crate::types::*;
use anyhow::{Context, Result};
use encoding_rs::WINDOWS_1250;
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// JSON Quiz file format structures
#[derive(Debug, Deserialize)]
pub struct JsonQuizFile {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<i32>,
    pub questions: Vec<JsonQuestion>,
}

#[derive(Debug, Deserialize)]
pub struct JsonQuestion {
    pub id: String,
    #[serde(default)]
    pub order: Option<i32>,
    pub text: String,
    #[serde(default)]
    pub multiple: bool,
    pub answers: Vec<JsonAnswer>,
}

#[derive(Debug, Deserialize)]
pub struct JsonAnswer {
    pub id: String,
    #[serde(default)]
    pub order: Option<i32>,
    pub text: String,
    pub is_correct: bool,
}

/// Reads a quiz file and parses it into a vector of questions
/// Supports both text format (.txt) and JSON format (.json)
pub fn read_questions_from_file(file_path: &str) -> Result<Vec<Question>> {
    let path = Path::new(file_path);
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    
    match extension.to_lowercase().as_str() {
        "json" => read_questions_from_json(file_path),
        _ => read_questions_from_text(file_path),
    }
}

/// Reads questions from a JSON quiz file
pub fn read_questions_from_json(file_path: &str) -> Result<Vec<Question>> {
    let content = fs::read_to_string(file_path)
        .context("Could not read JSON file")?;
    
    let quiz: JsonQuizFile = serde_json::from_str(&content)
        .context("Could not parse JSON quiz file")?;
    
    let mut questions = Vec::new();
    
    for json_q in quiz.questions.iter() {
        let tag = json_q.id.clone();
        
        // Convert JSON answers to our Answer format
        let answers: Vec<Answer> = json_q.answers.iter().enumerate().map(|(idx, json_a)| {
            Answer {
                id: idx,
                answer_type: AnswerType::Text,
                content: AnswerContent::Single { 
                    content: json_a.text.clone() 
                },
                is_correct: Some(json_a.is_correct),
                correct_option_id: None,
                options: None,
            }
        }).collect();
        
        let question = Question {
            tag,
            content_type: ContentType::Text,
            content: QuestionContent::Text(json_q.text.clone()),
            // Both single and multiple choice questions use QuestionType::Single in this codebase
            // The difference is in how many answers have is_correct = true
            question_type: QuestionType::Single,
            answers,
        };
        
        questions.push(question);
    }
    
    Ok(questions)
}

/// Returns the title from a JSON quiz file, if available
pub fn get_json_quiz_title(file_path: &str) -> Option<String> {
    let content = fs::read_to_string(file_path).ok()?;
    let quiz: JsonQuizFile = serde_json::from_str(&content).ok()?;
    Some(quiz.title)
}

/// Czyta plik z pytaniami w formacie tekstowym i parsuje go na wektor pytań
pub fn read_questions_from_text(file_path: &str) -> Result<Vec<Question>> {
    let bytes = fs::read(file_path).context("Nie można odczytać pliku")?;
    let text = detect_and_decode(&bytes)?;

    // Normalize line endings to Unix format
    let text = text.replace("\r\n", "\n");

    // Split by ## at start of line (with possible leading newline)
    let mut blocks = Vec::new();
    let mut current_block = String::new();

    for line in text.lines() {
        if line.trim() == "##" {
            if !current_block.trim().is_empty() {
                blocks.push(current_block.clone());
            }
            current_block.clear();
        } else {
            if !current_block.is_empty() {
                current_block.push('\n');
            }
            current_block.push_str(line);
        }
    }

    // Don't forget the last block
    if !current_block.trim().is_empty() {
        blocks.push(current_block);
    }

    let mut questions = Vec::new();
    for (index, block) in blocks.iter().enumerate() {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let tag = format!("question_{:03}", index + 1);
        match parse_question_block(block, tag) {
            Ok(question) => questions.push(question),
            Err(e) => eprintln!(
                "Ostrzeżenie: Nie można sparsować pytania {}: {}",
                index + 1,
                e
            ),
        }
    }

    Ok(questions)
}

/// Wykrywa kodowanie i dekoduje bajty na String
pub fn detect_and_decode(bytes: &[u8]) -> Result<String> {
    if is_utf8(bytes) {
        String::from_utf8(bytes.to_vec()).context("Błąd dekodowania UTF-8")
    } else {
        let (decoded, _, had_errors) = WINDOWS_1250.decode(bytes);
        if had_errors {
            eprintln!("Ostrzeżenie: Wykryto błędy podczas dekodowania Windows-1250");
        }
        Ok(decoded.into_owned())
    }
}

/// Sprawdza czy dane są w kodowaniu UTF-8
fn is_utf8(data: &[u8]) -> bool {
    let mut i = 0;
    let len = data.len();

    while i < len {
        let b = data[i];

        if b == 0x09 || b == 0x0A || b == 0x0D || (0x20..=0x7E).contains(&b) {
            i += 1;
            continue;
        }

        if (0xC2..=0xDF).contains(&b) {
            if i + 1 >= len || data[i + 1] < 0x80 || data[i + 1] > 0xBF {
                return false;
            }
            i += 2;
        } else if b == 0xE0 {
            if i + 2 >= len
                || data[i + 1] < 0xA0
                || data[i + 1] > 0xBF
                || data[i + 2] < 0x80
                || data[i + 2] > 0xBF
            {
                return false;
            }
            i += 3;
        } else if (0xE1..=0xEC).contains(&b) || b == 0xEE || b == 0xEF {
            if i + 2 >= len
                || data[i + 1] < 0x80
                || data[i + 1] > 0xBF
                || data[i + 2] < 0x80
                || data[i + 2] > 0xBF
            {
                return false;
            }
            i += 3;
        } else if b == 0xED {
            if i + 2 >= len
                || data[i + 1] < 0x80
                || data[i + 1] > 0x9F
                || data[i + 2] < 0x80
                || data[i + 2] > 0xBF
            {
                return false;
            }
            i += 3;
        } else if b == 0xF0 {
            if i + 3 >= len
                || data[i + 1] < 0x90
                || data[i + 1] > 0xBF
                || data[i + 2] < 0x80
                || data[i + 2] > 0xBF
                || data[i + 3] < 0x80
                || data[i + 3] > 0xBF
            {
                return false;
            }
            i += 4;
        } else if (0xF1..=0xF3).contains(&b) {
            if i + 3 >= len
                || data[i + 1] < 0x80
                || data[i + 1] > 0xBF
                || data[i + 2] < 0x80
                || data[i + 2] > 0xBF
                || data[i + 3] < 0x80
                || data[i + 3] > 0xBF
            {
                return false;
            }
            i += 4;
        } else if b == 0xF4 {
            if i + 3 >= len
                || data[i + 1] < 0x80
                || data[i + 1] > 0x8F
                || data[i + 2] < 0x80
                || data[i + 2] > 0xBF
                || data[i + 3] < 0x80
                || data[i + 3] > 0xBF
            {
                return false;
            }
            i += 4;
        } else {
            return false;
        }
    }

    true
}

/// Parsuje pojedynczy blok tekstu jako pytanie
pub fn parse_question_block(text: &str, tag: String) -> Result<Question> {
    let lines: Vec<&str> = text.lines().collect();

    if lines.is_empty() {
        anyhow::bail!("Pusty blok pytania");
    }

    let first_line = lines[0].trim();

    if first_line.starts_with('X') {
        parse_x_question(&lines, tag)
    } else if first_line.starts_with('Y') {
        parse_y_question(&lines, tag)
    } else {
        anyhow::bail!("Nieznany typ pytania: {}", first_line);
    }
}

/// Parsuje pytanie typu X (single/multiple choice)
/// Parsuje pytanie typu X (single/multiple choice)
pub fn parse_x_question(lines: &[&str], tag: String) -> Result<Question> {
    if lines.len() < 3 {
        anyhow::bail!("Za mało linii dla pytania typu X");
    }

    let first_line = lines[0].trim();
    let correct_answers_str = &first_line[1..];

    let correct_indices: Vec<usize> = correct_answers_str
        .chars()
        .enumerate()
        .filter(|(_, c)| *c == '1')
        .map(|(i, _)| i)
        .collect();

    // 1. Zaczynamy od bazowej treści pytania
    let mut question_text = lines[1].trim().to_string();

    // 2. Pobieramy wszystkie linie poniżej pytania
    let raw_answers: Vec<&str> = lines[2..]
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();

    // 3. Oddzielamy obrazki pytania od rzeczywistych odpowiedzi
    let mut final_answers = Vec::new();
    let all_answers_are_images = raw_answers.iter().all(|a| a.starts_with("[img]"));

    if !all_answers_are_images {
        for line in raw_answers {
            // Jeśli linia to czysty tag [img], doklejamy go do treści pytania
            if line.starts_with("[img]") && line.ends_with("[/img]") {
                question_text.push('\n');
                question_text.push_str(line);
            } else {
                final_answers.push(line);
            }
        }
    } else {
        // Jeśli wszystkie odpowiedzi to obrazki, nie modyfikujemy ich
        final_answers = raw_answers;
    }

    // 4. Określamy ostateczny typ i zawartość pytania
    let (content_type, content) = if question_text.starts_with("[img]") && !question_text.contains('\n') {
        (
            ContentType::Image,
            QuestionContent::Image(extract_image_link(&question_text)?),
        )
    } else {
        (
            ContentType::Text,
            QuestionContent::Text(question_text),
        )
    };

    // 5. Parsujemy właściwe odpowiedzi
    let answers: Vec<Answer> = final_answers
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            let (answer_type, content) = if line.starts_with("[img]") {
                (
                    AnswerType::Image,
                    extract_image_link(line).unwrap_or_default(),
                )
            } else {
                (AnswerType::Text, line.to_string())
            };

            Answer {
                id: index,
                answer_type,
                content: AnswerContent::Single { content },
                // Sprawdzamy is_correct bazując na nowym (przefiltrowanym) indeksie
                is_correct: Some(correct_indices.contains(&index)),
                correct_option_id: None,
                options: None,
            }
        })
        .collect();

    Ok(Question {
        tag,
        content_type,
        content,
        question_type: QuestionType::Single,
        answers,
    })
}

/// Parsuje pytanie typu Y (select/dropdown)
pub fn parse_y_question(lines: &[&str], tag: String) -> Result<Question> {
    if lines.len() < 3 {
        anyhow::bail!("Za mało linii dla pytania typu Y");
    }

    let first_line = lines[0].trim();
    if first_line.len() < 2 {
        anyhow::bail!("Nieprawidłowa pierwsza linia pytania Y");
    }

    let correct_answers_str = &first_line[1..];
    let correct_option_ids: Vec<usize> = correct_answers_str
        .chars()
        .filter_map(|c| c.to_digit(10))
        .map(|d| (d as usize).saturating_sub(1))
        .collect();

    let question_line = lines[1].trim();
    let content_type = if question_line.starts_with("[img]") {
        ContentType::Image
    } else {
        ContentType::Text
    };

    let content = QuestionContent::Select(parse_y_question_content(question_line));

    let answers: Vec<Answer> = lines[2..]
        .iter()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
        .map(|(select_index, line)| {
            let line = line.trim();
            let options_str: Vec<&str> = line.split(";;").filter(|s| !s.is_empty()).collect();

            let correct_option_id = correct_option_ids.get(select_index).copied().unwrap_or(0);

            let options: Vec<AnswerOption> = options_str
                .iter()
                .enumerate()
                .map(|(option_index, option_str)| {
                    let (option_type, option_content) = if option_str.starts_with("[img]") {
                        (
                            AnswerType::Image,
                            extract_image_link(option_str).unwrap_or_default(),
                        )
                    } else {
                        (AnswerType::Text, option_str.to_string())
                    };

                    AnswerOption {
                        id: option_index,
                        option_type,
                        content: option_content,
                        is_correct: option_index == correct_option_id,
                    }
                })
                .collect();

            Answer {
                id: select_index,
                answer_type: AnswerType::Text,
                content: AnswerContent::Single {
                    content: String::new(),
                },
                is_correct: None,
                correct_option_id: Some(correct_option_id),
                options: Some(options),
            }
        })
        .collect();

    Ok(Question {
        tag,
        content_type,
        content,
        question_type: QuestionType::Select,
        answers,
    })
}

/// Ekstrahuje link do obrazka z tagu [img]link[/img]
pub fn extract_image_link(line: &str) -> Result<String> {
    let re = Regex::new(r"\[img\](.*?)\[/img\]")?;
    if let Some(caps) = re.captures(line) {
        Ok(caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string())
    } else {
        anyhow::bail!("Nie znaleziono tagu [img]");
    }
}

/// Parsuje treść pytania Y z placeholderami {wybór N}
pub fn parse_y_question_content(line: &str) -> Vec<ContentPart> {
    let re = Regex::new(r"(\{wybór [1-9][0-9]*\})").unwrap();
    let mut result = Vec::new();
    let mut last_end = 0;

    for cap in re.captures_iter(line) {
        let mat = cap.get(0).unwrap();

        // Dodaj tekst przed placeholderem
        if mat.start() > last_end {
            let text = &line[last_end..mat.start()];
            if !text.is_empty() {
                result.push(ContentPart::Text(text.to_string()));
            }
        }

        // Parsuj numer selecta
        let select_str = mat.as_str();
        if let Some(num_str) = select_str
            .strip_prefix("{wybór ")
            .and_then(|s| s.strip_suffix("}"))
        {
            if let Ok(num) = num_str.parse::<usize>() {
                result.push(ContentPart::SelectPlaceholder {
                    select_id: num - 1,
                    visible_content: String::new(),
                });
            }
        }

        last_end = mat.end();
    }

    // Dodaj pozostały tekst
    if last_end < line.len() {
        let text = &line[last_end..];
        if !text.is_empty() {
            result.push(ContentPart::Text(text.to_string()));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_image_link() {
        let line = "[img]obrazek.png[/img]";
        let result = extract_image_link(line).unwrap();
        assert_eq!(result, "obrazek.png");
    }

    #[test]
    fn test_extract_image_link_with_text() {
        let line = "Jakiś tekst [img]path/to/image.jpg[/img] więcej tekstu";
        let result = extract_image_link(line).unwrap();
        assert_eq!(result, "path/to/image.jpg");
    }

    #[test]
    fn test_parse_y_question_content() {
        let line = "Wybierz {wybór 1} oraz {wybór 2} z listy";
        let result = parse_y_question_content(line);

        assert_eq!(result.len(), 5);
        match &result[0] {
            ContentPart::Text(s) => assert_eq!(s, "Wybierz "),
            _ => panic!("Expected Text"),
        }
        match &result[1] {
            ContentPart::SelectPlaceholder { select_id, .. } => assert_eq!(*select_id, 0),
            _ => panic!("Expected SelectPlaceholder"),
        }
    }

    #[test]
    fn test_is_utf8_valid() {
        let text = "Hello, world!";
        assert!(is_utf8(text.as_bytes()));
    }

    #[test]
    fn test_is_utf8_polish() {
        let text = "Cześć, świat!";
        assert!(is_utf8(text.as_bytes()));
    }

    #[test]
    fn test_split_questions_by_separator() {
        let test_content = "X1\nPytanie 1?\nOdp A\n*Odp B\n##\nX2\nPytanie 2?\n*Odp C\nOdp D";
        
        // Test parsing multiple questions
        let temp_file = std::env::temp_dir().join("test_questions_temp.txt");
        std::fs::write(&temp_file, test_content).unwrap();
        
        let questions = read_questions_from_file(temp_file.to_str().unwrap()).unwrap();
        std::fs::remove_file(&temp_file).unwrap();
        
        assert_eq!(questions.len(), 2, "Should parse 2 questions");
    }

    #[test]
    fn test_split_questions_windows_line_endings() {
        // Test with Windows line endings \r\n
        let test_content = "X1\r\nPytanie 1?\r\nOdp A\r\n*Odp B\r\n##\r\nX2\r\nPytanie 2?\r\n*Odp C\r\nOdp D";
        
        let temp_file = std::env::temp_dir().join("test_questions_windows.txt");
        std::fs::write(&temp_file, test_content).unwrap();
        
        let questions = read_questions_from_file(temp_file.to_str().unwrap()).unwrap();
        std::fs::remove_file(&temp_file).unwrap();
        
        assert_eq!(questions.len(), 2, "Should parse 2 questions with Windows line endings");
    }

    #[test]
    fn test_split_questions_with_spaces() {
        // Test with spaces around ##
        let test_content = "X1\nPytanie 1?\nOdp A\n*Odp B\n  ##  \nX2\nPytanie 2?\n*Odp C\nOdp D";
        
        let temp_file = std::env::temp_dir().join("test_questions_spaces.txt");
        std::fs::write(&temp_file, test_content).unwrap();
        
        let questions = read_questions_from_file(temp_file.to_str().unwrap()).unwrap();
        std::fs::remove_file(&temp_file).unwrap();
        
        assert_eq!(questions.len(), 2, "Should parse 2 questions with spaces around ##");
    }

    #[test]
    fn test_split_questions_separator_at_start() {
        // Test with ## at start of file
        let test_content = "##\nX1\nPytanie 1?\nOdp A\n*Odp B\n##\nX2\nPytanie 2?\n*Odp C\nOdp D";
        
        let temp_file = std::env::temp_dir().join("test_questions_start.txt");
        std::fs::write(&temp_file, test_content).unwrap();
        
        let questions = read_questions_from_file(temp_file.to_str().unwrap()).unwrap();
        std::fs::remove_file(&temp_file).unwrap();
        
        assert_eq!(questions.len(), 2, "Should parse 2 questions when ## is at start");
    }

    #[test]
    fn test_split_questions_empty_blocks() {
        // Test with multiple separators creating empty blocks
        let test_content = "X1\nPytanie 1?\nOdp A\n*Odp B\n##\n##\nX2\nPytanie 2?\n*Odp C\nOdp D\n##\n##";
        
        let temp_file = std::env::temp_dir().join("test_questions_empty.txt");
        std::fs::write(&temp_file, test_content).unwrap();
        
        let questions = read_questions_from_file(temp_file.to_str().unwrap()).unwrap();
        std::fs::remove_file(&temp_file).unwrap();
        
        assert_eq!(questions.len(), 2, "Should skip empty blocks and parse only 2 questions");
    }

    #[test]
    fn test_json_quiz_parsing() {
        let json_content = r#"{
            "title": "Test Quiz",
            "description": "Test description",
            "version": 1,
            "questions": [
                {
                    "id": "q1",
                    "order": 1,
                    "text": "What is 2+2?",
                    "multiple": false,
                    "answers": [
                        {"id": "a1", "order": 1, "text": "3", "is_correct": false},
                        {"id": "a2", "order": 2, "text": "4", "is_correct": true},
                        {"id": "a3", "order": 3, "text": "5", "is_correct": false}
                    ]
                },
                {
                    "id": "q2",
                    "order": 2,
                    "text": "What is the capital of Poland?",
                    "multiple": true,
                    "answers": [
                        {"id": "a4", "order": 1, "text": "Warsaw", "is_correct": true},
                        {"id": "a5", "order": 2, "text": "Berlin", "is_correct": false}
                    ]
                }
            ]
        }"#;
        
        let temp_file = std::env::temp_dir().join("test_quiz.json");
        std::fs::write(&temp_file, json_content).unwrap();
        
        let questions = read_questions_from_file(temp_file.to_str().unwrap()).unwrap();
        std::fs::remove_file(&temp_file).unwrap();
        
        assert_eq!(questions.len(), 2, "Should parse 2 questions from JSON");
        assert_eq!(questions[0].tag, "q1");
        assert_eq!(questions[0].answers.len(), 3);
        assert_eq!(questions[1].tag, "q2");
        assert_eq!(questions[1].answers.len(), 2);
    }

    #[test]
    fn test_json_quiz_title() {
        let json_content = r#"{
            "title": "My Test Quiz",
            "questions": []
        }"#;
        
        let temp_file = std::env::temp_dir().join("test_quiz_title.json");
        std::fs::write(&temp_file, json_content).unwrap();
        
        let title = get_json_quiz_title(temp_file.to_str().unwrap());
        std::fs::remove_file(&temp_file).unwrap();
        
        assert_eq!(title, Some("My Test Quiz".to_string()));
    }
}
