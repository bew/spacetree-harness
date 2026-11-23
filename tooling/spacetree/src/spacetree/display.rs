//! Rendering of paths in messages: absolute paths, with the home directory shortened.

use std::env;
use std::path::Path;

/// Render `path` for humans, with the home directory shortened to `~`.
///
/// Reads the home directory from the environment; without it, the path is rendered as is.
pub(crate) fn display_path(path: &Path) -> String {
    match env::var_os("HOME") {
        Some(home) => shorten_home(path, Path::new(&home)),
        None => path.display().to_string(),
    }
}

/// Render `path` with `home` replaced by `~`, leaving paths outside `home` untouched.
///
/// Comparison is component-wise, so a sibling directory that merely shares the home name
/// prefix (`/Users/ben` against `/Users/ben-x`) is not shortened.
fn shorten_home(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(relative) if relative.as_os_str().is_empty() => "~".to_string(),
        Ok(relative) => format!("~/{}", relative.display()),
        Err(_) => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorten_home_renders_a_path_inside_home() {
        let rendered = shorten_home(Path::new("/Users/ben/self/tree"), Path::new("/Users/ben"));

        assert_eq!(rendered, "~/self/tree");
    }

    #[test]
    fn shorten_home_renders_the_home_directory_itself() {
        let rendered = shorten_home(Path::new("/Users/ben"), Path::new("/Users/ben"));

        assert_eq!(rendered, "~");
    }

    #[test]
    fn shorten_home_leaves_a_path_outside_home() {
        let rendered = shorten_home(Path::new("/opt/tree"), Path::new("/Users/ben"));

        assert_eq!(rendered, "/opt/tree");
    }

    #[test]
    fn shorten_home_leaves_a_sibling_sharing_the_home_name_prefix() {
        let rendered = shorten_home(Path::new("/Users/ben-x/tree"), Path::new("/Users/ben"));

        assert_eq!(rendered, "/Users/ben-x/tree");
    }
}
