use std::fs;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};

pub(super) fn shebang_format_for_path(
    path: &Path,
    metadata: &fs::Metadata,
) -> Result<Option<&'static str>> {
    if !is_executable(metadata) || is_symlink(path) {
        return Ok(None);
    }

    let mut file =
        fs::File::open(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut buf = [0u8; 128];
    let read = file
        .read(&mut buf)
        .with_context(|| format!("failed to read `{}`", path.display()))?;
    let head = String::from_utf8_lossy(&buf[..read]);
    let Some(first_line) = head.lines().next() else {
        return Ok(None);
    };
    if !first_line.starts_with("#!") {
        return Ok(None);
    }

    let mut tokens = first_line[2..].split_whitespace();
    let Some(first_token) = tokens.next() else {
        return Ok(None);
    };
    let interpreter = if Path::new(first_token)
        .file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with("env"))
    {
        let Some(second_token) = tokens.next() else {
            return Ok(None);
        };
        if second_token.starts_with('-') {
            return Ok(None);
        }
        second_token
    } else {
        first_token
    };

    let Some(raw_name) = Path::new(interpreter).file_name() else {
        return Ok(None);
    };
    let raw_name = raw_name.to_string_lossy();
    if raw_name.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        return Ok(None);
    }

    Ok(shebang_name_to_format(&normalize_shebang_name(&raw_name)))
}

fn shebang_name_to_format(name: &str) -> Option<&'static str> {
    match name {
        "bash" | "sh" | "zsh" | "dash" | "ksh" => Some("bash"),
        "python" => Some("python"),
        "ruby" => Some("ruby"),
        "perl" => Some("perl"),
        "php" => Some("php"),
        "node" | "nodejs" => Some("javascript"),
        "lua" => Some("lua"),
        "tclsh" | "wish" => Some("tcl"),
        "groovy" => Some("groovy"),
        "awk" | "gawk" | "nawk" => Some("awk"),
        "rscript" => Some("r"),
        _ => None,
    }
}

fn normalize_shebang_name(raw_name: &str) -> String {
    let mut end = raw_name.len();
    if raw_name.as_bytes().last().is_some_and(u8::is_ascii_digit) {
        while end > 0
            && raw_name.as_bytes()[end - 1].is_ascii()
            && (raw_name.as_bytes()[end - 1].is_ascii_digit()
                || raw_name.as_bytes()[end - 1] == b'.')
        {
            end -= 1;
        }
    }
    raw_name[..end].to_ascii_lowercase()
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;

    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &fs::Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn normalizes_versioned_interpreter_names() {
        assert_eq!(normalize_shebang_name("python3.11"), "python");
        assert_eq!(normalize_shebang_name("NODEJS"), "nodejs");
        assert_eq!(normalize_shebang_name("perl5"), "perl");
        assert_eq!(normalize_shebang_name("rscript"), "rscript");
    }

    #[test]
    fn maps_common_interpreter_names_to_formats() {
        assert_eq!(shebang_name_to_format("bash"), Some("bash"));
        assert_eq!(shebang_name_to_format("nodejs"), Some("javascript"));
        assert_eq!(shebang_name_to_format("gawk"), Some("awk"));
        assert_eq!(shebang_name_to_format("rscript"), Some("r"));
        assert_eq!(shebang_name_to_format("unknown"), None);
    }

    #[cfg(unix)]
    #[test]
    fn detects_executable_env_shebangs() {
        let path = unique_temp_path("shebang-env");
        std::fs::write(&path, "#!/usr/bin/env python3.11\nprint('ok')\n").unwrap();
        make_executable(&path);
        let metadata = std::fs::metadata(&path).unwrap();

        let format = shebang_format_for_path(&path, &metadata).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(format, Some("python"));
    }

    #[cfg(unix)]
    #[test]
    fn ignores_non_executable_or_env_option_shebangs() {
        let non_executable = unique_temp_path("shebang-non-executable");
        std::fs::write(&non_executable, "#!/usr/bin/env node\nconsole.log(1)\n").unwrap();
        let metadata = std::fs::metadata(&non_executable).unwrap();
        assert_eq!(
            shebang_format_for_path(&non_executable, &metadata).unwrap(),
            None
        );

        let env_option = unique_temp_path("shebang-env-option");
        std::fs::write(&env_option, "#!/usr/bin/env -S node\nconsole.log(1)\n").unwrap();
        make_executable(&env_option);
        let metadata = std::fs::metadata(&env_option).unwrap();
        assert_eq!(
            shebang_format_for_path(&env_option, &metadata).unwrap(),
            None
        );

        let _ = std::fs::remove_file(non_executable);
        let _ = std::fs::remove_file(env_option);
    }

    #[cfg(unix)]
    fn make_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = std::fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }

    fn unique_temp_path(label: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("jscpd-rs-{label}-{}-{suffix}", std::process::id()))
    }
}
