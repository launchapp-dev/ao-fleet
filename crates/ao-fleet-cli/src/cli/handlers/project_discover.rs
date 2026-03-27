use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use ao_fleet_core::{NewProject, Project};
use ao_fleet_store::FleetStore;
use serde::Serialize;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_discover_command::ProjectDiscoverCommand;

pub fn project_discover(db_path: &str, command: ProjectDiscoverCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let search_roots = resolve_search_roots(command.search_roots)?;
    let discovered = discover_all_projects(&search_roots, command.max_depth)?;
    let registered_projects = store.list_projects(None)?;
    let registered_lookup = build_registered_lookup(&registered_projects);
    let mut used_slugs: BTreeSet<String> =
        registered_projects.into_iter().map(|project| project.slug).collect();

    if command.register && command.team_id.is_none() {
        bail!("--team-id is required when --register is set");
    }

    let mut results = Vec::new();
    let mut registered_count = 0_usize;

    for candidate in discovered {
        let existing = registered_lookup
            .get(&candidate.normalized_root_path)
            .or_else(|| registered_lookup.get(&candidate.normalized_ao_project_root))
            .cloned();

        let existing_project_id = existing.as_ref().map(|project| project.id.clone());
        let existing_team_id = existing.as_ref().map(|project| project.team_id.clone());
        let mut registered_project_id = existing_project_id.clone();

        if command.register && existing.is_none() {
            let project = register_discovered_project(
                &store,
                command.team_id.as_deref().expect("team_id validated"),
                &candidate,
                &mut used_slugs,
            )?;
            registered_count += 1;
            registered_project_id = Some(project.id);
        }

        results.push(DiscoveredProject {
            root_path: candidate.root_path,
            ao_project_root: candidate.ao_project_root,
            slug_hint: candidate.slug_hint,
            default_branch: candidate.default_branch,
            has_git: candidate.has_git,
            has_ao: candidate.has_ao,
            existing_project_id,
            existing_team_id,
            registered_project_id,
        });
    }

    print_json(&ProjectDiscoverResult {
        search_roots: search_roots
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
        max_depth: command.max_depth,
        register: command.register,
        team_id: command.team_id,
        discovered_count: results.len(),
        registered_count,
        projects: results,
    })
}

#[derive(Debug, Serialize)]
struct ProjectDiscoverResult {
    search_roots: Vec<String>,
    max_depth: usize,
    register: bool,
    team_id: Option<String>,
    discovered_count: usize,
    registered_count: usize,
    projects: Vec<DiscoveredProject>,
}

#[derive(Debug, Serialize)]
struct DiscoveredProject {
    root_path: String,
    ao_project_root: String,
    slug_hint: String,
    default_branch: String,
    has_git: bool,
    has_ao: bool,
    existing_project_id: Option<String>,
    existing_team_id: Option<String>,
    registered_project_id: Option<String>,
}

#[derive(Debug, Clone)]
struct DiscoveryCandidate {
    root_path: String,
    ao_project_root: String,
    normalized_root_path: String,
    normalized_ao_project_root: String,
    slug_hint: String,
    default_branch: String,
    has_git: bool,
    has_ao: bool,
}

fn register_discovered_project(
    store: &FleetStore,
    team_id: &str,
    candidate: &DiscoveryCandidate,
    used_slugs: &mut BTreeSet<String>,
) -> Result<Project> {
    let slug = allocate_unique_slug(slugify(&candidate.slug_hint), used_slugs);
    store
        .create_project(NewProject {
            team_id: team_id.to_string(),
            slug,
            root_path: candidate.root_path.clone(),
            ao_project_root: candidate.ao_project_root.clone(),
            default_branch: candidate.default_branch.clone(),
            enabled: candidate.has_ao,
        })
        .map_err(Into::into)
}

fn resolve_search_roots(search_roots: Vec<String>) -> Result<Vec<PathBuf>> {
    if !search_roots.is_empty() {
        return search_roots
            .into_iter()
            .map(|root| normalize_existing_path(Path::new(&root)))
            .collect();
    }

    let cwd = std::env::current_dir().context("failed to determine current directory")?;
    Ok(vec![cwd])
}

fn discover_all_projects(
    search_roots: &[PathBuf],
    max_depth: usize,
) -> Result<Vec<DiscoveryCandidate>> {
    let mut discovered = Vec::new();
    let mut seen = BTreeSet::new();

    for root in search_roots {
        for candidate in discover_projects_under_root(root, max_depth)? {
            if seen.insert(candidate.normalized_root_path.clone()) {
                discovered.push(candidate);
            }
        }
    }

    discovered.sort_by(|left, right| left.root_path.cmp(&right.root_path));
    Ok(discovered)
}

