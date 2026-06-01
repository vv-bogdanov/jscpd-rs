use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::cli::Options;

use super::SourceFile;
use super::discover;
use super::discovery::{
    build_glob_set, build_ignore_matcher, count_lines, decode_source, format_filter_skip_message,
    is_ignored, normalize_glob_path,
};
use super::gitignore::{
    collect_cwd_gitignore_patterns, collect_gitignore_patterns_with_global, gitignore_line_to_globs,
};
use super::paths::{display_relative_to, fast_glob_like_path_cmp, relative_path};
use super::test_support::unique_temp_path;

fn discovery_options(paths: Vec<PathBuf>) -> Options {
    Options {
        paths,
        min_lines: 1,
        reporters: vec!["json".to_string()],
        silent: true,
        gitignore: false,
        ..Options::default()
    }
}

fn javascript_discovery_options(paths: Vec<PathBuf>) -> Options {
    let mut options = discovery_options(paths);
    options.formats = Some(HashSet::from(["javascript".to_string()]));
    options
}

fn source_ids(files: &[SourceFile]) -> Vec<&str> {
    files.iter().map(|file| file.source_id.as_str()).collect()
}

#[test]
fn fast_glob_like_order_places_parent_files_before_child_files() {
    assert_eq!(
        fast_glob_like_path_cmp(
            Path::new("pkg/tokenizer/src/tokenize.ts"),
            Path::new("pkg/tokenizer/src/languages/markdown-tokenizer.ts"),
        ),
        Ordering::Less
    );
    assert_eq!(
        fast_glob_like_path_cmp(
            Path::new("pkg/tokenizer/src/languages/astro.ts"),
            Path::new("pkg/tokenizer/src/languages/vue.ts"),
        ),
        Ordering::Less
    );
    assert_eq!(
        fast_glob_like_path_cmp(
            Path::new("../example-app/landing/.next/types/validator.ts"),
            Path::new("../example-app/landing/.next/dev/types/validator.ts"),
        ),
        Ordering::Less
    );
}

#[test]
fn explicit_file_paths_preserve_cli_order_like_upstream() {
    let dir = unique_temp_path("explicit-order");
    let setup = dir.join("fixtures").join("setupTests.js");
    let utils = dir
        .join("packages")
        .join("react-devtools-shared")
        .join("utils.js");
    let console_mock = dir
        .join("packages")
        .join("internal-test-utils")
        .join("consoleMock.js");
    std::fs::create_dir_all(setup.parent().unwrap()).unwrap();
    std::fs::create_dir_all(utils.parent().unwrap()).unwrap();
    std::fs::create_dir_all(console_mock.parent().unwrap()).unwrap();
    std::fs::write(&setup, "const setup = 1;\n").unwrap();
    std::fs::write(&utils, "const utils = 1;\n").unwrap();
    std::fs::write(&console_mock, "const consoleMock = 1;\n").unwrap();

    let options =
        javascript_discovery_options(vec![setup.clone(), utils.clone(), console_mock.clone()]);

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let paths = source_ids(&files);

    assert_eq!(paths.len(), 3);
    assert!(paths[0].ends_with("fixtures/setupTests.js"));
    assert!(paths[1].ends_with("packages/react-devtools-shared/utils.js"));
    assert!(paths[2].ends_with("packages/internal-test-utils/consoleMock.js"));
}

#[test]
fn directory_discovery_preserves_glob_like_order_with_parallel_walk() {
    let dir = unique_temp_path("parallel-order");
    std::fs::create_dir_all(dir.join("packages/a")).unwrap();
    std::fs::create_dir_all(dir.join("packages/b")).unwrap();
    std::fs::write(dir.join("packages/root.js"), "const root = 1;\n").unwrap();
    std::fs::write(dir.join("packages/a/file.js"), "const a = 1;\n").unwrap();
    std::fs::write(dir.join("packages/b/file.js"), "const b = 1;\n").unwrap();

    let options = javascript_discovery_options(vec![dir.clone()]);

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let paths = source_ids(&files);

    assert_eq!(paths.len(), 3);
    assert!(paths[0].ends_with("packages/root.js"));
    assert!(paths[1].ends_with("packages/a/file.js"));
    assert!(paths[2].ends_with("packages/b/file.js"));
}

