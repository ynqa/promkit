use crate::{
    core::cursor::CompositeCursor,
    grapheme::{trim, StyledGraphemes},
    impl_as_any,
    pane::Pane,
};

use super::{JsonNode, JsonPath, JsonSyntaxKind};

#[derive(Clone)]
pub struct JsonBundle {
    roots: Vec<JsonNode>,
    cursor: CompositeCursor<Vec<JsonSyntaxKind>>,
}

impl JsonBundle {
    pub fn new<I: IntoIterator<Item = JsonNode>>(iter: I) -> Self {
        let roots: Vec<JsonNode> = iter.into_iter().collect();
        Self {
            roots: roots.clone(),
            cursor: CompositeCursor::new(roots.iter().map(|r| r.flatten_visibles()), 0),
        }
    }

    pub fn roots(&self) -> &Vec<JsonNode> {
        &self.roots
    }

    pub fn flatten_kinds(&self) -> Vec<JsonSyntaxKind> {
        self.roots
            .iter()
            .flat_map(|root| root.flatten_visibles().into_iter())
            .collect()
    }

    pub fn current_bundle_path_from_root(&self) -> JsonPath {
        let (index, inner) = self.cursor.current_bundle_index_and_inner_position();
        let kind = self.cursor.bundle()[index][inner].clone();
        let binding = vec![];
        let path = match kind {
            JsonSyntaxKind::ArrayStart { path, .. } => path,
            JsonSyntaxKind::ArrayEntry { path, .. } => path,
            JsonSyntaxKind::ArrayFolded { path, .. } => path,
            JsonSyntaxKind::MapStart { path, .. } => path,
            JsonSyntaxKind::MapEntry { path, .. } => path,
            JsonSyntaxKind::MapFolded { path, .. } => path,
            _ => binding,
        };

        path.clone()
    }

    pub fn toggle(&mut self) {
        let (index, inner) = self.cursor.current_bundle_index_and_inner_position();

        let kind = self.cursor.bundle()[index][inner].clone();
        let route = match kind {
            JsonSyntaxKind::ArrayStart { path, .. } => path,
            JsonSyntaxKind::ArrayFolded { path, .. } => path,
            JsonSyntaxKind::MapStart { path, .. } => path,
            JsonSyntaxKind::MapFolded { path, .. } => path,
            _ => return,
        };

        self.roots[index].toggle(&route);
        self.cursor = CompositeCursor::new(
            self.roots.iter().map(|r| r.flatten_visibles()),
            self.cursor.cross_contents_position(),
        );
    }

    /// Toggles the visibility of all nodes in the JSON tree.
    /// `expand` controls whether nodes are expanded or collapsed.
    fn toggle_all_visibility(&mut self, expand: bool) {
        fn toggle_visibility(node: &mut JsonNode, expand: bool) {
            match node {
                JsonNode::Object {
                    children,
                    children_visible,
                } => {
                    *children_visible = expand;
                    for child in children.values_mut() {
                        toggle_visibility(child, expand);
                    }
                }
                JsonNode::Array {
                    children,
                    children_visible,
                } => {
                    *children_visible = expand;
                    for child in children.iter_mut() {
                        toggle_visibility(child, expand);
                    }
                }
                _ => {}
            }
        }

        for root in &mut self.roots {
            toggle_visibility(root, expand);
        }
        self.cursor = CompositeCursor::new(
            self.roots.iter().map(|r| r.flatten_visibles()),
            self.cursor.cross_contents_position(),
        );
    }

    pub fn collapse_all(&mut self) {
        self.toggle_all_visibility(false);
    }

    pub fn expand_all(&mut self) {
        self.toggle_all_visibility(true);
    }

    pub fn backward(&mut self) -> bool {
        self.cursor.backward()
    }

    pub fn forward(&mut self) -> bool {
        self.cursor.forward()
    }

    /// Moves the cursor to the head of the JSON tree.
    pub fn move_to_head(&mut self) {
        self.cursor.move_to_head()
    }

    /// Moves the cursor to the tail of the JSON tree.
    pub fn move_to_tail(&mut self) {
        self.cursor.move_to_tail()
    }
}

#[derive(Clone)]
pub struct Renderer {
    pub bundle: JsonBundle,
    pub theme: super::Theme,
}

impl_as_any!(Renderer);

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        let layout = self
            .bundle
            .flatten_kinds()
            .iter()
            .enumerate()
            .map(|(i, kind)| {
                if i == self.bundle.cursor.cross_contents_position() {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from(
                            " ".repeat(super::Renderer::indent_level(kind, &self.theme)),
                        ),
                        super::Renderer::gen_syntax_style(kind, &self.theme)
                            .apply_attribute_to_all(self.theme.active_item_attribute),
                    ])
                } else {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from(
                            " ".repeat(super::Renderer::indent_level(kind, &self.theme)),
                        ),
                        super::Renderer::gen_syntax_style(kind, &self.theme),
                    ])
                    .apply_attribute_to_all(self.theme.inactive_item_attribute)
                }
            })
            .map(|row| trim(width as usize, &row))
            .collect::<Vec<StyledGraphemes>>();

        vec![Pane::new(
            layout,
            self.bundle.cursor.cross_contents_position(),
            self.theme.lines,
        )]
    }
}