fn discover_projects_under_root(root: &Path, max_depth: usize) -> Result<Vec<DiscoveryCandidate>> {
    let mut discovered = Vec::new();
    let mut queue = VecDeque::from([(root.to_path_buf(), 0_usize)]);
    let mut visited = BTreeSet::new();

    while let Some((path, depth)) = queue.pop_front() {
        let normalized = normalize_existing_path(&path)?;
        let normalized_string = normalized.to_string_lossy().to_string();
        if !visited.insert(normalized_string) {
            continue;
        }

        if let Some(candidate) = classify_project_root(&normalized)? {
            discovered.push(candidate);
            continue;
        }

        if depth >= max_depth {
            continue;
        }

        for child in read_subdirectories(&normalized)? {
            if should_skip_directory(&child) {
                continue;
            }
            queue.push_back((child, depth + 1));
        }
    }

    Ok(discovered)
}

fn classify_project_root(path: &Path) -> Result<Option<DiscoveryCandidate>> {
    let git_marker = path.join(".git");
    let ao_marker = path.join(".ao");
    let has_git = git_marker.exists();
    let has_ao = ao_marker.is_dir();

    if !(has_git || has_ao) {
        return Ok(None);
    }

    let normalized_root = normalize_existing_path(path)?;
    let normalized_root_string = normalized_root.to_string_lossy().to_string();
    Ok(Some(DiscoveryCandidate {
        root_path: normalized_root_string.clone(),
        ao_project_root: normalized_root_string.clone(),
        normalized_root_path: normalized_root_string.clone(),
        normalized_ao_project_root: normalized_root_string,
        slug_hint: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("project")
            .to_string(),
        default_branch: discover_default_branch(path).unwrap_or_else(|| "main".to_string()),
        has_git,
        has_ao,
    }))
}

fn discover_default_branch(path: &Path) -> Option<String> {
    let git_dir = resolve_git_dir(path)?;
    let head = fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let reference = head.trim().strip_prefix("ref: ")?;
    Some(reference.rsplit('/').next()?.to_string())
}

fn resolve_git_dir(path: &Path) -> Option<PathBuf> {
    let git_marker = path.join(".git");
    if git_marker.is_dir() {
        return Some(git_marker);
    }

    let git_file = fs::read_to_string(git_marker).ok()?;
    let raw_path = git_file.trim().strip_prefix("gitdir: ")?;
    let git_path = Path::new(raw_path);
    Some(if git_path.is_absolute() { git_path.to_path_buf() } else { path.join(git_path) })
}

fn read_subdirectories(path: &Path) -> Result<Vec<PathBuf>> {
    let mut directories = Vec::new();
    for entry in fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            directories.push(entry.path());
        }
    }
    Ok(directories)
}

fn should_skip_directory(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };

    matches!(
        name,
        ".git"
            | ".ao"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | ".next"
            | ".turbo"
            | ".venv"
            | "venv"
            | "__pycache__"
    )
}

fn normalize_existing_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path)
            .with_context(|| format!("failed to canonicalize {}", path.display()));
    }

    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn build_registered_lookup(projects: &[Project]) -> BTreeMap<String, Project> {
    let mut lookup = BTreeMap::new();
    for project in projects {
        lookup.insert(normalize_registered_path(&project.root_path), project.clone());
        lookup.insert(normalize_registered_path(&project.ao_project_root), project.clone());
    }
    lookup
}

fn normalize_registered_path(path: &str) -> String {
    let path = Path::new(path);
    normalize_existing_path(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }

    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() { "project".to_string() } else { trimmed }
}

fn allocate_unique_slug(base_slug: String, used_slugs: &mut BTreeSet<String>) -> String {
    if used_slugs.insert(base_slug.clone()) {
        return base_slug;
    }

    for index in 2.. {
        let candidate = format!("{base_slug}-{index}");
        if used_slugs.insert(candidate.clone()) {
            return candidate;
        }
    }

    unreachable!("slug allocation loop should always return")
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, write};

    use super::*;

    #[test]
    fn discover_projects_finds_git_and_ao_roots() {
        let temp = tempfile::tempdir().expect("temp dir should exist");
        let ao_root = temp.path().join("ao-app");
        let git_root = temp.path().join("git-only");

        create_dir_all(ao_root.join(".ao")).expect("ao dir should exist");
        create_dir_all(ao_root.join(".git")).expect("git dir should exist");
        write(ao_root.join(".git/HEAD"), "ref: refs/heads/main\n").expect("head should exist");
        create_dir_all(git_root.join(".git")).expect("git dir should exist");
        write(git_root.join(".git/HEAD"), "ref: refs/heads/trunk\n").expect("head should exist");

        let discovered =
            discover_projects_under_root(temp.path(), 4).expect("discovery should succeed");

        assert_eq!(discovered.len(), 2);
        assert!(
            discovered.iter().any(|project| project.has_ao && project.default_branch == "main")
        );
        assert!(
            discovered.iter().any(|project| !project.has_ao && project.default_branch == "trunk")
        );
    }

    #[test]
    fn allocate_unique_slug_appends_suffix() {
        let mut used = BTreeSet::from(["app".to_string(), "app-2".to_string()]);

        let slug = allocate_unique_slug("app".to_string(), &mut used);

        assert_eq!(slug, "app-3");
    }
}