#[test]
fn relative_path_formats_sibling_paths_like_upstream() {
    assert_eq!(
        relative_path(
            Path::new("/workspace/example-app/file.ts"),
            Path::new("/workspace/jscpd-rs")
        )
        .unwrap(),
        Path::new("../example-app/file.ts")
    );
}

#[test]
fn gitignore_line_to_globs_anchors_rooted_patterns_to_base_dir() {
    let globs = gitignore_line_to_globs("/node_modules/", Some(Path::new("/repo/app")));
    assert!(globs.iter().any(|glob| glob == "/repo/app/node_modules"));
    assert!(globs.iter().any(|glob| glob == "/repo/app/node_modules/**"));
}

#[test]
fn gitignore_line_to_globs_preserves_negations_like_upstream() {
    let globs = gitignore_line_to_globs("!ignored/keep.js", Some(Path::new("/repo/app")));

    assert!(
        globs
            .iter()
            .any(|glob| glob == "!/repo/app/ignored/keep.js")
    );
    assert!(
        globs
            .iter()
            .any(|glob| glob == "!/repo/app/ignored/keep.js/**")
    );
}

#[test]
fn gitignore_line_to_globs_matches_upstream_conversion_without_base_dir() {
    assert_eq!(
        gitignore_line_to_globs("/node_modules", None),
        vec!["node_modules", "node_modules/**"]
    );
    assert_eq!(
        gitignore_line_to_globs("src/dist", None),
        vec!["src/dist", "src/dist/**", "**/src/dist", "**/src/dist/**"]
    );
    assert_eq!(
        gitignore_line_to_globs("**/dist", None),
        vec!["**/dist", "**/dist/**"]
    );
    assert_eq!(
        gitignore_line_to_globs("!test.js", None),
        vec!["!**/test.js", "!**/test.js/**"]
    );
    assert!(gitignore_line_to_globs("# ignored", None).is_empty());
    assert!(gitignore_line_to_globs("  ", None).is_empty());
}

#[test]
fn gitignore_line_to_globs_keeps_upstream_variants_for_cwd_base_dir() {
    let cwd = std::env::current_dir().unwrap();

    let globs = gitignore_line_to_globs("src/dist", Some(&cwd));
    assert!(globs.iter().any(|glob| glob == "src/dist"));
    assert!(globs.iter().any(|glob| glob == "src/dist/**"));
    assert!(globs.iter().any(|glob| glob == "**/src/dist"));
    assert!(globs.iter().any(|glob| glob == "**/src/dist/**"));

    let negated = gitignore_line_to_globs("!test.js", Some(&cwd));
    assert!(negated.iter().any(|glob| glob == "!**/test.js"));
    assert!(negated.iter().any(|glob| glob == "!**/test.js/**"));
}

#[test]
fn collect_gitignore_patterns_includes_global_excludes_like_upstream() {
    let dir = unique_temp_path("global-excludes");
    std::fs::create_dir_all(&dir).unwrap();
    let global_excludes = dir.join("globalignore");
    std::fs::write(&global_excludes, "*.swp\n.DS_Store\n# comment\n\n").unwrap();

    let patterns =
        collect_gitignore_patterns_with_global(std::slice::from_ref(&dir), Some(&global_excludes));
    let _ = std::fs::remove_dir_all(&dir);

    assert!(patterns.iter().any(|pattern| pattern == "**/*.swp"));
    assert!(patterns.iter().any(|pattern| pattern == "**/*.swp/**"));
    assert!(patterns.iter().any(|pattern| pattern == "**/.DS_Store"));
    assert!(patterns.iter().all(|pattern| !pattern.contains("comment")));
}

