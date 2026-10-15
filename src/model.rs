use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize)]
pub struct Tool {
    pub name: String,
    pub label: String,
}
#[derive(Clone, Deserialize)]
pub struct Shelf {
    pub name: String,
    pub label: String,
    pub tools: Vec<Tool>,
}

/// Shell records are deliberately separate from a future geometry evaluation graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: u64,
    pub name: String,
    pub tool: String,
    pub label: String,
    pub position: [f32; 2],
    pub translate: [f32; 3],
    pub rotate: [f32; 3],
    pub scale: [f32; 3],
}
#[derive(Default)]
pub struct Session {
    pub nodes: Vec<Node>,
    pub selected: Option<u64>,
    next_id: u64,
}
impl Session {
    pub fn add(&mut self, tool: &Tool) -> u64 {
        self.next_id += 1;
        let id = self.next_id;
        self.nodes.push(Node {
            id,
            name: format!("{}{}", tool.label.to_lowercase().replace(' ', "_"), id),
            tool: tool.name.clone(),
            label: tool.label.clone(),
            position: [
                75.0 + ((id - 1) % 3) as f32 * 145.0,
                75.0 + ((id - 1) / 3) as f32 * 90.0,
            ],
            translate: [0.0; 3],
            rotate: [0.0; 3],
            scale: [1.0; 3],
        });
        self.selected = Some(id);
        id
    }
    pub fn selected_mut(&mut self) -> Option<&mut Node> {
        let id = self.selected?;
        self.nodes.iter_mut().find(|n| n.id == id)
    }
    pub fn remove_selected(&mut self) {
        if let Some(id) = self.selected.take() {
            self.nodes.retain(|n| n.id != id);
        }
    }
}
