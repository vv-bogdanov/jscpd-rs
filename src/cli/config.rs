use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde::de::{Error as DeError, MapAccess, Visitor};

use super::parsing::{
    compile_patterns, parse_format_mappings, parse_js_number, parse_size, split_csv,
};
use super::{ExitCode, FormatMappings, Options};

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileConfig {
    execution_id: Option<String>,
    path: Option<OneOrMany>,
    pattern: Option<String>,
    ignore: Option<OneOrMany>,
    reporters: Option<OneOrMany>,
    listeners: Option<OneOrMany>,
    reporters_options: Option<serde_json::Map<String, serde_json::Value>>,
    output: Option<PathBuf>,
    format: Option<OneOrMany>,
    formats_exts: Option<FormatMappingsConfig>,
    formats_names: Option<FormatMappingsConfig>,
    ignore_pattern: Option<OneOrMany>,
    #[serde(default, deserialize_with = "deserialize_optional_usize_or_string")]
    min_lines: Option<usize>,
    min_tokens: Option<usize>,
    #[serde(default, deserialize_with = "deserialize_optional_usize_or_string")]
    max_lines: Option<usize>,
    max_size: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_f64_or_string")]
    threshold: Option<f64>,
    mode: Option<String>,
    store: Option<String>,
    store_path: Option<PathBuf>,
    blame: Option<bool>,
    cache: Option<bool>,
    silent: Option<bool>,
    absolute: Option<bool>,
    no_symlinks: Option<bool>,
    ignore_case: Option<bool>,
    gitignore: Option<bool>,
    debug: Option<bool>,
    verbose: Option<bool>,
    skip_local: Option<bool>,
    exit_code: Option<ExitCodeConfig>,
    no_tips: Option<bool>,
    tokens_to_skip: Option<OneOrMany>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    fn into_vec(self) -> Vec<String> {
        match self {
            Self::One(value) => split_csv(&value),
            Self::Many(values) => values,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ExitCodeConfig {
    Boolean(bool),
    Number(f64),
    String(String),
}

impl From<ExitCodeConfig> for ExitCode {
    fn from(value: ExitCodeConfig) -> Self {
        match value {
            ExitCodeConfig::Boolean(value) => Self::Boolean(value),
            ExitCodeConfig::Number(value) => Self::Number(value),
            ExitCodeConfig::String(value) => Self::String(value),
        }
    }
}

fn deserialize_optional_usize_or_string<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<usize>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Number(number)) => number
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .map(Some)
            .ok_or_else(|| D::Error::custom("expected a non-negative integer")),
        Some(serde_json::Value::String(value)) => parse_config_usize_string(&value)
            .map(Some)
            .map_err(D::Error::custom),
        Some(value) => Err(D::Error::custom(format!(
            "invalid type: {}, expected integer or string",
            json_type_name(&value)
        ))),
    }
}

fn parse_config_usize_string(value: &str) -> std::result::Result<usize, String> {
    let number = parse_js_number(value)?;
    if !number.is_finite() || number < 0.0 || number.fract() != 0.0 {
        return Err(format!("invalid integer `{value}`"));
    }
    if number > usize::MAX as f64 {
        return Err(format!("integer `{value}` is too large"));
    }
    Ok(number as usize)
}

fn deserialize_optional_f64_or_string<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Number(number)) => number
            .as_f64()
            .map(Some)
            .ok_or_else(|| D::Error::custom("expected a finite number")),
        Some(serde_json::Value::String(value)) => {
            parse_js_number(&value).map(Some).map_err(D::Error::custom)
        }
        Some(value) => Err(D::Error::custom(format!(
            "invalid type: {}, expected number or string",
            json_type_name(&value)
        ))),
    }
}

fn json_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FormatMappingsConfig {
    String(String),
    Map(OrderedFormatMappings),
}

#[derive(Debug)]
struct OrderedFormatMappings(Vec<(String, Vec<String>)>);

impl<'de> Deserialize<'de> for OrderedFormatMappings {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(OrderedFormatMappingsVisitor)
    }
}

struct OrderedFormatMappingsVisitor;

impl<'de> Visitor<'de> for OrderedFormatMappingsVisitor {
    type Value = OrderedFormatMappings;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a format-to-values mapping object")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut items = Vec::with_capacity(map.size_hint().unwrap_or(0));
        while let Some((format, values)) = map.next_entry::<String, Vec<String>>()? {
            items.push((format, values));
        }
        Ok(OrderedFormatMappings(items))
    }
}

