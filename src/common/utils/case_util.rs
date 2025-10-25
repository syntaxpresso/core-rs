#![allow(dead_code)]

use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutySnakeCase, ToSnakeCase, ToTitleCase,
    ToTrainCase,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaseType {
    Snake,
    Camel,
    Pascal,
    Kebab,
    Title,
    Train,
    ScreamingSnake,
    Unknown,
}

pub fn to_snake_case(s: &str) -> String {
    s.to_snake_case()
}

pub fn to_camel_case(s: &str) -> String {
    s.to_lower_camel_case()
}

pub fn to_pascal_case(s: &str) -> String {
    s.to_pascal_case()
}

pub fn to_kebab_case(s: &str) -> String {
    s.to_kebab_case()
}

pub fn to_title_case(s: &str) -> String {
    s.to_title_case()
}

pub fn to_train_case(s: &str) -> String {
    s.to_train_case()
}

pub fn to_screaming_snake_case(s: &str) -> String {
    s.to_shouty_snake_case()
}

pub fn detect_case(s: &str) -> CaseType {
    if s.trim().is_empty() {
        return CaseType::Unknown;
    }

    let has_underscores = s.contains('_');
    let has_hyphens = s.contains('-');
    let has_spaces = s.contains(' ');
    let has_uppercase = s.chars().any(|c| c.is_uppercase());
    let has_lowercase = s.chars().any(|c| c.is_lowercase());
    let starts_with_uppercase = s.chars().next().is_some_and(|c| c.is_uppercase());
    let is_all_uppercase = s
        .chars()
        .filter(|c| c.is_alphabetic())
        .all(|c| c.is_uppercase());
    // Check for screaming snake case first (ALL_UPPERCASE_WITH_UNDERSCORES)
    if has_underscores && is_all_uppercase && !has_hyphens && !has_spaces {
        return CaseType::ScreamingSnake;
    }
    // Check for snake_case
    if has_underscores && !has_uppercase && !has_hyphens && !has_spaces {
        return CaseType::Snake;
    }
    // Check for kebab-case
    if has_hyphens && !has_uppercase && !has_underscores && !has_spaces {
        return CaseType::Kebab;
    }
    // Check for Train-Case
    if has_hyphens && has_uppercase && starts_with_uppercase && !has_underscores && !has_spaces {
        return CaseType::Train;
    }
    // Check for Title Case (spaces with each word capitalized)
    if has_spaces && has_uppercase && starts_with_uppercase && !has_underscores && !has_hyphens {
        let words: Vec<&str> = s.split_whitespace().collect();
        let all_words_capitalized = words
            .iter()
            .all(|word| word.chars().next().is_some_and(|c| c.is_uppercase()));
        if all_words_capitalized {
            return CaseType::Title;
        }
    }
    // Check for PascalCase (starts with uppercase, no separators)
    if starts_with_uppercase
        && has_uppercase
        && has_lowercase
        && !has_underscores
        && !has_hyphens
        && !has_spaces
    {
        return CaseType::Pascal;
    }
    // Check for camelCase (starts with lowercase, has uppercase, no separators)
    if !starts_with_uppercase
        && has_uppercase
        && has_lowercase
        && !has_underscores
        && !has_hyphens
        && !has_spaces
    {
        return CaseType::Camel;
    }
    CaseType::Unknown
}

pub fn convert_case(s: &str, from_case: CaseType, to_case: CaseType) -> String {
    if from_case == to_case {
        return s.to_string();
    }
    match to_case {
        CaseType::Snake => to_snake_case(s),
        CaseType::Camel => to_camel_case(s),
        CaseType::Pascal => to_pascal_case(s),
        CaseType::Kebab => to_kebab_case(s),
        CaseType::Title => to_title_case(s),
        CaseType::Train => to_train_case(s),
        CaseType::ScreamingSnake => to_screaming_snake_case(s),
        CaseType::Unknown => s.to_string(),
    }
}

pub fn auto_convert_case(s: &str, to_case: CaseType) -> String {
    let detected_case = detect_case(s);
    convert_case(s, detected_case, to_case)
}