#[test]
fn collect_cwd_gitignore_patterns_uses_upstream_unscoped_conversion() {
    let dir = unique_temp_path("cwd-gitignore");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(".gitignore"), "/target/\nreport\n# comment\n\n").unwrap();

    let patterns = collect_cwd_gitignore_patterns(&dir);
    let _ = std::fs::remove_dir_all(&dir);

    assert!(patterns.iter().any(|pattern| pattern == "target"));
    assert!(patterns.iter().any(|pattern| pattern == "target/**"));
    assert!(patterns.iter().any(|pattern| pattern == "**/report"));
    assert!(patterns.iter().all(|pattern| !pattern.contains("comment")));
}

#[test]
fn collect_cwd_gitignore_patterns_returns_empty_without_gitignore() {
    let dir = unique_temp_path("missing-cwd-gitignore");
    std::fs::create_dir_all(&dir).unwrap();

    let patterns = collect_cwd_gitignore_patterns(&dir);
    let _ = std::fs::remove_dir_all(&dir);

    assert!(patterns.is_empty());
}

#[test]
fn format_filter_skip_message_matches_upstream_shape() {
    let cwd = Path::new("/repo");
    let path = Path::new("/repo/src/file.ts");

    assert_eq!(
        format_filter_skip_message(path, "typescript", cwd),
        "File src/file.ts skipped! Format \"typescript\" does not included to supported formats."
    );
}

#[test]
fn empty_glob_set_matches_nothing() {
    let glob_set = build_glob_set(&[]).unwrap();

    assert!(!glob_set.is_match("src/main.js"));
}

#[test]
fn normalize_glob_path_removes_dot_and_parent_components() {
    assert_eq!(
        normalize_glob_path(PathBuf::from("./src/../dist/file.js")),
        "dist/file.js"
    );
}

#[test]
fn decode_source_reuses_valid_utf8_and_falls_back_to_lossy() {
    assert_eq!(
        decode_source(b"const answer = 42;\n".to_vec()),
        "const answer = 42;\n"
    );
    assert_eq!(decode_source(vec![b'a', 0xff, b'b']), "a\u{fffd}b");
}

#[test]
fn debug_discovery_uses_sequential_walk_and_pattern_filter() {
    let dir = unique_temp_path("debug-sequential");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.js"), "const main = 1;\n").unwrap();
    std::fs::write(dir.join("style.css"), "body { color: red; }\n").unwrap();

    let mut options = discovery_options(vec![dir.clone()]);
    options.debug = true;
    options.pattern = "**/*.js".to_string();

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(files.len(), 1);
    assert!(files[0].source_id.ends_with("main.js"));
}

#[test]
fn absolute_discovery_uses_canonical_source_id() {
    let dir = unique_temp_path("absolute-source-id");
    let path = dir.join("main.js");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(&path, "const main = 1;\n").unwrap();

    let mut options = discovery_options(vec![path.clone()]);
    options.absolute = true;
    options.reporters = Vec::new();

    let files = discover(&options).unwrap();
    let expected = path.canonicalize().unwrap().display().to_string();
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].source_id, expected);
}

#[test]
fn discovery_filters_line_count_bounds() {
    let dir = unique_temp_path("line-filter");
    let path = dir.join("short.js");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(&path, "const short = 1;\n").unwrap();

    let mut options = discovery_options(vec![path]);
    options.min_lines = 3;
    options.reporters = Vec::new();

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert!(files.is_empty());
}

#[test]
fn discovery_skips_formats_outside_filter() {
    let dir = unique_temp_path("format-filter");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.ts"), "const value: number = 1;\n").unwrap();

    let mut options = javascript_discovery_options(vec![dir.clone()]);
    options.debug = true;

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert!(files.is_empty());
}

#[test]
fn ignore_matcher_checks_paths_relative_to_cwd() {
    let cwd = Path::new("/repo");
    let matcher = build_ignore_matcher(&["src/generated/**".to_string()]).unwrap();

    assert!(is_ignored(
        Path::new("/repo/src/generated/file.js"),
        &matcher,
        cwd
    ));
}