impl FormatMappingsConfig {
    fn into_mappings(self) -> FormatMappings {
        match self {
            Self::String(value) => parse_format_mappings(&value),
            Self::Map(map) => FormatMappings(map.0),
        }
    }
}

pub(super) fn read_config(path: Option<&Path>) -> Result<Option<(FileConfig, PathBuf, PathBuf)>> {
    let path = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(".jscpd.json"));
    if !path.exists() {
        return Ok(None);
    }

    let path = absolute_config_path(&path)
        .with_context(|| format!("failed to resolve config path `{}`", path.display()))?;
    let data = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config `{}`", path.display()))?;
    let config = match serde_json::from_str::<FileConfig>(&data) {
        Ok(config) => config,
        Err(error)
            if matches!(
                error.classify(),
                serde_json::error::Category::Syntax | serde_json::error::Category::Eof
            ) =>
        {
            bail!("{}", config_syntax_error(&path, &data, &error));
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to parse config `{}`", path.display()));
        }
    };
    let config_dir = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    Ok(Some((config, config_dir, path)))
}

fn absolute_config_path(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .context("failed to resolve current directory")?
            .join(path)
    };
    Ok(clean_lexical_path(&path))
}

fn clean_lexical_path(path: &Path) -> PathBuf {
    let mut clean = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => clean.push(prefix.as_os_str()),
            Component::RootDir => clean.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !clean.pop() && !clean.has_root() {
                    clean.push("..");
                }
            }
            Component::Normal(value) => clean.push(value),
        }
    }
    clean
}

pub(super) fn read_package_json_config() -> Result<Option<(FileConfig, PathBuf, PathBuf)>> {
    let path = std::env::current_dir()?.join("package.json");
    if !path.exists() {
        return Ok(None);
    }

    let data = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("Warning: Could not read {}: {error}", path.display());
            return Ok(None);
        }
    };
    let package = match serde_json::from_str::<PackageJson>(&data) {
        Ok(package) => package,
        Err(error) => {
            if serde_json::from_str::<serde_json::Value>(&data).is_ok() {
                return Err(error).with_context(|| {
                    format!("failed to parse jscpd config in `{}`", path.display())
                });
            }
            eprintln!("Warning: Could not parse {}: {error}", path.display());
            return Ok(None);
        }
    };
    let Some(config) = package.jscpd else {
        return Ok(None);
    };
    let config_dir = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    Ok(Some((config, config_dir, path)))
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    jscpd: Option<FileConfig>,
}

