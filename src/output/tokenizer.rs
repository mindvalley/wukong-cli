use once_cell::sync::Lazy;
use owo_colors::{colors::xterm::Gray, OwoColorize};
use regex::Regex;
use std::{char, fmt::Display};

pub struct OutputTokenizer;

#[derive(Clone, Debug, PartialEq)]
pub enum Punctuation {
    /// Colon: ':'
    Colon,
    /// Comma: ','
    Comma,
    /// Dash: '-'
    Dash,
    /// Exclamation: '!'
    Exclamation,
    /// Period: '.'
    Period,
    /// Question: '?'
    Question,
    /// Semicolon: ';'
    Semicolon,
}

impl Display for Punctuation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Punctuation::Colon => write!(f, ":"),
            Punctuation::Comma => write!(f, ","),
            Punctuation::Dash => write!(f, "-"),
            Punctuation::Exclamation => write!(f, "!"),
            Punctuation::Period => write!(f, "."),
            Punctuation::Question => write!(f, "?"),
            Punctuation::Semicolon => write!(f, ";"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    ShortCommitHash(String),
    LongCommitHash(String),
    Email(String),
    GithubUsername(String),
    PR(String),
    JiraTicket(String),
    BuildArtifact(String),
    Punctuation(Punctuation),
    Word(String),
    Whitespace,
    Unknown(String),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::ShortCommitHash(value) => write!(f, "{}", value.bright_yellow()),
            Token::LongCommitHash(value) => write!(f, "{}", value.bright_yellow()),
            Token::Email(value) => write!(f, "{}", value.bright_blue()),
            Token::GithubUsername(value) => write!(f, "{}", value.bright_green()),
            Token::PR(value) => write!(f, "{}", value.cyan()),
            Token::JiraTicket(value) => write!(f, "{}", value.cyan()),
            Token::BuildArtifact(value) => write!(f, "{}", value.bold().default_color()),
            Token::Punctuation(value) => write!(f, "{}", value),
            Token::Word(value) => write!(f, "{}", value),
            Token::Whitespace => write!(f, " "),
            Token::Unknown(value) => write!(f, "{}", value),
        }
    }
}

impl OutputTokenizer {
    pub fn tokenize(source: String) -> Vec<Token> {
        let mut words = source.split(char::is_whitespace).peekable();
        let mut tokens = Vec::new();

        while let Some(word) = words.next() {
            tokens.push(Token::Unknown(word.to_string()));
            if words.peek().is_some() {
                tokens.push(Token::Whitespace);
            }
        }

        Self::separate_end_punctuation(&mut tokens);
        Self::parse_word(&mut tokens);

        tokens
    }

