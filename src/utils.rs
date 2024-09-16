pub fn split_slug(slug: &str) -> Option<(String, String)> {
    // Attempt to split the slug into two parts based on the '/'
    match slug.split_once('/') {
        Some((org_id, name)) => Some((org_id.to_string(), name.to_string())),
        None => None, // Return None if the slug does not contain a '/'
    }
}
