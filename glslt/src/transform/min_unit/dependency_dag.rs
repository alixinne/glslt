use glsl_lang::ast::SmolStr;
use petgraph::graph::NodeIndex;

#[derive(Clone, Copy)]
enum ExtractIdentState {
    Init,
    Ident { start_position: usize },
}

pub struct ExtractIdents<'i> {
    input: &'i str,
    current_position: usize,
    state: ExtractIdentState,
}

impl<'i> Iterator for ExtractIdents<'i> {
    type Item = &'i str;

    fn next(&mut self) -> Option<Self::Item> {
        // The rest of the string that wasn't parsed yet
        let remaining = &self.input[self.current_position..];

        let mut last_index = self.current_position;

        for (i, ch) in remaining.char_indices() {
            self.current_position = i;

            match self.state {
                ExtractIdentState::Init => {
                    // Initial state, look for a valid char
                    if ch == '_' || ('a'..'z').contains(&ch) || ('A'..'Z').contains(&ch) {
                        self.state = ExtractIdentState::Ident { start_position: i };
                    }
                }
                ExtractIdentState::Ident { start_position } => {
                    // First char seen, look for following chars
                    if ch == '_'
                        || ('a'..'z').contains(&ch)
                        || ('A'..'Z').contains(&ch)
                        || ('0'..'9').contains(&ch)
                    {
                        // Stay in the current state
                        self.state = ExtractIdentState::Ident { start_position };
                    } else {
                        // Not an ident, reset state
                        self.state = ExtractIdentState::Init;

                        // Return ident slice
                        return Some(&self.input[start_position..=last_index]);
                    }
                }
            }

            last_index = i;
        }

        None
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ExternalIdentifier {
    /// Function definition
    FunctionDefinition(SmolStr),
    /// Standalone declaration
    Declaration(SmolStr),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ExternalId<'a> {
    /// Function definition
    FunctionDefinition(&'a str),
    /// Standalone declaration
    Declaration(&'a str),
}

impl<'a> ExternalId<'a> {
    pub fn to_owned(&self) -> ExternalIdentifier {
        match self {
            Self::FunctionDefinition(sym) => ExternalIdentifier::FunctionDefinition((*sym).into()),
            Self::Declaration(sym) => ExternalIdentifier::Declaration((*sym).into()),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct DependencyDag {
    symbol_map: bimap::BiMap<ExternalIdentifier, usize>,
    graph: petgraph::Graph<(), (), petgraph::Directed>,
}

impl DependencyDag {
    pub fn declare_symbol(&mut self, raw_symbol: ExternalId) -> usize {
        // TODO: What to do when the raw_symbol is also a preprocessor #define'd symbol
        self.symbol_to_id(&raw_symbol.to_owned())
    }

    pub fn symbol_to_id(&mut self, symbol: &ExternalIdentifier) -> usize {
        if let Some(id) = self.symbol_map.get_by_left(symbol) {
            *id
        } else {
            // Add node
            let id = self.graph.add_node(()).index();
            self.symbol_map.insert(symbol.clone(), id);
            id
        }
    }

    pub fn add_dep(&mut self, scope: usize, dependency: usize) {
        // Self-reference makes no sense here
        assert!(scope != dependency);

        trace!(
            "DependencyDag: add_dep {:?} -> {:?}",
            self.symbol_map.get_by_right(&scope).unwrap(),
            self.symbol_map.get_by_right(&dependency).unwrap()
        );

        self.graph
            .add_edge(NodeIndex::new(scope), NodeIndex::new(dependency), ());
    }

    pub fn into_dependencies(mut self, wanted: &[ExternalIdentifier]) -> Vec<ExternalIdentifier> {
        // Create a wanted node
        let wanted_id = self.graph.add_node(());

        // The wanted node should reference all wanted targets
        for wanted in wanted {
            let id = self.symbol_to_id(wanted);
            self.graph.add_edge(wanted_id, NodeIndex::new(id), ());
        }

        // Select nodes by walking the graph from the root
        let mut dfs = petgraph::visit::DfsPostOrder::new(&self.graph, wanted_id);

        // Push all dependencies in order in a vector
        let mut res = Vec::with_capacity(self.symbol_map.len());

        while let Some(nx) = dfs.next(&self.graph) {
            // if let because wanted doesn't have an associated symbol
            if let Some(sym) = self.symbol_map.remove_by_right(&nx.index()) {
                res.push(sym.0);
            }
        }

        res
    }
}
