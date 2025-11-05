pub(crate) fn try_strip_leading_slash(target: &str) -> &str {
    match target.strip_prefix('/') {
        Some(stripped) => stripped,
        None => target,
    }
}

/// Extracts xmlns attribute declarations from an XML tag
/// Returns vec of (attribute_name, uri) pairs
/// e.g., [("xmlns", "http://..."), ("xmlns:p", "http://...")]
pub fn parse_xmlns_attributes(tag_content: &str) -> Vec<(String, String)> {
    let mut attributes = Vec::new();
    let mut chars = tag_content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == 'x' {
            // Check for "xmlns" or "xmlns:"
            let mut attr_name = String::from("x");
            let mut is_xmlns = true;

            // Read rest of potential "xmlns" or "xmlns:"
            for expected in ['m', 'l', 'n', 's'] {
                if chars.peek() == Some(&expected) {
                    attr_name.push(chars.next().unwrap());
                } else {
                    is_xmlns = false;
                    break;
                }
            }

            if is_xmlns {
                // Check for colon (namespaced)
                if chars.peek() == Some(&':') {
                    attr_name.push(chars.next().unwrap());
                    // Read namespace prefix
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphabetic() || ch == '_' || ch == '-' {
                            attr_name.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                }

                // Expect equals sign
                if chars.next() == Some('=') && chars.next() == Some('"') {
                    let mut uri = String::new();
                    // Read until closing quote
                    while let Some(ch) = chars.next() {
                        if ch == '"' {
                            break;
                        }
                        uri.push(ch);
                    }
                    attributes.push((attr_name, uri));
                }
            }
        }
    }

    attributes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xmlns_attributes_simple() {
        let xml = r#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" unit="millimeter">"#;
        let attrs = parse_xmlns_attributes(xml);
        assert_eq!(attrs, vec![("xmlns".to_string(), "http://schemas.microsoft.com/3dmanufacturing/core/2015/02".to_string())]);
    }

    #[test]
    fn test_parse_xmlns_attributes_prefixed() {
        let xml = r#"<model xmlns="http://core" xmlns:p="http://prod" xmlns:b="http://beam">"#;
        let attrs = parse_xmlns_attributes(xml);
        assert_eq!(attrs, vec![
            ("xmlns".to_string(), "http://core".to_string()),
            ("xmlns:p".to_string(), "http://prod".to_string()),
            ("xmlns:b".to_string(), "http://beam".to_string()),
        ]);
    }

    #[test]
    fn test_parse_xmlns_attributes_empty() {
        let xml = r#"<model unit="millimeter">"#;
        let attrs = parse_xmlns_attributes(xml);
        assert_eq!(attrs, Vec::<(String, String)>::new());
    }

    #[test]
    fn test_parse_xmlns_attributes_mixed() {
        let xml = r#"<model xmlns="http://core" unit="millimeter" xmlns:p="http://prod" requiredextensions="ext">"#;
        let attrs = parse_xmlns_attributes(xml);
        assert_eq!(attrs, vec![
            ("xmlns".to_string(), "http://core".to_string()),
            ("xmlns:p".to_string(), "http://prod".to_string()),
        ]);
    }
}
