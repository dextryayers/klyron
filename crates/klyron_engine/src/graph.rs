use std::collections::{HashMap, HashSet};

use regex::Regex;

#[derive(Debug, Clone)]
pub struct ModuleGraphEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone)]
pub struct ModuleGraph {
    pub edges: Vec<ModuleGraphEdge>,
    pub cycles: Vec<Vec<String>>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            cycles: Vec::new(),
        }
    }

    pub fn parse_source(source: &str, module_name: &str) -> Vec<ModuleGraphEdge> {
        let mut edges = Vec::new();
        let import_re = Regex::new(r#"(?:import\s+(?:\{[^}]*\}\s+)?(?:[\w*{}\s,]+)?\s+from\s+['"]([^'"]+)['"]|require\s*\(\s*['"]([^'"]+)['"]\s*\))"#).unwrap();

        for cap in import_re.captures_iter(source) {
            let target = cap.get(1).or_else(|| cap.get(2))
                .map(|m| m.as_str().to_string());
            if let Some(target) = target {
                edges.push(ModuleGraphEdge {
                    from: module_name.to_string(),
                    to: target,
                });
            }
        }

        edges
    }

    pub fn add_edges(&mut self, edges: Vec<ModuleGraphEdge>) {
        self.edges.extend(edges);
    }

    pub fn detect_cycles(&mut self) -> Vec<Vec<String>> {
        let adj = self.adjacency_list();
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        let nodes: Vec<String> = adj.keys().cloned().collect();

        for node in &nodes {
            if !visited.contains(node) {
                Self::dfs_cycle(node, &adj, &mut visited, &mut rec_stack, &mut path, &mut cycles);
            }
        }

        self.cycles = cycles.clone();
        cycles
    }

    fn dfs_cycle(
        node: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = adj.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    Self::dfs_cycle(neighbor, adj, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(neighbor) {
                    let cycle_start = path.iter().position(|n| n == neighbor).unwrap();
                    let cycle: Vec<String> = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    pub fn adjacency_list(&self) -> HashMap<String, Vec<String>> {
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.from.clone()).or_default().push(edge.to.clone());
            adj.entry(edge.to.clone()).or_default();
        }
        adj
    }

    pub fn has_cycles(&self) -> bool {
        !self.cycles.is_empty()
    }

    pub fn export_dot(&self) -> String {
        let cycles_set: HashSet<(String, String)> = self.cycles.iter()
            .flat_map(|cycle| {
                let mut pairs = Vec::new();
                for i in 0..cycle.len() {
                    let from = cycle[i].clone();
                    let to = cycle[(i + 1) % cycle.len()].clone();
                    pairs.push((from, to));
                }
                pairs
            })
            .collect();

        let mut dot = String::from("digraph ModuleGraph {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=rounded];\n");
        dot.push_str("  splines=true;\n\n");

        for edge in &self.edges {
            let escaped_from = edge.from.replace('"', "\\\"");
            let escaped_to = edge.to.replace('"', "\\\"");
            if cycles_set.contains(&(edge.from.clone(), edge.to.clone())) {
                dot.push_str(&format!("  \"{}\" -> \"{}\" [color=red, penwidth=2.0];\n", escaped_from, escaped_to));
            } else {
                dot.push_str(&format!("  \"{}\" -> \"{}\";\n", escaped_from, escaped_to));
            }
        }

        dot.push_str("}\n");
        dot
    }

    pub fn clear(&mut self) {
        self.edges.clear();
        self.cycles.clear();
    }
}

impl Default for ModuleGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph_empty() {
        let graph = ModuleGraph::new();
        assert!(graph.edges.is_empty());
        assert!(graph.cycles.is_empty());
    }

    #[test]
    fn test_add_edges() {
        let mut graph = ModuleGraph::new();
        graph.add_edges(vec![
            ModuleGraphEdge { from: "a".to_string(), to: "b".to_string() },
            ModuleGraphEdge { from: "b".to_string(), to: "c".to_string() },
        ]);
        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn test_adjacency_list() {
        let mut graph = ModuleGraph::new();
        graph.add_edges(vec![
            ModuleGraphEdge { from: "a".to_string(), to: "b".to_string() },
            ModuleGraphEdge { from: "a".to_string(), to: "c".to_string() },
        ]);
        let adj = graph.adjacency_list();
        assert_eq!(adj.get("a").unwrap().len(), 2);
        assert!(adj.get("a").unwrap().contains(&"b".to_string()));
        assert!(adj.get("a").unwrap().contains(&"c".to_string()));
    }

    #[test]
    fn test_no_cycles() {
        let mut graph = ModuleGraph::new();
        graph.add_edges(vec![
            ModuleGraphEdge { from: "a".to_string(), to: "b".to_string() },
            ModuleGraphEdge { from: "b".to_string(), to: "c".to_string() },
        ]);
        let cycles = graph.detect_cycles();
        assert!(cycles.is_empty());
        assert!(!graph.has_cycles());
    }

    #[test]
    fn test_simple_cycle() {
        let mut graph = ModuleGraph::new();
        graph.add_edges(vec![
            ModuleGraphEdge { from: "a".to_string(), to: "b".to_string() },
            ModuleGraphEdge { from: "b".to_string(), to: "a".to_string() },
        ]);
        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());
        assert!(graph.has_cycles());
    }

    #[test]
    fn test_triangular_cycle() {
        let mut graph = ModuleGraph::new();
        graph.add_edges(vec![
            ModuleGraphEdge { from: "a".to_string(), to: "b".to_string() },
            ModuleGraphEdge { from: "b".to_string(), to: "c".to_string() },
            ModuleGraphEdge { from: "c".to_string(), to: "a".to_string() },
        ]);
        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_self_cycle() {
        let mut graph = ModuleGraph::new();
        graph.add_edges(vec![
            ModuleGraphEdge { from: "a".to_string(), to: "a".to_string() },
        ]);
        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_dot_output_no_cycles() {
        let mut graph = ModuleGraph::new();
        graph.add_edges(vec![
            ModuleGraphEdge { from: "main".to_string(), to: "lib".to_string() },
        ]);
        let dot = graph.export_dot();
        assert!(dot.starts_with("digraph ModuleGraph {"));
        assert!(dot.contains("\"main\" -> \"lib\""));
        assert!(dot.ends_with("}\n"));
    }

    #[test]
    fn test_dot_output_with_cycles() {
        let mut graph = ModuleGraph::new();
        graph.add_edges(vec![
            ModuleGraphEdge { from: "a".to_string(), to: "b".to_string() },
            ModuleGraphEdge { from: "b".to_string(), to: "a".to_string() },
        ]);
        graph.detect_cycles();
        let dot = graph.export_dot();
        assert!(dot.contains("[color=red"));
    }

    #[test]
    fn test_parse_source_simple_import() {
        let edges = ModuleGraph::parse_source("import { foo } from './bar';", "main.js");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from, "main.js");
        assert_eq!(edges[0].to, "./bar");
    }

    #[test]
    fn test_parse_source_require() {
        let edges = ModuleGraph::parse_source("const x = require('./module');", "main.js");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].to, "./module");
    }

    #[test]
    fn test_parse_source_multiple_imports() {
        let edges = ModuleGraph::parse_source(
            "import a from 'a';\nimport b from 'b';\nconst c = require('c');",
            "main.js",
        );
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_parse_source_no_imports() {
        let edges = ModuleGraph::parse_source("const x = 1;", "main.js");
        assert!(edges.is_empty());
    }

    #[test]
    fn test_clear_graph() {
        let mut graph = ModuleGraph::new();
        graph.add_edges(vec![
            ModuleGraphEdge { from: "a".to_string(), to: "b".to_string() },
        ]);
        graph.detect_cycles();
        graph.clear();
        assert!(graph.edges.is_empty());
        assert!(graph.cycles.is_empty());
    }

    #[test]
    fn test_disconnected_components() {
        let mut graph = ModuleGraph::new();
        graph.add_edges(vec![
            ModuleGraphEdge { from: "a".to_string(), to: "b".to_string() },
            ModuleGraphEdge { from: "c".to_string(), to: "d".to_string() },
        ]);
        let cycles = graph.detect_cycles();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_cycle_contains_correct_nodes() {
        let mut graph = ModuleGraph::new();
        graph.add_edges(vec![
            ModuleGraphEdge { from: "x".to_string(), to: "y".to_string() },
            ModuleGraphEdge { from: "y".to_string(), to: "z".to_string() },
            ModuleGraphEdge { from: "z".to_string(), to: "x".to_string() },
        ]);
        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());
        let cycle = &cycles[0];
        assert!(cycle.contains(&"x".to_string()));
        assert!(cycle.contains(&"y".to_string()));
        assert!(cycle.contains(&"z".to_string()));
    }
}
