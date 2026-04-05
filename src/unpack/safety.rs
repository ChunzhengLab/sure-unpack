/// Check a list of entry paths for safety issues.
/// Returns the paths that triggered warnings.
pub fn check_entries(entries: &[String]) -> Vec<Warning> {
    let mut warnings = Vec::new();
    for entry in entries {
        if is_absolute(entry) {
            warnings.push(Warning {
                path: entry.clone(),
                kind: WarningKind::AbsolutePath,
            });
        } else if has_traversal(entry) {
            warnings.push(Warning {
                path: entry.clone(),
                kind: WarningKind::PathTraversal,
            });
        }
    }
    warnings
}

/// Print warnings to stderr. Returns true if any warnings were found.
pub fn print_warnings(warnings: &[Warning]) -> bool {
    if warnings.is_empty() {
        return false;
    }
    for w in warnings {
        eprintln!("warning: {}: {}", w.kind.label(), w.path);
    }
    eprintln!(
        "warning: {} suspicious entry(ies) found in archive",
        warnings.len()
    );
    true
}

#[derive(Debug)]
pub struct Warning {
    pub path: String,
    pub kind: WarningKind,
}

#[derive(Debug)]
pub enum WarningKind {
    AbsolutePath,
    PathTraversal,
}

impl WarningKind {
    fn label(&self) -> &'static str {
        match self {
            WarningKind::AbsolutePath => "absolute path",
            WarningKind::PathTraversal => "path traversal (..)",
        }
    }
}

fn is_absolute(path: &str) -> bool {
    path.starts_with('/') || path.starts_with('\\')
}

fn has_traversal(path: &str) -> bool {
    for component in path.split('/') {
        if component == ".." {
            return true;
        }
    }
    // Also check backslash-separated paths (Windows archives)
    for component in path.split('\\') {
        if component == ".." {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_paths() {
        let entries = vec![
            "dir/file.txt".to_string(),
            "dir/".to_string(),
            "file.txt".to_string(),
            "./relative".to_string(),
        ];
        let warnings = check_entries(&entries);
        assert!(warnings.is_empty());
    }

    #[test]
    fn absolute_path() {
        let entries = vec!["/etc/passwd".to_string()];
        let warnings = check_entries(&entries);
        assert_eq!(warnings.len(), 1);
        assert!(matches!(warnings[0].kind, WarningKind::AbsolutePath));
    }

    #[test]
    fn path_traversal() {
        let entries = vec!["../escape.txt".to_string()];
        let warnings = check_entries(&entries);
        assert_eq!(warnings.len(), 1);
        assert!(matches!(warnings[0].kind, WarningKind::PathTraversal));
    }

    #[test]
    fn nested_traversal() {
        let entries = vec!["foo/../../etc/passwd".to_string()];
        let warnings = check_entries(&entries);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn backslash_traversal() {
        let entries = vec!["..\\escape.txt".to_string()];
        let warnings = check_entries(&entries);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn mixed_dangerous() {
        let entries = vec![
            "safe/file.txt".to_string(),
            "/absolute".to_string(),
            "../traversal".to_string(),
            "also/safe".to_string(),
        ];
        let warnings = check_entries(&entries);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn dotdot_in_name_is_not_traversal() {
        // "foo..bar" contains ".." but not as a path component
        let entries = vec!["foo..bar".to_string()];
        let warnings = check_entries(&entries);
        assert!(warnings.is_empty());
    }
}