pub(super) fn apply_config(
    options: &mut Options,
    config: FileConfig,
    config_dir: &Path,
) -> Result<()> {
    if let Some(execution_id) = config.execution_id {
        options.execution_id = Some(execution_id);
    }
    if let Some(paths) = config.path {
        options.paths = paths
            .into_vec()
            .into_iter()
            .map(|path| resolve_config_path(config_dir, path))
            .collect();
    }
    if let Some(pattern) = config.pattern {
        options.pattern = pattern;
    }
    if let Some(ignore) = config.ignore {
        options.ignore = ignore
            .into_vec()
            .into_iter()
            .map(|pattern| resolve_config_ignore(config_dir, pattern))
            .collect::<Result<Vec<_>>>()?;
    }
    if let Some(reporters) = config.reporters {
        options.reporters = reporters.into_vec();
    }
    if let Some(listeners) = config.listeners {
        options.listeners = listeners.into_vec();
    }
    if let Some(reporters_options) = config.reporters_options {
        options.reporters_options = reporters_options;
    }
    if let Some(output) = config.output {
        options.output = output;
    }
    if let Some(format) = config.format {
        let formats = format.into_vec();
        options.formats = Some(formats.iter().cloned().collect());
        options.format_order = Some(formats);
    }
    if let Some(formats_exts) = config.formats_exts {
        options.formats_exts = formats_exts.into_mappings();
    }
    if let Some(formats_names) = config.formats_names {
        options.formats_names = formats_names.into_mappings();
    }
    if let Some(ignore_pattern) = config.ignore_pattern {
        options.ignore_pattern = compile_patterns(ignore_pattern.into_vec())
            .context("invalid ignorePattern in config")?;
    }
    if let Some(min_lines) = config.min_lines {
        options.min_lines = min_lines;
    }
    if let Some(min_tokens) = config.min_tokens {
        options.min_tokens = min_tokens;
    }
    if let Some(max_lines) = config.max_lines {
        options.max_lines = max_lines;
    }
    if let Some(max_size) = config.max_size {
        options.max_size_bytes = parse_size(&max_size)
            .with_context(|| format!("invalid maxSize value `{max_size}` in config"))?;
    }
    if let Some(threshold) = config.threshold {
        options.threshold = Some(threshold);
    }
    if let Some(mode) = config.mode {
        options.mode = super::parse_mode(&mode)?;
    }
    if let Some(store) = config.store {
        options.store = Some(store);
    }
    if let Some(store_path) = config.store_path {
        options.store_path = Some(store_path);
    }
    if let Some(blame) = config.blame {
        options.blame = blame;
    }
    if let Some(cache) = config.cache {
        options.cache = cache;
    }
    if let Some(silent) = config.silent {
        options.silent = silent;
    }
    if let Some(absolute) = config.absolute {
        options.absolute = absolute;
    }
    if let Some(no_symlinks) = config.no_symlinks {
        options.no_symlinks = no_symlinks;
    }
    if let Some(ignore_case) = config.ignore_case {
        options.ignore_case = ignore_case;
    }
    if let Some(gitignore) = config.gitignore {
        options.gitignore = gitignore;
    }
    if let Some(debug) = config.debug {
        options.debug = debug;
    }
    if let Some(verbose) = config.verbose {
        options.verbose = verbose;
    }
    if let Some(skip_local) = config.skip_local {
        options.skip_local = skip_local;
    }
    if let Some(exit_code) = config.exit_code {
        options.exit_code = exit_code.into();
    }
    if let Some(no_tips) = config.no_tips {
        options.no_tips = no_tips;
    }
    if let Some(tokens_to_skip) = config.tokens_to_skip {
        options.tokens_to_skip = tokens_to_skip.into_vec();
    }
    Ok(())
}

fn resolve_config_path<T: Into<PathBuf>>(config_dir: &Path, path: T) -> PathBuf {
    let path = path.into();
    if path.is_absolute() {
        path
    } else {
        config_dir.join(path)
    }
}

fn config_syntax_error(path: &Path, data: &str, error: &serde_json::Error) -> String {
    format!(
        "SyntaxError: {}: {}",
        path.display(),
        node_like_json_syntax_message(data, error)
    )
}

fn node_like_json_syntax_message(data: &str, error: &serde_json::Error) -> String {
    let line = error.line();
    let column = error.column();
    let position = json_error_position(data, line, column);
    let message = error.to_string();

    if message.starts_with("key must be a string") {
        format!(
            "Expected property name or '}}' in JSON at position {position} (line {line} column {column})"
        )
    } else if matches!(error.classify(), serde_json::error::Category::Eof) {
        "Unexpected end of JSON input".to_string()
    } else {
        format!("{message} at position {position} (line {line} column {column})")
    }
}

fn json_error_position(data: &str, line: usize, column: usize) -> usize {
    let before_line = data
        .lines()
        .take(line.saturating_sub(1))
        .map(|line| line.len() + 1)
        .sum::<usize>();
    before_line + column.saturating_sub(1)
}

