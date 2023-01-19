use once_cell::sync::Lazy;
use owo_colors::OwoColorize;
use regex::Regex;
use std::fmt::Display;

pub struct OutputTokenizer;

static END_PUNCTUATION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\.\?\-:!,;]+)$").unwrap());
static APPLICATION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^mv-[a-zA-z-]+$").unwrap());
static SHORT_COMMIT_HASH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(commit:[ ]{0,}([a-f0-9]{7}))$|^(([a-f0-9]{7}))$").unwrap());
static LONG_COMMIT_HASH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(commit:[ ]{0,}([a-f0-9]{40}))$|^(([a-f0-9]{40}))$").unwrap());
static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([\w_\.\-\+])+@([\w\-]+\.)+([\w]{2,10})+$").unwrap());
static GITHUB_USERNAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^@([a-z\d]+-)*[a-z\d]+$").unwrap());
static PR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^#([\d]+$)").unwrap());
static JIRA_TICKET_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Z]+-[\d]+$").unwrap());
static BUILD_ARTIFACT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-z]+-build-\d+$|^build-\d+$").unwrap());
static WORD_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w+$").unwrap());

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
pub enum Version {
    Blue,
    Green,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::Blue => write!(f, "{}", "Blue".blue()),
            Version::Green => write!(f, "{}", "Green".green()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Ok(String),
    Fail(String),
    Abort(String),
    Running(String),
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Ok(status) => write!(f, "{}", status.green().bold()),
            Status::Fail(status) => write!(f, "{}", status.red().bold()),
            Status::Abort(status) => write!(f, "{}", status.black().bold()),
            Status::Running(status) => write!(f, "{}", status.yellow().bold()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Application(String),
    ShortCommitHash(String),
    LongCommitHash(String),
    Email(String),
    GithubUsername(String),
    PR(String),
    JiraTicket(String),
    BuildArtifact(String),
    Punctuation(Punctuation),
    Parentheses(String),
    Version(Version),
    Status(Status),
    Word(String),
    Whitespace,
    Newline,
    Unknown(String),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Application(value) => write!(f, "{}", value.bright_green()),
            Token::ShortCommitHash(value) => write!(f, "{}", value.bright_yellow()),
            Token::LongCommitHash(value) => write!(f, "{}", value.bright_yellow()),
            Token::Email(value) => write!(f, "{}", value.bright_blue()),
            Token::GithubUsername(value) => write!(f, "{}", value.bright_green()),
            Token::PR(value) => write!(f, "{}", value.cyan()),
            Token::JiraTicket(value) => write!(f, "{}", value.cyan()),
            Token::BuildArtifact(value) => write!(f, "{}", value.bold().default_color()),
            Token::Punctuation(value) => write!(f, "{}", value),
            Token::Word(value) => write!(f, "{}", value),
            Token::Version(value) => write!(f, "{}", value),
            Token::Status(value) => write!(f, "{}", value),
            Token::Whitespace => write!(f, " "),
            Token::Newline => writeln!(f),
            Token::Unknown(value) => write!(f, "{}", value),
            Token::Parentheses(value) => write!(f, "{}", value),
        }
    }
}