#[test]
fn count_lines_matches_upstream_empty_and_newline_rules() {
    assert_eq!(count_lines(b""), 1);
    assert_eq!(count_lines(b"one"), 1);
    assert_eq!(count_lines(b"one\n"), 2);
    assert_eq!(count_lines(b"one\ntwo"), 2);
}

#[cfg(unix)]
#[test]
fn discovers_executable_node_shebang_without_extension() {
    use super::test_support::make_executable;

    let path = unique_temp_path("node-shebang");
    std::fs::write(&path, "#!/usr/bin/env node\nconsole.log(1);\n").unwrap();
    make_executable(&path);

    let options = javascript_discovery_options(vec![path.clone()]);

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_file(&path);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].format, "javascript");
}

#[test]
fn discovers_common_non_native_formats() {
    let dir = unique_temp_path("formats");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("style.css"), "body { color: red; }\n").unwrap();
    std::fs::write(dir.join("index.html"), "<main>hello</main>\n").unwrap();
    std::fs::write(dir.join("config.yaml"), "enabled: true\n").unwrap();
    std::fs::write(dir.join("settings.toml"), "enabled = true\n").unwrap();
    std::fs::write(dir.join("Component.vue"), "<template><div /></template>\n").unwrap();

    let options = discovery_options(vec![dir.clone()]);

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let formats = files
        .iter()
        .map(|file| file.format.as_str())
        .collect::<HashSet<_>>();

    assert!(formats.contains("css"));
    assert!(formats.contains("markup"));
    assert!(formats.contains("yaml"));
    assert!(formats.contains("toml"));
    assert!(formats.contains("vue"));
}

#[test]
fn discovers_custom_extension_mappings() {
    let dir = unique_temp_path("custom-exts");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("component.foo"), "const answer = 42;\n").unwrap();

    let mut options = discovery_options(vec![dir.clone()]);
    options.formats_exts = crate::cli::FormatMappings::from_pairs(vec![(
        "javascript".to_string(),
        vec!["foo".to_string()],
    )]);

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].format, "javascript");
}

#[test]
fn discovers_custom_extensionless_name_mappings() {
    let dir = unique_temp_path("custom-names");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Recipe"), "target:\n\tprintf ok\n").unwrap();

    let mut options = discovery_options(vec![dir.clone()]);
    options.formats_names = crate::cli::FormatMappings::from_pairs(vec![(
        "makefile".to_string(),
        vec!["Recipe".to_string()],
    )]);

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].format, "makefile");
}

#[test]
fn reporter_uses_report_paths_when_silent() {
    let dir = unique_temp_path("reporter-paths");
    let path = dir.join("file.js");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(&path, "const alpha = 1;\n").unwrap();

    let mut options = discovery_options(vec![path.clone()]);
    options.reporters = vec!["html".to_string()];
    let cwd = std::env::current_dir().unwrap();

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].source_id, display_relative_to(&path, &cwd));
}

#[test]
fn relative_ignore_pattern_matches_absolute_scan_root_like_upstream() {
    let dir = unique_temp_path("relative-ignore");
    std::fs::create_dir_all(dir.join("patches")).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("patches").join("patch.js"), "const patch = 1;\n").unwrap();
    std::fs::write(dir.join("src").join("main.js"), "const main = 1;\n").unwrap();

    let mut options = javascript_discovery_options(vec![dir.clone()]);
    options.ignore = vec!["patches/**".to_string()];

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let paths = source_ids(&files);

    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("src/main.js"));
}

#[test]
fn dot_relative_ignore_pattern_matches_absolute_scan_root_like_upstream() {
    let dir = unique_temp_path("dot-relative-ignore");
    std::fs::create_dir_all(dir.join("patches")).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("patches").join("patch.js"), "const patch = 1;\n").unwrap();
    std::fs::write(dir.join("src").join("main.js"), "const main = 1;\n").unwrap();

    let mut options = javascript_discovery_options(vec![dir.clone()]);
    options.ignore = vec!["./patches/**".to_string()];

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let paths = source_ids(&files);

    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("src/main.js"));
}