    fn separate_end_punctuation(tokens: &mut Vec<Token>) {
        static PUNCTUATION_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"([\.\?\-:!,;]+)$").unwrap());

        let mut i = 0;
        while i < tokens.len() {
            let token = match tokens.get(i) {
                Some(Token::Unknown(token)) => token,
                _ => {
                    i += 1;
                    continue;
                }
            };

            let (before, punctuation, after) =
                if let Some(regex_match) = PUNCTUATION_REGEX.find(token) {
                    let punctuation = match regex_match.as_str() {
                        "!" => Punctuation::Exclamation,
                        "," => Punctuation::Comma,
                        "-" => Punctuation::Dash,
                        "." => Punctuation::Period,
                        ":" => Punctuation::Colon,
                        ";" => Punctuation::Semicolon,
                        "?" => Punctuation::Question,
                        _ => {
                            i += 1;
                            continue;
                        }
                    };

                    let before = token
                        .get(0..regex_match.start())
                        .filter(|s| s.len() > 0)
                        .map(|s| s.to_string());

                    let after = token
                        .get(regex_match.end()..token.len())
                        .filter(|s| s.len() > 0)
                        .map(|s| s.to_string());

                    (before, punctuation, after)
                } else {
                    i += 1;
                    continue;
                };

            // String before the punctuation match
            let mut insert = false;
            if let Some(before) = before {
                if let Some(elem) = tokens.get_mut(i) {
                    *elem = Token::Unknown(before);
                }
                i += 1;
                insert = true;
            }

            // Punctuation
            if insert {
                tokens.insert(i, Token::Punctuation(punctuation));
            } else {
                if let Some(elem) = tokens.get_mut(i) {
                    *elem = Token::Punctuation(punctuation);
                }
            }

            i += 1;

            // String after the punctuation match
            if let Some(after) = after {
                tokens.insert(i, Token::Unknown(after));
                i += 1;
            }
        }
    }

    fn parse_word(tokens: &mut Vec<Token>) {
        static SHORT_COMMIT_HASH_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(commit:[ ]{0,}([a-f0-9]{7}))|(([a-f0-9]{7}))").unwrap());
        static LONG_COMMIT_HASH_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(commit:[ ]{0,}([a-f0-9]{40}))|(([a-f0-9]{40}))").unwrap());
        static EMAIL_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"([\w_\.\-\+])+@([\w\-]+\.)+([\w]{2,10})+").unwrap());
        static GITHUB_USERNAME_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"@([a-z\d]+-)*[a-z\d]+").unwrap());
        static PR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"#([\d]+)").unwrap());
        static JIRA_TICKET_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Z]+-[\d]+").unwrap());
        static BUILD_ARTIFACT_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"[a-zA-z]+-build-\d+|build-\d+").unwrap());
        static WORD_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w+$").unwrap());

        for token in tokens.iter_mut() {
            match token {
                Token::Unknown(value) => {
                    if LONG_COMMIT_HASH_REGEX.is_match(&value) {
                        *token = Token::LongCommitHash(value.clone());
                    } else if SHORT_COMMIT_HASH_REGEX.is_match(&value) {
                        *token = Token::ShortCommitHash(value.clone());
                    } else if EMAIL_REGEX.is_match(&value) {
                        *token = Token::Email(value.clone());
                    } else if GITHUB_USERNAME_REGEX.is_match(&value) {
                        *token = Token::GithubUsername(value.clone());
                    } else if PR_REGEX.is_match(&value) {
                        *token = Token::PR(value.clone());
                    } else if JIRA_TICKET_REGEX.is_match(&value) {
                        *token = Token::JiraTicket(value.clone());
                    } else if BUILD_ARTIFACT_REGEX.is_match(&value) {
                        *token = Token::BuildArtifact(value.clone());
                    } else if WORD_REGEX.is_match(&value) {
                        *token = Token::Word(value.clone());
                    }
                }
                _ => continue,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sentence_with_email() {
        let sentence = "This is from abc@gmail.com.";
        assert_eq!(
            OutputTokenizer::tokenize(sentence.to_string()),
            vec![
                Token::Word("This".into()),
                Token::Word("is".into()),
                Token::Word("from".into()),
                Token::Email("abc@gmail.com".into()),
                Token::Punctuation(Punctuation::Period)
            ]
        );
    }

    #[test]
    fn test_sentence_with_different_tokens() {
        let sentence = "The last build is build-213. This is built from PR #31 by abc@gmail.com (@jk-gan). The commit hash is a9a97d98635e7a5218c554ee9a41132e3603cc97.";
        assert_eq!(
            OutputTokenizer::tokenize(sentence.to_string()),
            vec![
                Token::Word("The".into()),
                Token::Word("last".into()),
                Token::Word("build".into()),
                Token::Word("is".into()),
                Token::BuildArtifact("build-213".into()),
                Token::Punctuation(Punctuation::Period),
                Token::Word("This".into()),
                Token::Word("is".into()),
                Token::Word("built".into()),
                Token::Word("from".into()),
                Token::Word("PR".into()),
                Token::PR("#31".into()),
                Token::Word("by".into()),
                Token::Email("abc@gmail.com".into()),
                Token::GithubUsername("(@jk-gan)".into()),
                Token::Punctuation(Punctuation::Period),
                Token::Word("The".into()),
                Token::Word("commit".into()),
                Token::Word("hash".into()),
                Token::Word("is".into()),
                Token::LongCommitHash("a9a97d98635e7a5218c554ee9a41132e3603cc97".into()),
                Token::Punctuation(Punctuation::Period),
            ]
        );
    }
}