impl OutputTokenizer {
    pub fn tokenize(source: String) -> Vec<Token> {
        let mut words = source.split(' ').peekable();
        let mut tokens = Vec::new();

        while let Some(word) = words.next() {
            // split with newline
            if word.contains("\n") {
                let mut word_without_newline = word.split('\n').peekable();

                while let Some(word) = word_without_newline.next() {
                    if word.contains('(') {
                        let open_index = word.find('(').unwrap();
                        tokens.push(Token::Parentheses("(".to_string()));

                        if word.contains(')') {
                            let close_index = word.find(')').unwrap();
                            if close_index != word.len() - 1 {
                                let before_value =
                                    word.get((open_index + 1)..(close_index)).unwrap_or("");
                                let after_value = word.get((close_index + 1)..).unwrap_or("");
                                tokens.push(Token::Unknown(before_value.to_string()));
                                tokens.push(Token::Parentheses(")".to_string()));
                                tokens.push(Token::Unknown(after_value.to_string()));
                            } else {
                                let value = word.get((open_index + 1)..(close_index)).unwrap_or("");
                                tokens.push(Token::Unknown(value.to_string()));
                                tokens.push(Token::Parentheses(")".to_string()));
                            }
                        } else {
                            let value = word.get((open_index + 1)..).unwrap_or("");
                            tokens.push(Token::Unknown(value.to_string()));
                        }
                    } else if word.contains(')') {
                        let close_index = word.find(')').unwrap();
                        if close_index != word.len() - 1 {
                            let before_value = word.get(..(close_index)).unwrap_or("");
                            let after_value = word.get((close_index + 1)..).unwrap_or("");
                            tokens.push(Token::Unknown(before_value.to_string()));
                            tokens.push(Token::Parentheses(")".to_string()));
                            tokens.push(Token::Unknown(after_value.to_string()));
                        } else {
                            let value = word.get(..(close_index)).unwrap_or("");
                            tokens.push(Token::Unknown(value.to_string()));
                            tokens.push(Token::Parentheses(")".to_string()));
                        }
                    } else {
                        tokens.push(Token::Unknown(word.to_string()));
                    }

                    // since we split with newline, here we need to add back Token::Newline so we
                    // will know where to print the newline later
                    if word_without_newline.peek().is_some() {
                        tokens.push(Token::Newline);
                    }
                }
            } else {
                // this is more readable
                #[allow(clippy::collapsible-else-if)]
                if word.contains('(') {
                    let open_index = word.find('(').unwrap();
                    tokens.push(Token::Parentheses("(".to_string()));

                    if word.contains(')') {
                        let close_index = word.find(')').unwrap();
                        if close_index != word.len() - 1 {
                            let before_value =
                                word.get((open_index + 1)..(close_index)).unwrap_or("");
                            let after_value = word.get((close_index + 1)..).unwrap_or("");
                            tokens.push(Token::Unknown(before_value.to_string()));
                            tokens.push(Token::Parentheses(")".to_string()));
                            tokens.push(Token::Unknown(after_value.to_string()));
                        } else {
                            let value = word.get((open_index + 1)..(close_index)).unwrap_or("");
                            tokens.push(Token::Unknown(value.to_string()));
                            tokens.push(Token::Parentheses(")".to_string()));
                        }
                    } else {
                        let value = word.get((open_index + 1)..).unwrap_or("");
                        tokens.push(Token::Unknown(value.to_string()));
                    }
                } else if word.contains(')') {
                    let close_index = word.find(')').unwrap();
                    if close_index != word.len() - 1 {
                        let before_value = word.get(..(close_index)).unwrap_or("");
                        let after_value = word.get((close_index + 1)..).unwrap_or("");
                        tokens.push(Token::Unknown(before_value.to_string()));
                        tokens.push(Token::Parentheses(")".to_string()));
                        tokens.push(Token::Unknown(after_value.to_string()));
                    } else {
                        let value = word.get(..(close_index)).unwrap_or("");
                        tokens.push(Token::Unknown(value.to_string()));
                        tokens.push(Token::Parentheses(")".to_string()));
                    }
                } else {
                    tokens.push(Token::Unknown(word.to_string()));
                }
            }

            // since we split with whitespace, here we need to add back Token::Whitespace so we
            // will know where to print the whitespace later
            if words.peek().is_some() {
                tokens.push(Token::Whitespace);
            }
        }

        Self::separate_end_punctuation(&mut tokens);
        Self::parse_word(&mut tokens);

        tokens
    }

