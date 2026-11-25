//! Minimal MediaWiki template reader (`{{Name|positional|key=value|key={{Nested|..}}}}`).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub name: String,
    pub args: Vec<Arg>,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arg {
    pub key: Option<String>,
    pub value: String,
}

impl Template {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.args
            .iter()
            .find(|a| {
                a.key
                    .as_deref()
                    .is_some_and(|k| k.eq_ignore_ascii_case(key))
            })
            .map(|a| a.value.as_str())
    }

    pub fn positional(&self) -> impl Iterator<Item = &str> {
        self.args
            .iter()
            .filter(|a| a.key.is_none())
            .map(|a| a.value.as_str())
    }

    pub fn is(&self, name: &str) -> bool {
        self.name.eq_ignore_ascii_case(name)
    }
}

pub fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("<!--") {
        out.push_str(&rest[..i]);
        match rest[i..].find("-->") {
            Some(j) => rest = &rest[i + j + 3..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

pub fn read_template(text: &str, start: usize) -> Option<Template> {
    let bytes = text.as_bytes();
    if !text[start..].starts_with("{{") {
        return None;
    }
    let mut i = start + 2;
    let mut braces = 1usize;
    let mut links = 0usize;
    let mut seg_start = i;
    let mut segments: Vec<&str> = Vec::new();
    while i < bytes.len() {
        match bytes.get(i..i + 2) {
            Some(b"{{") => {
                braces += 1;
                i += 2;
            }
            Some(b"}}") => {
                braces -= 1;
                if braces == 0 {
                    segments.push(&text[seg_start..i]);
                    let mut it = segments.into_iter();
                    let name = it.next()?.trim().to_string();
                    let args = it.map(parse_arg).collect();
                    return Some(Template {
                        name,
                        args,
                        start,
                        end: i + 2,
                    });
                }
                i += 2;
            }
            Some(b"[[") => {
                links += 1;
                i += 2;
            }
            Some(b"]]") => {
                links = links.saturating_sub(1);
                i += 2;
            }
            _ => {
                if bytes[i] == b'|' && braces == 1 && links == 0 {
                    segments.push(&text[seg_start..i]);
                    seg_start = i + 1;
                }
                i += 1;
            }
        }
    }
    None
}

fn parse_arg(segment: &str) -> Arg {
    let bytes = segment.as_bytes();
    let (mut depth, mut i) = (0usize, 0usize);
    while i < bytes.len() {
        match bytes.get(i..i + 2) {
            Some(b"{{") | Some(b"[[") => {
                depth += 1;
                i += 2;
                continue;
            }
            Some(b"}}") | Some(b"]]") => {
                depth = depth.saturating_sub(1);
                i += 2;
                continue;
            }
            _ => {}
        }
        if bytes[i] == b'=' && depth == 0 {
            let key = segment[..i].trim();
            if !key.is_empty() && !key.contains('{') && !key.contains('[') {
                return Arg {
                    key: Some(key.to_string()),
                    value: segment[i + 1..].trim().to_string(),
                };
            }
            break;
        }
        i += 1;
    }
    Arg {
        key: None,
        value: segment.trim().to_string(),
    }
}

pub fn find_templates(text: &str, name: &str) -> Vec<Template> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(i) = text[from..].find("{{") {
        let at = from + i;
        match read_template(text, at) {
            Some(t) if t.is(name) => {
                from = t.end;
                found.push(t);
            }
            Some(_) | None => from = at + 2,
        }
    }
    found
}
