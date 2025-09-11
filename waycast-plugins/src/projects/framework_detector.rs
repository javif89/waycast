use crate::projects::framework_macro::FrameworkHeuristics;

pub enum Framework {
    Laravel,
    Rails,
    Vue,
    NextJS,
    Svelte,
    Django,
    Flask,
    Fiber,
    Ansible,
}

crate::frameworks! {
    Laravel {
        files: ["composer.json"],
        json_checks: [("composer.json", "require.laravel/framework")],
    },
    Rails {
        files: ["Gemfile"],
        json_checks: [("package.json", "dependencies.rails"), ("package.json", "devDependencies.rails")],
        custom: |project_path: &str| {
            use crate::projects::framework_macro::{has_file, read_json_config};

            // Check for Gemfile with rails gem
            if let Ok(content) = std::fs::read_to_string(format!("{}/Gemfile", project_path)) {
                if content.contains("gem 'rails'") || content.contains("gem \"rails\"") {
                    return true;
                }
            }

            // Check for Rails-specific directories
            has_file(project_path, "config/application.rb") ||
            has_file(project_path, "app/controllers") ||
            has_file(project_path, "config/routes.rb")
        },
    },
    NextJS {
        files: ["package.json"],
        json_checks: [("package.json", "dependencies.next"), ("package.json", "devDependencies.next")],
        custom: |project_path: &str| {
            use crate::projects::framework_macro::has_file;
            has_file(project_path, "next.config.js") || has_file(project_path, "next.config.mjs")
        },
    },
    Vue {
        files: ["package.json"],
        json_checks: [("package.json", "dependencies.vue"), ("package.json", "devDependencies.vue")],
        custom: |project_path: &str| {
            use crate::projects::framework_macro::has_file;
            has_file(project_path, "vue.config.js") ||
            has_file(project_path, "src/App.vue")
        },
    },
    Svelte {
        files: ["package.json"],
        json_checks: [("package.json", "dependencies.svelte"), ("package.json", "devDependencies.svelte")],
        custom: |project_path: &str| {
            use crate::projects::framework_macro::has_file;
            has_file(project_path, "svelte.config.js") ||
            has_file(project_path, "src/App.svelte")
        },
    },
    Django {
        files: ["manage.py"],
        custom: |project_path: &str| {
            use crate::projects::framework_macro::has_file;

            // Check for requirements.txt with Django
            if let Ok(content) = std::fs::read_to_string(format!("{}/requirements.txt", project_path)) {
                if content.contains("Django") || content.contains("django") {
                    return true;
                }
            }

            // Check for Django-specific files
            has_file(project_path, "settings.py") ||
            has_file(project_path, "wsgi.py") ||
            has_file(project_path, "urls.py")
        },
    },
    Flask {
        custom: |project_path: &str| {
            use crate::projects::framework_macro::has_file;

            // Check for requirements.txt with Flask
            if let Ok(content) = std::fs::read_to_string(format!("{}/requirements.txt", project_path)) {
                if content.contains("Flask") || content.contains("flask") {
                    return true;
                }
            }

            // Check for common Flask files
            has_file(project_path, "app.py") ||
            has_file(project_path, "main.py") ||
            has_file(project_path, "run.py")
        },
    },
    Fiber {
        files: ["go.mod"],
        custom: |project_path: &str| {
            // Check for go.mod with fiber dependency
            if let Ok(content) = std::fs::read_to_string(format!("{}/go.mod", project_path)) {
                if content.contains("github.com/gofiber/fiber") {
                    return true;
                }
            }
            false
        },
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
