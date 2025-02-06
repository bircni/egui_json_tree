use crate::{
    node::JsonTreeNode,
    render::{JsonTreeRenderer, RenderContext},
    value::ToJsonTreeValue,
    DefaultExpand, JsonTreeResponse, JsonTreeStyle,
};
use egui::{Id, Ui};
use std::hash::Hash;

pub struct JsonTreeConfig<'a, T: ToJsonTreeValue> {
    pub(crate) style: Option<JsonTreeStyle>,
    pub(crate) default_expand: Option<DefaultExpand<'a>>,
    pub(crate) renderer: JsonTreeRenderer<'a, T>,
}

impl<'a, T: ToJsonTreeValue> Default for JsonTreeConfig<'a, T> {
    fn default() -> Self {
        Self {
            style: Option::default(),
            default_expand: Option::default(),
            renderer: JsonTreeRenderer::default(),
        }
    }
}

/// An interactive JSON tree visualiser.
#[must_use = "You should call .show()"]
pub struct JsonTree<'a, T: ToJsonTreeValue> {
    pub(crate) id: Id,
    pub(crate) value: &'a T,
    pub(crate) config: JsonTreeConfig<'a, T>,
}

impl<'a, T: ToJsonTreeValue> JsonTree<'a, T> {
    /// Creates a new [`JsonTree`].
    /// `id` must be a globally unique identifier.
    pub fn new(id: impl Hash, value: &'a T) -> Self {
        Self {
            id: Id::new(id),
            value,
            config: JsonTreeConfig::default(),
        }
    }

    /// Override colors for JSON syntax highlighting, and search match highlighting.
    pub fn style(mut self, style: JsonTreeStyle) -> Self {
        self.config.style = Some(style);
        self
    }

    /// Override how the [`JsonTree`] expands arrays/objects by default.
    pub const fn default_expand(mut self, default_expand: DefaultExpand<'a>) -> Self {
        self.config.default_expand = Some(default_expand);
        self
    }

    /// A convenience method for conditionally registering a custom rendering hook.
    /// See [`JsonTree::on_render`].
    pub fn on_render_if(
        self,
        condition: bool,
        render_hook: impl FnMut(&mut Ui, RenderContext<'a, '_, T>) + 'a,
    ) -> Self {
        if condition {
            self.on_render(render_hook)
        } else {
            self
        }
    }

    /// Customise rendering of the [`JsonTree`], and/or handle interactions.
    ///
    /// This hook can be used to enrich the visualisation with
    /// extra UI interactions by handling [`egui::Response`] values,
    /// and adding UI elements such as buttons and checkboxes within the [`JsonTree`].
    ///
    /// The provided hook will be called in order to render array indices and brackets,
    /// object keys and braces, and non-recursive JSON values, instead of the default render implementation.
    ///
    /// The [`RenderContext`] argument to the hook provides information about the render call,
    /// including the JSON value and a JSON pointer to it.
    ///
    /// You may also call [`render_ctx.render_default(ui)`](crate::render::DefaultRender) on this argument
    /// (or on any of the render contexts contained within its enum variants) to render as normal.
    ///
    /// See [`copy_to_clipboard.rs`](https://github.com/dmackdev/egui_json_tree/blob/master/examples/demo/src/apps/copy_to_clipboard.rs)
    /// and [`editor.rs`](https://github.com/dmackdev/egui_json_tree/blob/master/examples/demo/src/apps/editor.rs)
    /// from the demo for detailed examples and usage.
    pub fn on_render(
        mut self,
        render_hook: impl FnMut(&mut Ui, RenderContext<'a, '_, T>) + 'a,
    ) -> Self {
        self.config.renderer.render_hook = Some(Box::new(render_hook));
        self
    }

    /// Show the JSON tree visualisation within the `Ui`.
    pub fn show(self, ui: &mut Ui) -> JsonTreeResponse {
        JsonTreeNode::show(self, ui)
    }
}

#[cfg(test)]
mod test {
    use crate::DefaultExpand;

    use super::JsonTree;

    #[test]
    fn test_search_populates_all_collapsing_state_ids_in_response() {
        let value = serde_json::json!({"foo": [1, 2, [3]], "bar": { "qux" : false, "thud": { "a/b": [4, 5, { "m~n": "Greetings!" }]}, "grep": 21}, "baz": null});

        egui::__run_test_ui(|ui| {
            let response = JsonTree::new("id", &value)
                .default_expand(DefaultExpand::SearchResults("g"))
                .show(ui);

            assert_eq!(response.collapsing_state_ids.len(), 7);
        });
    }
}
