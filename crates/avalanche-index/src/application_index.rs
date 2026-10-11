use std::collections::HashSet;

use avalanche_model::{Application, ApplicationId, Scope};

use crate::static_index::StaticIndex;

const SYS_PREFIX: &str = "options.gb.programs.system.";
const HOME_PREFIX: &str = "options.gb.home.programs.";
const ENABLE_SUFFIX: &str = ".enable";

pub fn build_application_index(static_idx: &StaticIndex) -> Vec<Application> {
    let mut seen: HashSet<(u8, String, String)> = HashSet::new();
    let mut apps: Vec<Application> = Vec::new();

    for (path, assignments) in &static_idx.assignments {
        let scope = if path.starts_with(SYS_PREFIX) {
            Scope::System
        } else if path.starts_with(HOME_PREFIX) {
            Scope::Home
        } else {
            continue;
        };
        let prefix = if scope == Scope::System {
            SYS_PREFIX
        } else {
            HOME_PREFIX
        };
        let Some(inner) = path
            .strip_prefix(prefix)
            .and_then(|r| r.strip_suffix(ENABLE_SUFFIX))
        else {
            continue;
        };
        if inner.is_empty() {
            continue;
        }

        let segments: Vec<&str> = inner.split('.').filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            continue;
        }
        let (category, name) = if segments.len() == 1 {
            (String::new(), segments[0].to_string())
        } else {
            (
                segments[..segments.len() - 1].join("."),
                segments[segments.len() - 1].to_string(),
            )
        };

        let scope_key = if scope == Scope::System { 0u8 } else { 1u8 };
        if !seen.insert((scope_key, category.clone(), name.clone())) {
            continue;
        }

        let module_path = assignments
            .first()
            .map(|a| a.location.file.clone())
            .unwrap_or_default();

        apps.push(Application {
            id: ApplicationId {
                scope,
                category,
                name,
            },
            module_path,
        });
    }

    apps.sort_by(|a, b| {
        let ka = (
            scope_key(&a.id.scope),
            a.id.category.as_str(),
            a.id.name.as_str(),
        );
        let kb = (
            scope_key(&b.id.scope),
            b.id.category.as_str(),
            b.id.name.as_str(),
        );
        ka.cmp(&kb)
    });
    apps
}

fn scope_key(scope: &Scope) -> u8 {
    if *scope == Scope::System {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use avalanche_source::SourceService;
    use std::path::Path;

    fn write(root: &Path, rel: &str, content: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    fn tmp(tag: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("avalanche-appidx-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn discovers_flat_system_and_namespaced_home_apps() {
        let dir = tmp("disc");
        write(
            &dir,
            "modules/programs/system/steam.nix",
            "{ options.gb.programs.system.steam.enable = lib.mkEnableOption \"Steam\"; }\n",
        );
        write(
            &dir,
            "modules/programs/home/web/firefox.nix",
            "{ options.gb.home.programs.web.firefox.enable = lib.mkEnableOption \"Firefox\"; }\n",
        );
        let source = SourceService::new();
        let idx = crate::static_index::build_static_index(&dir, &source).unwrap();
        let apps = build_application_index(&idx);

        assert_eq!(apps.len(), 2);
        let steam = apps.iter().find(|a| a.id.name == "steam").unwrap();
        assert_eq!(steam.id.scope, Scope::System);
        assert_eq!(steam.id.category, "");
        assert_eq!(steam.module_path, "modules/programs/system/steam.nix");
        assert_eq!(steam.id.option_path(), "gb.programs.system.steam");

        let ff = apps.iter().find(|a| a.id.name == "firefox").unwrap();
        assert_eq!(ff.id.scope, Scope::Home);
        assert_eq!(ff.id.category, "web");
        assert_eq!(ff.id.option_path(), "gb.home.programs.web.firefox");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ignores_apps_outside_application_namespace() {
        let dir = tmp("excl");
        write(
            &dir,
            "modules/programs/home/dev/editors/zed.nix",
            "{ options.gb.home.dev.editors.zed.enable = lib.mkEnableOption \"Zed\"; }\n",
        );
        let source = SourceService::new();
        let idx = crate::static_index::build_static_index(&dir, &source).unwrap();
        let apps = build_application_index(&idx);
        assert!(
            apps.is_empty(),
            "gb.home.dev.* is not an application namespace"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dedupes_multi_file_declaration() {
        let dir = tmp("dedup");
        write(
            &dir,
            "modules/programs/system/wine.nix",
            "{ options.gb.programs.system.wine.enable = lib.mkEnableOption \"Wine\"; }\n",
        );
        let source = SourceService::new();
        let idx = crate::static_index::build_static_index(&dir, &source).unwrap();
        let apps = build_application_index(&idx);
        assert_eq!(apps.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
