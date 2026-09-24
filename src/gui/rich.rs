// Rendering of question/answer content that may contain `[img]...[/img]` tags.

use crate::gui::styles::{image_frame_style, missing_image_style, Colors};
use crate::gui::Message;
use iced::widget::{container, image as img_widget, row, text, Column};
use iced::{Alignment, Color, ContentFit, Element, Font, Length};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const EMOJI_FONT: Font = Font::with_name("Segoe UI Emoji");

/// How deep subdirectories of the quiz folder are searched for images
const MAX_SEARCH_DEPTH: usize = 3;
/// Safety cap on the number of directories visited while searching
const MAX_SEARCHED_DIRS: usize = 500;

/// Size limits for a rendered image
#[derive(Debug, Clone, Copy)]
pub struct ImageSize {
    pub max_width: f32,
    pub max_height: f32,
}

impl ImageSize {
    pub const QUESTION: ImageSize = ImageSize { max_width: f32::INFINITY, max_height: 300.0 };
    pub const ANSWER: ImageSize = ImageSize { max_width: 420.0, max_height: 200.0 };
}

/// Renders multi-line text in which `[img]` tags may appear anywhere
/// (own line or inline). Text and images are emitted in source order.
pub fn rich_content<'a>(
    base_dir: &Path,
    content: &str,
    text_size: u16,
    text_color: Color,
    image_size: ImageSize,
) -> Column<'a, Message> {
    let mut col = Column::new().spacing(10);
    for raw_line in content.split('\n') {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        col = push_line(base_dir, col, line, text_size, text_color, image_size);
    }
    col
}

fn push_line<'a>(
    base_dir: &Path,
    mut col: Column<'a, Message>,
    line: &str,
    text_size: u16,
    text_color: Color,
    image_size: ImageSize,
) -> Column<'a, Message> {
    let mut rest = line;
    loop {
        let Some(open) = rest.find("[img]") else {
            let tail = rest.trim();
            if !tail.is_empty() {
                col = col.push(text(tail.to_string()).size(text_size).style(text_color));
            }
            return col;
        };

        let pre = rest[..open].trim();
        if !pre.is_empty() {
            col = col.push(text(pre.to_string()).size(text_size).style(text_color));
        }

        let after_open = &rest[open + "[img]".len()..];
        let Some(close) = after_open.find("[/img]") else {
            // Unclosed tag: show the remainder as plain text
            let tail = rest[open..].trim();
            if !tail.is_empty() {
                col = col.push(text(tail.to_string()).size(text_size).style(text_color));
            }
            return col;
        };

        let img_path = after_open[..close].trim();
        if !img_path.is_empty() {
            col = col.push(image_view(base_dir, img_path, image_size));
        }
        rest = &after_open[close + "[/img]".len()..];
    }
}

/// Renders a single image keeping its aspect ratio. Small images are never
/// upscaled (no blur), large ones are scaled down to fit `size`.
/// A visible placeholder is shown when the file cannot be found.
pub fn image_view<'a>(base_dir: &Path, img_path: &str, size: ImageSize) -> Element<'a, Message> {
    match resolve_image_path(base_dir, img_path) {
        Some(path) => {
            let image = img_widget(path)
                .width(Length::Shrink)
                .height(Length::Shrink)
                .content_fit(ContentFit::ScaleDown);

            let mut frame = container(image)
                .padding(6)
                .max_height(size.max_height + 12.0)
                .style(image_frame_style());
            if size.max_width.is_finite() {
                frame = frame.max_width(size.max_width + 12.0);
            }
            frame.into()
        }
        None => container(
            row![
                text("🖼").font(EMOJI_FONT).size(16),
                text(format!("Nie znaleziono obrazka: {}", img_path))
                    .size(13)
                    .style(Colors::WARNING),
            ]
            .spacing(8)
            .align_items(Alignment::Center),
        )
        .padding([8, 12])
        .style(missing_image_style())
        .into(),
    }
}

/// Resolves an image reference from a quiz file to an existing file.
///
/// Handles: paths relative to the quiz folder, absolute paths, Windows
/// separators, `./` prefixes, images placed in (nested) subfolders and
/// file names whose letter case differs from the reference (common for
/// quizzes created on Windows). Results are cached.
pub fn resolve_image_path(base_dir: &Path, reference: &str) -> Option<PathBuf> {
    static CACHE: OnceLock<Mutex<HashMap<(PathBuf, String), Option<PathBuf>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = (base_dir.to_path_buf(), reference.to_string());

    if let Ok(guard) = cache.lock() {
        if let Some(hit) = guard.get(&key) {
            // A cached path that disappeared in the meantime is looked up again
            match hit {
                Some(p) if !p.is_file() => {}
                _ => return hit.clone(),
            }
        }
    }

    let resolved = find_image(base_dir, reference);
    if let Ok(mut guard) = cache.lock() {
        guard.insert(key, resolved.clone());
    }
    resolved
}

fn find_image(base_dir: &Path, reference: &str) -> Option<PathBuf> {
    let normalized = reference
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .replace('\\', "/");
    let normalized = normalized.trim_start_matches("./");
    if normalized.is_empty() {
        return None;
    }

    let rel = PathBuf::from(normalized);
    if rel.is_absolute() {
        if rel.is_file() {
            return Some(rel);
        }
    }

    let base = if base_dir.as_os_str().is_empty() { Path::new(".") } else { base_dir };

    // 1. Exact path relative to the quiz folder
    let direct = base.join(&rel);
    if direct.is_file() {
        return Some(direct);
    }

    let file_name = rel.file_name()?.to_string_lossy().to_lowercase();

    // 2. Breadth-first search of the quiz folder and its subfolders:
    //    exact relative path first, then a case-insensitive file name match
    let mut queue: Vec<(PathBuf, usize)> = vec![(base.to_path_buf(), 0)];
    let mut visited = 0;
    while !queue.is_empty() && visited < MAX_SEARCHED_DIRS {
        let (dir, depth) = queue.remove(0);
        visited += 1;

        let candidate = dir.join(&rel);
        if candidate.is_file() {
            return Some(candidate);
        }

        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        let mut subdirs = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if is_dir {
                if depth < MAX_SEARCH_DEPTH {
                    subdirs.push(path);
                }
            } else if entry.file_name().to_string_lossy().to_lowercase() == file_name {
                return Some(path);
            }
        }
        subdirs.sort();
        queue.extend(subdirs.into_iter().map(|d| (d, depth + 1)));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_nested_and_case_insensitive() {
        let root = std::env::temp_dir().join(format!("testownik_img_{}", std::process::id()));
        let nested = root.join("obrazki").join("rozdzial1");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(root.join("A.PNG"), b"x").unwrap();
        std::fs::write(nested.join("z1.png"), b"x").unwrap();

        assert_eq!(find_image(&root, "A.PNG"), Some(root.join("A.PNG")));
        assert_eq!(find_image(&root, "a.png"), Some(root.join("A.PNG")));
        assert_eq!(find_image(&root, "./a.png"), Some(root.join("A.PNG")));
        assert_eq!(find_image(&root, "Z1.png"), Some(nested.join("z1.png")));
        assert_eq!(find_image(&root, "obrazki\\rozdzial1\\z1.png"), Some(nested.join("z1.png")));
        assert_eq!(find_image(&root, "brak.png"), None);

        std::fs::remove_dir_all(&root).unwrap();
    }
}