pub(super) fn resolve_config_ignore(config_dir: &Path, pattern: String) -> Result<String> {
    let path = Path::new(&pattern);
    if path.is_absolute() || pattern.starts_with("**/") {
        return Ok(pattern);
    }

    let absolute = config_dir.join(&pattern);
    let cwd = std::env::current_dir().context("failed to resolve current directory")?;
    if let Ok(relative) = absolute.strip_prefix(cwd) {
        return Ok(relative.display().to_string());
    }

    Ok(absolute.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn malformed_config_json_uses_upstream_style_syntax_error() {
        let path = Path::new("/tmp/project/.jscpd.json");
        let data = "{ invalid json\n";
        let error = serde_json::from_str::<FileConfig>(data).unwrap_err();

        assert_eq!(
            config_syntax_error(path, data, &error),
            "SyntaxError: /tmp/project/.jscpd.json: Expected property name or '}' in JSON at position 2 (line 1 column 3)"
        );
    }

    #[test]
    fn config_number_strings_follow_node_number_rules() {
        assert_eq!(parse_config_usize_string("0x10").unwrap(), 16);
        assert_eq!(parse_config_usize_string(" 42 ").unwrap(), 42);
        assert!(parse_config_usize_string("1.5").is_err());
        assert!(parse_config_usize_string("-1").is_err());
        assert!(parse_config_usize_string("Infinity").is_err());

        let config: FileConfig =
            serde_json::from_str(r#"{"minLines":"0b101","threshold":"1e2"}"#).unwrap();
        assert_eq!(config.min_lines, Some(5));
        assert_eq!(config.threshold, Some(100.0));

        let error = serde_json::from_str::<FileConfig>(r#"{"minLines":[]}"#).unwrap_err();
        assert!(error.to_string().contains("invalid type: array"));
    }

    #[test]
    fn config_number_deserializers_accept_numbers_nulls_and_reject_wrong_types() {
        let config: FileConfig =
            serde_json::from_str(r#"{"minLines":7,"maxLines":null,"threshold":2.5}"#).unwrap();
        assert_eq!(config.min_lines, Some(7));
        assert_eq!(config.max_lines, None);
        assert_eq!(config.threshold, Some(2.5));

        let config: FileConfig =
            serde_json::from_str(r#"{"minLines":null,"threshold":null}"#).unwrap();
        assert_eq!(config.min_lines, None);
        assert_eq!(config.threshold, None);

        let error = serde_json::from_str::<FileConfig>(r#"{"threshold":{}}"#).unwrap_err();
        assert!(error.to_string().contains("invalid type: object"));
    }

    #[test]
    fn clean_lexical_path_collapses_dot_and_parent_components() {
        assert_eq!(
            clean_lexical_path(Path::new("repo/app/./src/../tests/file.js")),
            PathBuf::from("repo/app/tests/file.js")
        );
        assert_eq!(
            clean_lexical_path(Path::new("../repo/./src")),
            PathBuf::from("../repo/src")
        );
        assert!(
            absolute_config_path(Path::new("Cargo.toml"))
                .unwrap()
                .is_absolute()
        );
    }

    #[test]
    fn resolves_config_paths_and_ignores_like_upstream() {
        let cwd = std::env::current_dir().unwrap();
        let config_dir = cwd.join("fixtures").join("config");
        let absolute = cwd.join("src");

        assert_eq!(
            resolve_config_path(&config_dir, "src"),
            config_dir.join("src")
        );
        assert_eq!(resolve_config_path(&config_dir, absolute.clone()), absolute);
        assert_eq!(
            resolve_config_ignore(&config_dir, "**/vendor/**".to_string()).unwrap(),
            "**/vendor/**"
        );
        assert_eq!(
            resolve_config_ignore(&cwd, "target/**".to_string()).unwrap(),
            "target/**"
        );

        let absolute = cwd.join("dist/**").display().to_string();
        assert_eq!(
            resolve_config_ignore(&config_dir, absolute.clone()).unwrap(),
            absolute
        );
    }

    #[test]
    fn applies_config_mappings_and_exit_code_variants() {
        let config: FileConfig = serde_json::from_str(
            r#"{
                "format": "javascript,typescript",
                "formatsExts": {"javascript": ["mjs", "cjs"]},
                "formatsNames": "makefile:Makefile,GNUmakefile",
                "exitCode": false,
                "tokensToSkip": "import,require"
            }"#,
        )
        .unwrap();
        let mut options = Options::default();

        apply_config(&mut options, config, Path::new(".")).unwrap();

        assert_eq!(
            options.format_order,
            Some(vec!["javascript".to_string(), "typescript".to_string()])
        );
        assert_eq!(
            options.formats_exts.find_format_for_value("mjs"),
            Some("javascript")
        );
        assert_eq!(
            options.formats_names.find_format_for_value("GNUmakefile"),
            Some("makefile")
        );
        assert_eq!(options.exit_code, ExitCode::Boolean(false));
        assert_eq!(options.tokens_to_skip, vec!["import", "require"]);
    }

    #[test]
    fn node_like_json_syntax_message_handles_eof() {
        let data = "{\"minLines\":";
        let error = serde_json::from_str::<FileConfig>(data).unwrap_err();

        assert_eq!(
            node_like_json_syntax_message(data, &error),
            "Unexpected end of JSON input"
        );
        assert_eq!(json_error_position("a\nbc", 2, 2), 3);

        let trailing = "[1,]";
        let error = serde_json::from_str::<serde_json::Value>(trailing).unwrap_err();
        assert!(node_like_json_syntax_message(trailing, &error).contains("at position"));
    }

    #[test]
    fn read_config_reports_syntax_errors_with_upstream_shape() {
        let root = unique_temp_dir("jscpd-rs-config-syntax");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join(".jscpd.json");
        std::fs::write(&path, "{ invalid json\n").unwrap();

        let error = read_config(Some(&path)).unwrap_err();
        let _ = std::fs::remove_dir_all(root);

        assert!(error.to_string().starts_with("SyntaxError: "));
        assert!(error.to_string().contains("Expected property name or '}'"));
    }

    #[test]
    fn apply_config_updates_main_runtime_flags() {
        let config: FileConfig = serde_json::from_str(
            r#"{
                "executionId": "run-1",
                "path": "src,tests",
                "pattern": "**/*.rs",
                "ignore": "target/**,dist/**",
                "reporters": "json,console",
                "ignorePattern": ["import .*"],
                "minLines": 3,
                "minTokens": 7,
                "maxLines": 99,
                "maxSize": "2kb",
                "threshold": 10,
                "mode": "weak",
                "silent": true,
                "absolute": true,
                "noSymlinks": true,
                "ignoreCase": true,
                "gitignore": false,
                "debug": true,
                "verbose": true,
                "skipLocal": true,
                "exitCode": 2,
                "noTips": true
            }"#,
        )
        .unwrap();
        let mut options = Options::default();

        apply_config(&mut options, config, Path::new(".")).unwrap();

        assert_eq!(options.execution_id.as_deref(), Some("run-1"));
        assert_eq!(
            options.paths,
            vec![PathBuf::from("./src"), PathBuf::from("./tests")]
        );
        assert_eq!(options.pattern, "**/*.rs");
        assert_eq!(options.ignore, vec!["./target/**", "./dist/**"]);
        assert_eq!(options.reporters, vec!["json", "console"]);
        assert_eq!(options.ignore_pattern.len(), 1);
        assert_eq!(options.min_lines, 3);
        assert_eq!(options.min_tokens, 7);
        assert_eq!(options.max_lines, 99);
        assert_eq!(options.max_size_bytes, 2 * 1024);
        assert_eq!(options.threshold, Some(10.0));
        assert_eq!(options.mode, super::super::Mode::Weak);
        assert!(options.silent);
        assert!(options.absolute);
        assert!(options.no_symlinks);
        assert!(options.ignore_case);
        assert!(!options.gitignore);
        assert!(options.debug);
        assert!(options.verbose);
        assert!(options.skip_local);
        assert_eq!(options.exit_code, ExitCode::Number(2.0));
        assert!(options.no_tips);
    }

    #[cfg(unix)]
    #[test]
    fn read_config_preserves_symlink_path_like_upstream() {
        let root = unique_temp_dir("jscpd-rs-config-symlink");
        let real_dir = root.join("real");
        let link_dir = root.join("link");
        std::fs::create_dir_all(&real_dir).unwrap();
        std::fs::create_dir_all(link_dir.join("src")).unwrap();
        std::fs::write(
            real_dir.join(".jscpd.json"),
            r#"{"path":["src"],"ignore":["ignored/**"]}"#,
        )
        .unwrap();
        std::os::unix::fs::symlink("../real/.jscpd.json", link_dir.join(".jscpd.json")).unwrap();

        let link_config = link_dir.join(".jscpd.json");
        let (config, config_dir, config_path) = read_config(Some(&link_config)).unwrap().unwrap();

        assert_eq!(config_path, link_config);
        assert_eq!(config_dir, link_dir);

        let mut options = Options::default();
        apply_config(&mut options, config, &config_dir).unwrap();
        assert_eq!(options.paths, vec![root.join("link/src")]);
        assert_eq!(
            options.ignore,
            vec![root.join("link/ignored/**").display().to_string()]
        );

        let _ = std::fs::remove_dir_all(root);
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{}-{suffix}", std::process::id()))
    }
}
