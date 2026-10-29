/// Convert a slice of plain branch names as Strings to a Vec of colorized Strings.
///
/// Branches are combined with their remote heads:
///
/// ['main','origin/main'] -> ['origin/','main']
///
fn to_decorated_list(branch_names: &[&str]) -> Vec<String> {
    // TODO: Find out how to determine which branches are upstream of others
    branch_names.iter().map(|&s| String::from(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_branch_should_return_same_branch() {
        // Given
        let input = ["main"];

        // When
        let result = to_decorated_list(&input);

        // Then
        let expected = vec!["main"];
        assert_eq!(expected, result);
    }
}
