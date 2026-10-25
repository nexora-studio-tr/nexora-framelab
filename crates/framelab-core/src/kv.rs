use crate::model::Result;
use std::ops::Range;

#[derive(Clone, Debug)]
pub enum Value {
    Text(String, Range<usize>),
    Object(Vec<Entry>),
}
#[derive(Clone, Debug)]
pub struct Entry {
    pub key: String,
    pub value: Value,
}
#[derive(Clone, Debug)]
struct Token {
    value: String,
    span: Range<usize>,
    brace: bool,
}

fn tokenize(text: &str) -> Result<Vec<Token>> {
    if text.len() > 1024 * 1024 {
        return Err("CONFIG_TOO_LARGE".into());
    }
    let bytes = text.as_bytes();
    let mut i = if bytes.starts_with(&[239, 187, 191]) {
        3
    } else {
        0
    };
    let mut out = Vec::new();
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if bytes.get(i..i + 2) == Some(b"//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if out.len() > 16000 {
            return Err("CONFIG_TOO_COMPLEX".into());
        }
        let start = i;
        if matches!(bytes[i], b'{' | b'}') {
            out.push(Token {
                value: (bytes[i] as char).to_string(),
                span: i..i + 1,
                brace: true,
            });
            i += 1;
            continue;
        }
        if bytes[i] == b'"' {
            i += 1;
            let mut value = String::new();
            let mut closed = false;
            while i < bytes.len() {
                if bytes[i] == b'"' {
                    i += 1;
                    closed = true;
                    break;
                }
                if bytes[i] == b'\\' && matches!(bytes.get(i + 1), Some(b'\\' | b'"')) {
                    i += 1;
                    value.push(bytes[i] as char);
                    i += 1;
                } else {
                    let ch = text[i..].chars().next().ok_or("CONFIG_INVALID_UTF8")?;
                    value.push(ch);
                    i += ch.len_utf8();
                }
            }
            if !closed {
                return Err("CONFIG_UNCLOSED_STRING".into());
            }
            out.push(Token {
                value,
                span: start..i,
                brace: false,
            });
        } else {
            while i < bytes.len()
                && !bytes[i].is_ascii_whitespace()
                && !matches!(bytes[i], b'{' | b'}')
            {
                i += 1;
            }
            let value = text[start..i].to_string();
            if value.starts_with('#') || value == "=" {
                return Err("CONFIG_UNSUPPORTED_SYNTAX".into());
            }
            out.push(Token {
                value,
                span: start..i,
                brace: false,
            });
        }
    }
    Ok(out)
}
fn object(tokens: &[Token], at: &mut usize, depth: usize, nested: bool) -> Result<Vec<Entry>> {
    if depth > 16 {
        return Err("CONFIG_TOO_DEEP".into());
    }
    let mut entries: Vec<Entry> = Vec::new();
    while *at < tokens.len() {
        let key = &tokens[*at];
        if key.brace && key.value == "}" {
            if !nested {
                return Err("CONFIG_UNEXPECTED_BRACE".into());
            }
            *at += 1;
            return Ok(entries);
        }
        if key.brace {
            return Err("CONFIG_EXPECTED_KEY".into());
        }
        if entries
            .iter()
            .any(|e| e.key.eq_ignore_ascii_case(&key.value))
        {
            return Err("CONFIG_DUPLICATE_KEY".into());
        }
        let name = key.value.clone();
        *at += 1;
        let token = tokens.get(*at).ok_or("CONFIG_MISSING_VALUE")?;
        let value = if token.brace && token.value == "{" {
            *at += 1;
            Value::Object(object(tokens, at, depth + 1, true)?)
        } else if token.brace {
            return Err("CONFIG_MISSING_VALUE".into());
        } else {
            *at += 1;
            Value::Text(token.value.clone(), token.span.clone())
        };
        entries.push(Entry { key: name, value });
    }
    if nested {
        return Err("CONFIG_UNCLOSED_OBJECT".into());
    }
    Ok(entries)
}
pub fn parse(text: &str) -> Result<Vec<Entry>> {
    let tokens = tokenize(text)?;
    object(&tokens, &mut 0, 0, false)
}
pub fn children<'a>(entries: &'a [Entry], key: &str) -> Option<&'a [Entry]> {
    match &entries
        .iter()
        .find(|e| e.key.eq_ignore_ascii_case(key))?
        .value
    {
        Value::Object(v) => Some(v),
        _ => None,
    }
}
pub fn text<'a>(entries: &'a [Entry], key: &str) -> Option<&'a str> {
    match &entries
        .iter()
        .find(|e| e.key.eq_ignore_ascii_case(key))?
        .value
    {
        Value::Text(v, _) => Some(v),
        _ => None,
    }
}
pub fn video(text: &str) -> Result<Vec<Entry>> {
    let root = parse(text)?;
    if root.len() != 1 {
        return Err("CONFIG_UNEXPECTED_ROOT".into());
    }
    children(&root, "video.cfg")
        .map(|x| x.to_vec())
        .ok_or_else(|| "CONFIG_UNEXPECTED_ROOT".into())
}
pub fn replace(text: &str, patches: &[crate::model::Patch], reverse: bool) -> Result<String> {
    let entries = video(text)?;
    let mut changes = Vec::new();
    for patch in patches {
        let from = if reverse { &patch.after } else { &patch.before };
        let to = if reverse { &patch.before } else { &patch.after };
        let entry = entries
            .iter()
            .find(|e| e.key == patch.key)
            .ok_or("CONFIG_FIELD_MISSING")?;
        let Value::Text(current, span) = &entry.value else {
            return Err("CONFIG_FIELD_INVALID".into());
        };
        if current == to {
            continue;
        }
        if current != from {
            return Err("CONFIG_CONFLICT".into());
        }
        if to.len() > 16 || !to.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return Err("CONFIG_VALUE_INVALID".into());
        }
        changes.push((span.clone(), format!("\"{to}\"")));
    }
    changes.sort_by_key(|(span, _)| std::cmp::Reverse(span.start));
    let mut result = text.to_string();
    for (span, value) in changes {
        result.replace_range(span, &value);
    }
    Ok(result)
}
