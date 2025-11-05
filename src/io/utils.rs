pub(crate) fn try_strip_leading_slash(target: &str) -> &str {
    match target.strip_prefix('/') {
        Some(stripped) => stripped,
        None => target,
    }
}
