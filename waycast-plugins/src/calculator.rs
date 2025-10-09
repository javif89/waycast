use waycast_core::{LauncherListItem, LauncherPlugin};
use waycast_macros::{launcher_entry, plugin};

#[derive(Debug, Clone)]
pub struct CalculatorResult {
    value: String,
}

impl LauncherListItem for CalculatorResult {
    launcher_entry! {
        id: "calculator_result".into(),
        title: self.value.to_owned(),
        icon: {
           "accessories-calculator".into()
        },
        execute: {
            Ok(())
        }
    }
}

pub struct CalculatorPlugin;

impl Default for CalculatorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorPlugin {
    pub fn new() -> Self {
        CalculatorPlugin
    }
}

impl LauncherPlugin for CalculatorPlugin {
    plugin! {
        name: "calculator",
        priority: 1100,
        description: "Run different calculations and get the result in the launcher list",
        prefix: "calc"
    }

    fn filter(&self, query: &str) -> Vec<Box<dyn LauncherListItem>> {
        if query.is_empty() {
            return self.default_list();
        }

        // TODO: Just check if the query even resembles a math expression
        // before wasting valuable CPU cycles trying to evaluate

        if let Ok(result) = mathengine::evaluate_expression(query) {
            return vec![Box::new(CalculatorResult {
                value: format!("{}", result),
            })];
        }

        Vec::new()
    }
}

pub fn new() -> CalculatorPlugin {
    CalculatorPlugin::new()
}
