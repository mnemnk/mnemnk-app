use unicode_normalization::UnicodeNormalization;
use unicode_script::UnicodeScript;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
struct Token {
    text: String,
    script: unicode_script::Script,
}

/// ex. C++, mnemnk-app
fn is_latin_allowed(ch: char) -> bool {
    ch.is_ascii() && (ch.is_alphanumeric() || ch == '+' || ch == '-' || ch == '_')
}

fn tokenize(s: &str) -> Vec<Token> {
    let s = s.nfkc().collect::<String>();
    let s = s.trim().to_lowercase();
    let mut tokens = Vec::new();
    let mut current_text = String::new();
    let mut current_script: Option<unicode_script::Script> = None;

    for grapheme in s.graphemes(true) {
        // white space is a separator
        if grapheme.trim().is_empty() {
            if !current_text.is_empty() {
                tokens.push(Token {
                    text: current_text.clone(),
                    script: current_script.unwrap(),
                });
                current_text.clear();
                current_script = None;
            }
            continue;
        }

        // decide script by the first char
        let first_char = grapheme.chars().next().unwrap();
        let grapheme_script = first_char.script();

        match current_script {
            None => {
                current_script = Some(grapheme_script);
                current_text.push_str(grapheme);
            }
            Some(script) => {
                if script == unicode_script::Script::Latin {
                    if grapheme.chars().all(is_latin_allowed) {
                        current_text.push_str(grapheme);
                    } else {
                        tokens.push(Token {
                            text: current_text.clone(),
                            script,
                        });
                        current_text.clear();
                        current_script = Some(grapheme_script);
                        current_text.push_str(grapheme);
                    }
                } else {
                    // non-Latin
                    if grapheme_script == script {
                        current_text.push_str(grapheme);
                    } else {
                        tokens.push(Token {
                            text: current_text.clone(),
                            script,
                        });
                        current_text.clear();
                        current_script = Some(grapheme_script);
                        current_text.push_str(grapheme);
                    }
                }
            }
        }
    }

    if !current_text.is_empty() {
        tokens.push(Token {
            text: current_text,
            script: current_script.unwrap(),
        });
    }

    tokens
}

fn join_tokens(tokens: Vec<Token>) -> String {
    tokens
        .into_iter()
        .filter(|token| {
            token.script != unicode_script::Script::Common
                && token.script != unicode_script::Script::Hiragana
        })
        .map(|token| token.text)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn tokenize_text(s: &str) -> String {
    join_tokens(tokenize(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_latin() {
        let s = "This is a test. C++ Rust!🚀";
        let tokens = tokenize(s);
        assert_eq!(tokens.len(), 8);
        assert_eq!(tokens[0].text, "this");
        assert_eq!(tokens[0].script, unicode_script::Script::Latin);
        assert_eq!(tokens[1].text, "is");
        assert_eq!(tokens[1].script, unicode_script::Script::Latin);
        assert_eq!(tokens[2].text, "a");
        assert_eq!(tokens[2].script, unicode_script::Script::Latin);
        assert_eq!(tokens[3].text, "test");
        assert_eq!(tokens[3].script, unicode_script::Script::Latin);
        assert_eq!(tokens[4].text, ".");
        assert_eq!(tokens[4].script, unicode_script::Script::Common);
        assert_eq!(tokens[5].text, "c++");
        assert_eq!(tokens[5].script, unicode_script::Script::Latin);
        assert_eq!(tokens[6].text, "rust");
        assert_eq!(tokens[6].script, unicode_script::Script::Latin);
        assert_eq!(tokens[7].text, "!🚀");
        assert_eq!(tokens[7].script, unicode_script::Script::Common);
    }

    #[test]
    fn test_tokenize_ja() {
        let s = "こんにちはmnemnk-app。こんにちは、世界！";
        let tokens = tokenize(s);
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].text, "こんにちは");
        assert_eq!(tokens[0].script, unicode_script::Script::Hiragana);
        assert_eq!(tokens[1].text, "mnemnk-app");
        assert_eq!(tokens[1].script, unicode_script::Script::Latin);
        assert_eq!(tokens[2].text, "。");
        assert_eq!(tokens[2].script, unicode_script::Script::Common);
        assert_eq!(tokens[3].text, "こんにちは");
        assert_eq!(tokens[3].script, unicode_script::Script::Hiragana);
        assert_eq!(tokens[4].text, "、");
        assert_eq!(tokens[4].script, unicode_script::Script::Common);
        assert_eq!(tokens[5].text, "世界");
        assert_eq!(tokens[5].script, unicode_script::Script::Han);
        assert_eq!(tokens[6].text, "!");
        assert_eq!(tokens[6].script, unicode_script::Script::Common);
    }

    #[test]
    fn test_join_tokens() {
        let tokens = vec![
            Token {
                text: "C++".to_string(),
                script: unicode_script::Script::Latin,
            },
            Token {
                text: "mnemnk-app".to_string(),
                script: unicode_script::Script::Latin,
            },
            Token {
                text: "こんにちは".to_string(),
                script: unicode_script::Script::Hiragana,
            },
        ];
        let s = join_tokens(tokens);
        assert_eq!(s, "C++ mnemnk-app");
    }

    #[test]
    fn test_tokenize_text() {
        let s = "This is a test. C++ Rust!🚀";
        let s = tokenize_text(s);
        assert_eq!(s, "this is a test c++ rust");
    }

    #[test]
    fn test_tokenize_text_ja() {
        let s = "こんにちはmnemnk-app。こんにちは、世界！";
        let s = tokenize_text(s);
        assert_eq!(s, "mnemnk-app 世界");
    }
}
