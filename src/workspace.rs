use craft_ui::{
    docking::{Layout, Node},
    layout::{SplitAxis, SplitSize},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Panel {
    Scene,
    Parameters,
    Network,
    Spreadsheet,
    Animation,
    Tree,
    Materials,
    Performance,
    Log,
    Composite,
    Tasks,
    Python,
}
impl Panel {
    pub const ALL: [Self; 12] = [
        Self::Scene,
        Self::Parameters,
        Self::Network,
        Self::Spreadsheet,
        Self::Animation,
        Self::Tree,
        Self::Materials,
        Self::Performance,
        Self::Log,
        Self::Composite,
        Self::Tasks,
        Self::Python,
    ];
    pub fn label(&self) -> &'static str {
        match self {
            Self::Scene => "Scene View",
            Self::Parameters => "Parameters",
            Self::Network => "Network Editor",
            Self::Spreadsheet => "Geometry Spreadsheet",
            Self::Animation => "Animation Editor",
            Self::Tree => "Tree View",
            Self::Materials => "Material Palette",
            Self::Performance => "Performance Monitor",
            Self::Log => "Log Viewer",
            Self::Composite => "Composite View",
            Self::Tasks => "Task Graph",
            Self::Python => "Python Shell",
        }
    }
}
pub const DESKTOPS: [&str; 7] = [
    "Build",
    "Animate",
    "Solaris",
    "Solaris LookDev",
    "TOPs",
    "Technical",
    "Technical Wide",
];
fn tabs(panels: &[Panel]) -> Node<Panel> {
    Node::Tabs {
        panels: panels.to_vec(),
        active: 0,
    }
}
fn split(axis: SplitAxis, ratio: f32, first: Node<Panel>, second: Node<Panel>) -> Node<Panel> {
    Node::Split {
        axis,
        size: SplitSize::Ratio(ratio),
        first: Box::new(first),
        second: Box::new(second),
    }
}
pub fn preset(name: &str) -> Layout<Panel> {
    use Panel::*;
    use SplitAxis::*;
    let right = split(
        Vertical,
        0.5,
        tabs(&[Parameters, Performance, Log]),
        tabs(&[Network, Tree, Materials]),
    );
    let left = match name {
        "Animate" => split(
            Vertical,
            0.62,
            tabs(&[Scene, Composite]),
            tabs(&[Animation, Spreadsheet]),
        ),
        "Technical" | "Technical Wide" => split(
            Vertical,
            0.72,
            tabs(&[Scene, Spreadsheet, Composite]),
            tabs(&[Python, Animation]),
        ),
        "Solaris" | "Solaris LookDev" => split(
            Vertical,
            0.72,
            tabs(&[Scene, Composite]),
            tabs(&[Spreadsheet, Animation]),
        ),
        "TOPs" => tabs(&[Tasks, Scene, Animation, Spreadsheet, Composite]),
        _ => tabs(&[Scene, Animation, Composite, Spreadsheet]),
    };
    Layout {
        root: Some(split(
            Horizontal,
            if name == "Technical Wide" { 0.52 } else { 0.68 },
            left,
            right,
        )),
        floating: vec![],
    }
}
