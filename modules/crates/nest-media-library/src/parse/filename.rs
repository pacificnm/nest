//! Filename title and year heuristics.

use std::path::Path;

/// Parsed title/year guess from a media path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilenameGuess {
    /// Guessed title.
    pub title: Option<String>,
    /// Guessed release year.
    pub year: Option<u16>,
}

/// Guesses movie title and year from a relative media path.
pub fn guess_from_path(path: &str) -> FilenameGuess {
    let path = path.replace('\\', "/");
    let file_name = Path::new(&path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&path);
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or(file_name);

    let year = find_year_in_parens(&path).or_else(|| find_year_dotted(stem));
    let title = extract_title(stem, year);
    FilenameGuess { title, year }
}

fn find_year_in_parens(text: &str) -> Option<u16> {
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if ch != '(' {
            continue;
        }
        let rest = &text[index + 1..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.len() == 4 {
            if let Ok(year) = digits.parse::<u16>() {
                if (1900..=2100).contains(&year) {
                    return Some(year);
                }
            }
        }
    }
    None
}

fn find_year_dotted(stem: &str) -> Option<u16> {
    for part in stem.split(['.', ' ', '_', '-']) {
        if part.len() == 4 {
            if let Ok(year) = part.parse::<u16>() {
                if (1900..=2100).contains(&year) {
                    return Some(year);
                }
            }
        }
    }
    None
}

fn extract_title(stem: &str, year: Option<u16>) -> Option<String> {
    let mut title = stem.to_string();

    if let Some(year) = year {
        title = title.replace(&format!("({year})"), " ");
        title = title.replace(&year.to_string(), " ");
    }

    title = title.replace('.', " ");
    title = title.replace('_', " ");
    title = collapse_whitespace(&title);

    let lowered = title.to_ascii_lowercase();
    for token in ["1080p", "720p", "2160p", "4k", "bluray", "webrip", "x264", "x265", "h264", "h265"]
    {
        title = replace_token_case_insensitive(&title, token, "");
    }

    title = collapse_whitespace(&title);
    if title.is_empty() {
        None
    } else {
        Some(title)
    }
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn replace_token_case_insensitive(text: &str, token: &str, replacement: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let mut result = String::new();
    let mut index = 0;
    while let Some(found) = lower[index..].find(token) {
        let start = index + found;
        let end = start + token.len();
        result.push_str(&text[index..start]);
        result.push_str(replacement);
        index = end;
    }
    result.push_str(&text[index..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_folder_and_filename_with_year() {
        let guess = guess_from_path("Movies/Alien (1979)/Alien (1979).mkv");
        assert_eq!(guess.title.as_deref(), Some("Alien"));
        assert_eq!(guess.year, Some(1979));
    }

    #[test]
    fn parses_dotted_filename_with_year() {
        let guess = guess_from_path("Alien.1979.1080p.mkv");
        assert_eq!(guess.title.as_deref(), Some("Alien"));
        assert_eq!(guess.year, Some(1979));
    }

    #[test]
    fn unparseable_still_returns_none_title_when_empty() {
        let guess = guess_from_path("1080p.mkv");
        assert!(guess.title.is_none());
        assert!(guess.year.is_none());
    }
}