    fn separate_end_punctuation(tokens: &mut Vec<Token>) {
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
                if let Some(regex_match) = END_PUNCTUATION_REGEX.find(token) {
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
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());

                    let after = token
                        .get(regex_match.end()..token.len())
                        .filter(|s| !s.is_empty())
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
            } else if let Some(elem) = tokens.get_mut(i) {
                *elem = Token::Punctuation(punctuation);
            }

            i += 1;

            // String after the punctuation match
            if let Some(after) = after {
                tokens.insert(i, Token::Unknown(after));
                i += 1;
            }
        }
    }

    fn parse_word(tokens: &mut [Token]) {
        for token in tokens.iter_mut() {
            match token {
                Token::Unknown(value) => {
                    if APPLICATION_REGEX.is_match(value) {
                        *token = Token::Application(value.clone());
                    } else if LONG_COMMIT_HASH_REGEX.is_match(value) {
                        *token = Token::LongCommitHash(value.clone());
                    } else if SHORT_COMMIT_HASH_REGEX.is_match(value) {
                        *token = Token::ShortCommitHash(value.clone());
                    } else if EMAIL_REGEX.is_match(value) {
                        *token = Token::Email(value.clone());
                    } else if GITHUB_USERNAME_REGEX.is_match(value) {
                        *token = Token::GithubUsername(value.clone());
                    } else if PR_REGEX.is_match(value) {
                        *token = Token::PR(value.clone());
                    } else if JIRA_TICKET_REGEX.is_match(value) {
                        *token = Token::JiraTicket(value.clone());
                    } else if BUILD_ARTIFACT_REGEX.is_match(value) {
                        *token = Token::BuildArtifact(value.clone());
                    } else if &value.to_lowercase() == "blue" || &value.to_lowercase() == "green" {
                        match value.to_lowercase().as_str() {
                            "blue" => {
                                *token = Token::Version(Version::Blue);
                            }
                            "green" => *token = Token::Version(Version::Green),
                            _ => unreachable!(),
                        }
                    } else {
                        match value.as_str() {
                            "SUCCESS" | "[SUCCESS]" | "SUCCEEDED" | "[SUCCEEDED]" => {
                                *token = Token::Status(Status::Ok(value.clone()));
                            }
                            "TERMINAL" | "FAILURE" | "[TERMINAL]" | "[FAILURE]" => {
                                *token = Token::Status(Status::Fail(value.clone()));
                            }
                            "ABORT" | "CANCELED" | "CANCELLED" => {
                                *token = Token::Status(Status::Abort(value.clone()));
                            }
                            "RUNNING" | "BUILDING" | "[RUNNING]" | "[BUILDING]" => {
                                *token = Token::Status(Status::Running(value.clone()));
                            }
                            _ => {
                                if WORD_REGEX.is_match(value) {
                                    *token = Token::Word(value.clone());
                                }
                            }
                        }
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
    fn test_end_punctuation_regex() {
        let test_group = [
            ("a,", Some(",")),
            ("a.", Some(".")),
            ("a?", Some("?")),
            ("a, ", None),
            ("a: b, c: d", None),
        ];

        for test in test_group {
            assert_eq!(
                END_PUNCTUATION_REGEX.find(test.0).map(|x| x.as_str()),
                test.1
            );
        }
    }

    #[test]
    fn test_application_regex() {
        let test_group = [
            ("mv-platform", Some("mv-platform")),
            ("mv-wukong-ci-mock", Some("mv-wukong-ci-mock")),
            ("mv-base", Some("mv-base")),
            ("mv-base ", None),
            (" mv-base", None),
            (" mv-base ", None),
            ("platform", None),
        ];

        for test in test_group {
            assert_eq!(APPLICATION_REGEX.find(test.0).map(|x| x.as_str()), test.1);
        }
    }

    #[test]
    fn test_short_commit_hash_regex() {
        let test_group = [
            ("97dd2ae", Some("97dd2ae")),
            ("abcd123", Some("abcd123")),
            ("97dd2ae ", None),
            (" 97dd2ae", None),
            (" 97dd2ae ", None),
            ("97dd2aeb", None), // short commit hash regex only match 7 characters
            ("(97dd2ae)", None),
            ("97dd2ae065771908ee9ae0fa08ccdb58b5a6b18f", None),
        ];

        for test in test_group {
            assert_eq!(
                SHORT_COMMIT_HASH_REGEX.find(test.0).map(|x| x.as_str()),
                test.1
            );
        }
    }

    #[test]
    fn test_long_commit_hash_regex() {
        let test_group = [
            ("97dd2ae", None),
            ("abcd123", None),
            ("97dd2ae065771908ee9ae0fa08ccdb58b5a6b18f ", None),
            (" 97dd2ae065771908ee9ae0fa08ccdb58b5a6b18f", None),
            (" 97dd2ae065771908ee9ae0fa08ccdb58b5a6b18f ", None),
            ("(97dd2ae065771908ee9ae0fa08ccdb58b5a6b18f)", None),
            (
                "97dd2ae065771908ee9ae0fa08ccdb58b5a6b18f",
                Some("97dd2ae065771908ee9ae0fa08ccdb58b5a6b18f"),
            ),
        ];

        for test in test_group {
            assert_eq!(
                LONG_COMMIT_HASH_REGEX.find(test.0).map(|x| x.as_str()),
                test.1
            );
        }
    }

    #[test]
    fn test_email_regex() {
        let test_group = [
            ("test@example.com", Some("test@example.com")),
            ("admin@example.com", Some("admin@example.com")),
            (
                "this.is.a.test.email@example.com",
                Some("this.is.a.test.email@example.com"),
            ),
            ("test", None),
            ("test@example", None),
            ("@example", None),
            ("test@example.com ", None),
            (" test@example.com", None),
            (" test@example.com ", None),
            ("(test@example.com)", None),
        ];

        for test in test_group {
            assert_eq!(EMAIL_REGEX.find(test.0).map(|x| x.as_str()), test.1);
        }
    }

    #[test]
    fn test_github_username_regex() {
        let test_group = [
            ("@jk-gan", Some("@jk-gan")),
            ("@josevalim", Some("@josevalim")),
            ("@example", Some("@example")),
            ("@jk-gan ", None),
            (" @jk-gan", None),
            (" @jk-gan ", None),
            ("(@jk-gan)", None),
            ("jk-gan", None),
            ("test", None),
            ("test@example.com", None),
            ("this.is.a.test.email@example.com", None),
        ];

        for test in test_group {
            assert_eq!(
                GITHUB_USERNAME_REGEX.find(test.0).map(|x| x.as_str()),
                test.1
            );
        }
    }

    #[test]
    fn test_pr_regex() {
        let test_group = [
            ("#1", Some("#1")),
            ("#10", Some("#10")),
            ("#100", Some("#100")),
            ("#1000", Some("#1000")),
            ("#100 ", None),
            (" #100", None),
            (" #100 ", None),
            ("(#100)", None),
            ("100", None),
            ("#abc", None),
        ];

        for test in test_group {
            assert_eq!(PR_REGEX.find(test.0).map(|x| x.as_str()), test.1);
        }
    }

    #[test]
    fn test_jira_ticket_regex() {
        let test_group = [
            ("MVBASE-9999", Some("MVBASE-9999")),
            ("PXP-9999", Some("PXP-9999")),
            ("UP-9999", Some("UP-9999")),
            ("OPS-9999", Some("OPS-9999")),
            ("ABC-9999", Some("ABC-9999")),
            ("PXP_9999", None),
            ("PXP-9999 ", None),
            (" PXP-9999", None),
            (" PXP-9999 ", None),
            ("(PXP-9999)", None),
            ("9999", None),
            ("PXP-abc", None),
        ];

        for test in test_group {
            assert_eq!(JIRA_TICKET_REGEX.find(test.0).map(|x| x.as_str()), test.1);
        }
    }

    #[test]
    fn test_build_artifact_regex() {
        let test_group = [
            ("main-build-100", Some("main-build-100")),
            ("dev-build-100", Some("dev-build-100")),
            ("staging-build-100", Some("staging-build-100")),
            ("build-100", Some("build-100")),
            ("build_100", None),
            ("BUILD-100", None),
            ("build-100 ", None),
            (" build-100", None),
            (" build-100 ", None),
            ("(build-100)", None),
            ("9999", None),
            ("mainbuild-100", None),
        ];

        for test in test_group {
            assert_eq!(
                BUILD_ARTIFACT_REGEX.find(test.0).map(|x| x.as_str()),
                test.1
            );
        }
    }

    #[test]
    fn test_word_regex() {
        let test_group = [
            ("this", Some("this")),
            ("is", Some("is")),
            ("testing", Some("testing")),
            ("testing123", Some("testing123")),
            ("9999", Some("9999")),
            ("testing.", None),
            ("testing-1", None),
            (" testing", None),
            ("testing ", None),
            (" testing ", None),
            ("(testing)", None),
        ];

        for test in test_group {
            assert_eq!(WORD_REGEX.find(test.0).map(|x| x.as_str()), test.1);
        }
    }

    #[test]
    fn test_sentence_with_email() {
        let sentence = "This is from abc@gmail.com.";
        assert_eq!(
            OutputTokenizer::tokenize(sentence.to_string()),
            vec![
                Token::Word("This".into()),
                Token::Whitespace,
                Token::Word("is".into()),
                Token::Whitespace,
                Token::Word("from".into()),
                Token::Whitespace,
                Token::Email("abc@gmail.com".into()),
                Token::Punctuation(Punctuation::Period)
            ]
        );
    }

    #[test]
    fn test_sentence_with_different_tokens() {
        let sentence = "The last build for mv-ci-mock is build-213. This is built from PR #31 by abc@gmail.com (@jk-gan).\nThe commit hash is a9a97d98635e7a5218c554ee9a41132e3603cc97.";
        assert_eq!(
            OutputTokenizer::tokenize(sentence.to_string()),
            vec![
                Token::Word("The".into()),
                Token::Whitespace,
                Token::Word("last".into()),
                Token::Whitespace,
                Token::Word("build".into()),
                Token::Whitespace,
                Token::Word("for".into()),
                Token::Whitespace,
                Token::Application("mv-ci-mock".into()),
                Token::Whitespace,
                Token::Word("is".into()),
                Token::Whitespace,
                Token::BuildArtifact("build-213".into()),
                Token::Punctuation(Punctuation::Period),
                Token::Whitespace,
                Token::Word("This".into()),
                Token::Whitespace,
                Token::Word("is".into()),
                Token::Whitespace,
                Token::Word("built".into()),
                Token::Whitespace,
                Token::Word("from".into()),
                Token::Whitespace,
                Token::Word("PR".into()),
                Token::Whitespace,
                Token::PR("#31".into()),
                Token::Whitespace,
                Token::Word("by".into()),
                Token::Whitespace,
                Token::Email("abc@gmail.com".into()),
                Token::Whitespace,
                Token::Parentheses("(".into()),
                Token::GithubUsername("@jk-gan".into()),
                Token::Parentheses(")".into()),
                Token::Punctuation(Punctuation::Period),
                Token::Newline,
                Token::Word("The".into()),
                Token::Whitespace,
                Token::Word("commit".into()),
                Token::Whitespace,
                Token::Word("hash".into()),
                Token::Whitespace,
                Token::Word("is".into()),
                Token::Whitespace,
                Token::LongCommitHash("a9a97d98635e7a5218c554ee9a41132e3603cc97".into()),
                Token::Punctuation(Punctuation::Period),
            ]
        );
    }
}
