use crate::projects::framework_macro::FrameworkHeuristics;

pub enum Framework {
    Laravel,
    Rails,
    Vue,
    NextJS,
    Ansible,
}

crate::frameworks! {
    Laravel {
        files: ["composer.json"],
        json_checks: [("composer.json", "require.laravel/framework")],
    },
    Ansible {
        directories: ["playbooks"],
        custom: |project_path: &str| {
            use crate::projects::framework_macro::has_file;
            has_file(project_path, "ansible.cfg") ||
            has_file(project_path, "playbook.yml") ||
            has_file(project_path, "site.yml") ||
            has_file(project_path, "inventory") ||
            has_file(project_path, "hosts") ||
            has_file(project_path, "hosts.yml")
        },
    },
}

pub struct FrameworkDetector {
    heuristics: &'static [&'static dyn FrameworkHeuristics],
}

impl FrameworkDetector {
    pub fn new() -> FrameworkDetector {
        FrameworkDetector {
            heuristics: HEURISTICS,
        }
    }

    pub fn detect(&self, project_path: &str) -> Option<String> {
        for h in self.heuristics {
            if h.matches(project_path) {
                return Some(String::from(h.name()));
            }
        }

        None
    }
}