#[test]
fn relative_ignore_patterns_match_dot_relative_walk_paths() {
    let matcher = build_ignore_matcher(&[
        "jscpd/**".to_string(),
        "target/**".to_string(),
        ".git/**".to_string(),
    ])
    .unwrap();
    let cwd = std::env::current_dir().unwrap();

    assert!(is_ignored(Path::new("./jscpd/file.js"), &matcher, &cwd));
    assert!(is_ignored(Path::new("./target/debug/app"), &matcher, &cwd));
    assert!(is_ignored(Path::new("./.git/config"), &matcher, &cwd));
    assert!(!is_ignored(Path::new("./src/lib.rs"), &matcher, &cwd));
}

#[cfg(unix)]
#[test]
fn no_symlinks_skips_symlink_scan_directory_like_upstream() {
    let dir = unique_temp_path("no-symlink-dir");
    let real_dir = dir.join("real");
    let link_dir = dir.join("linkdir");
    std::fs::create_dir_all(&real_dir).unwrap();
    std::fs::write(real_dir.join("file.js"), "const linked = 1;\n").unwrap();
    std::os::unix::fs::symlink(&real_dir, &link_dir).unwrap();

    let mut options = javascript_discovery_options(vec![link_dir]);
    options.no_symlinks = true;

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert!(files.is_empty());
}

#[cfg(unix)]
#[test]
fn no_symlinks_skips_symlink_scan_file_like_upstream() {
    let dir = unique_temp_path("no-symlink-file");
    let real_file = dir.join("real.js");
    let link_file = dir.join("link.js");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(&real_file, "const linked = 1;\n").unwrap();
    std::os::unix::fs::symlink(&real_file, &link_file).unwrap();

    let mut options = javascript_discovery_options(vec![link_file]);
    options.no_symlinks = true;

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert!(files.is_empty());
}

#[test]
fn empty_file_counts_as_one_line_like_upstream() {
    let dir = unique_temp_path("empty-lines");
    let path = dir.join("empty.js");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(&path, "").unwrap();

    let mut options = discovery_options(vec![path.clone()]);
    options.max_lines = 1;
    options.reporters = Vec::new();

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].source_id, path.display().to_string());
}

#[test]
fn known_extension_files_over_max_size_are_filtered() {
    let dir = unique_temp_path("max-size-filter");
    let path = dir.join("large.js");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(&path, "const value = 'larger than the configured size';\n").unwrap();

    let mut options = discovery_options(vec![path]);
    options.max_size_bytes = 10;
    options.reporters = Vec::new();

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert!(files.is_empty());
}

#[test]
fn gitignore_negation_reincludes_files_during_compat_discovery() {
    let dir = unique_temp_path("gitignore-negation");
    let ignored = dir.join("ignored");
    std::fs::create_dir_all(&ignored).unwrap();
    std::fs::write(dir.join(".gitignore"), "ignored/**\n!ignored/keep.js\n").unwrap();
    std::fs::write(ignored.join("drop.js"), "const drop = 1;\n").unwrap();
    std::fs::write(ignored.join("keep.js"), "const keep = 1;\n").unwrap();

    let mut options = discovery_options(vec![dir.clone()]);
    options.gitignore = true;

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let paths = source_ids(&files);

    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("ignored/keep.js"));
}

#[test]
fn gitignore_broad_ignore_with_negated_filename_keeps_nested_file() {
    let dir = unique_temp_path("gitignore-issue-723");
    let nested = dir.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(dir.join(".gitignore"), "**/**/*\n!test.js\n").unwrap();
    std::fs::write(nested.join("drop.js"), "const drop = 1;\n").unwrap();
    std::fs::write(nested.join("test.js"), "const keep = 1;\n").unwrap();

    let mut options = discovery_options(vec![dir.clone()]);
    options.gitignore = true;

    let files = discover(&options).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let paths = source_ids(&files);

    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("nested/test.js"));
}
