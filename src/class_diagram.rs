extern crate env_logger;

use itertools::Itertools;

pub struct ClassDiagram {
    pub classes: Vec<String>,
    pub relationships: Vec<String>,
    pub path: String,
}

impl ClassDiagram {
    pub fn new() -> Self {
        Self {
            classes: vec![],
            relationships: vec![],
            path: String::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.classes.is_empty() && self.relationships.is_empty()
    }

    pub fn render(&self) -> String {
        let mut res = String::new();
        res.push_str("```mermaid\n");

        if !self.path.is_empty() {
            res.push_str("---\n");
            res.push_str(&format!("title: {}\n", self.path));
            res.push_str("---\n");
        }

        res.push_str("classDiagram\n");

        for class in self.classes.iter().unique() {
            res.push_str(class);
            res.push_str("\n\n");
        }

        for relationship in self.relationships.iter().unique() {
            res.push_str(relationship);
            res.push('\n');
        }

        res = res.trim_end().to_string();

        res.push_str("\n```\n");

        res
    }

    pub fn write_to_file(&self) -> bool {
        if self.is_empty() {
            info!("No classes found for {0:?}.", self.path);
            return false;
        }

        let path = format!("./output/{}.md", self.path);
        if let Some(parent_dir) = std::path::Path::new(&path).parent() {
            std::fs::create_dir_all(parent_dir).unwrap();
        }
        std::fs::write(&path, self.render()).unwrap();
        println!("Mermaid file written to: {:?}", path);

        true
    }
}
