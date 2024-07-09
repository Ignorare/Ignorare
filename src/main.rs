use pest::Parser;
use pest_derive::Parser;
use petgraph::visit::EdgeRef;
use petgraph::{graph::NodeIndex, Directed, Graph};
use std::collections::HashSet;
use std::fs::File;
use std::io::{LineWriter, Write};
use std::{collections::HashMap, fs};

#[derive(Parser)]
#[grammar = "../grammar.pest"]
pub struct IgnorefilesParser;

#[derive(Debug, Clone, PartialEq)]
pub struct Ignorefile {
    pub from: String,
    pub alias: String,
    pub ignores: Vec<String>,
    pub exports: String,
}

fn main() {
    let unparsed_file = fs::read_to_string("Ignorefile").expect("cannot read file");

    let mut graph = Graph::<Ignorefile, u32, Directed>::new();

    let ignorefiles = match IgnorefilesParser::parse(Rule::ignorefiles, &unparsed_file) {
        Ok(ignorefiles) => ignorefiles,
        Err(e) => panic!("error parsing file: {:?}", e),
    };

    let mut dag_indices: HashMap<String, NodeIndex<u32>> = HashMap::new();
    let mut export_indices: HashMap<String, NodeIndex<u32>> = HashMap::new();

    ignorefiles.for_each(|ignorefile| {
        ignorefile.into_inner().for_each(|file| {
            let mut from = String::new();
            let mut alias = String::new();
            let mut ignores = Vec::new();
            let mut exports = String::new();

            for line in file.into_inner() {
                match line.as_rule() {
                    Rule::from_statement => {
                        line.into_inner()
                            .for_each(|from_item| match from_item.as_rule() {
                                Rule::identifier => from = from_item.as_str().to_string(),
                                Rule::from_alias => {
                                    alias =
                                        from_item.into_inner().next().unwrap().as_str().to_string()
                                }
                                _ => println!("unexpected rule: {:?}", from_item),
                            });
                    }
                    Rule::ignore_statement => {
                        for ignore in line.into_inner() {
                            ignores.push(ignore.as_str().to_string());
                        }
                    }
                    Rule::export_statement => {
                        exports = line.into_inner().next().unwrap().as_str().to_string()
                    }
                    _ => {
                        println!("unexpected rule: {:?}", line);
                    }
                }
            }

            if from.is_empty() {
                return;
            }

            // add ignorefile to graph
            let ignorefile = Ignorefile {
                from,
                alias,
                ignores,
                exports,
            };
            let node = graph.add_node(ignorefile.clone());

            if !ignorefile.alias.is_empty() {
                dag_indices.insert(ignorefile.alias.clone(), node);
            }

            if !ignorefile.from.eq("scratch") {
                let from_node = dag_indices.get(&ignorefile.from).unwrap();
                graph.add_edge(node, *from_node, 0);
            }

            if !ignorefile.exports.is_empty() {
                export_indices.insert(ignorefile.exports.clone(), node);
            }
        });
    });

    export_indices.into_iter().for_each(|(export, node)| {
        println!("Creating export: {}", export);

        let file = File::create(&export).expect("Oops");
        let mut file = LineWriter::new(file);
        let mut visited = HashSet::new();

        // Start the recursive writing process
        write_ignores_recursively(node, &graph, &mut file, &mut visited);

        file.flush().expect("Oops");
    });
}

fn write_ignores_recursively(
    node: petgraph::graph::NodeIndex,
    graph: &Graph::<Ignorefile, u32, Directed>,
    file: &mut LineWriter<File>,
    visited: &mut HashSet<petgraph::graph::NodeIndex>,
) {
    if visited.contains(&node) {
        return;
    }
    visited.insert(node);

    let node_data = graph.node_weight(node).unwrap();
    file.write_all(node_data.ignores.join("\n").as_bytes())
        .expect("Oops");
    file.write_all(b"\n").expect("Oops");

    // Recursively process each connected node
    for edge in graph.edges(node) {
        let target = edge.target();
        write_ignores_recursively(target, graph, file, visited);
    }
}
