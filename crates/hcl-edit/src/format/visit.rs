use crate::{
    expr::{
        Array, Expression, FuncArgs, Object, ObjectKey, ObjectKeyMut, ObjectValue,
        ObjectValueAssignment, ObjectValueTerminator,
    },
    format::{
        decor::{ModifyDecor, Padding},
        Formatter,
    },
    structure::{AttributeMut, Block, BlockLabel, Body, StructureMut},
    visit_mut::{visit_body_mut, visit_expr_mut, visit_object_mut, VisitMut},
    Decorate,
};

#[doc(hidden)]
impl VisitMut for Formatter {
    fn visit_body_mut(&mut self, node: &mut Body) {
        self.visit(node, visit_body_mut);
    }

    fn visit_structure_mut(&mut self, mut node: StructureMut<'_>) {
        self.visit_decorated(
            &mut node,
            |prefix| prefix.indent_first_line(true).padding(Padding::End),
            |fmt, node| {
                if let Some(attr) = node.as_attribute_mut() {
                    fmt.visit_attr_mut(attr);
                } else if let Some(block) = node.as_block_mut() {
                    fmt.visit_block_mut(block);
                }
            },
            |suffix| suffix.padding(Padding::Start),
        );
    }

    fn visit_attr_mut(&mut self, mut node: AttributeMut<'_>) {
        self.visit_decor(
            node.key_decor_mut(),
            |prefix| prefix.inline().padding(Padding::End),
            |suffix| suffix.inline().padding(Padding::Both),
        );

        self.visit_decorated(
            node.value_mut(),
            |prefix| {
                prefix
                    .inline()
                    .indent_first_line(false)
                    .padding(Padding::Both)
            },
            visit_expr_mut,
            |suffix| suffix.inline().padding(Padding::Start),
        );
    }

    fn visit_block_mut(&mut self, node: &mut Block) {
        self.visit_decor(
            node.ident.decor_mut(),
            |prefix| prefix.inline().padding(Padding::End),
            |suffix| suffix.inline().padding(Padding::Both),
        );

        for label in &mut node.labels {
            self.visit_block_label_mut(label);
        }

        if node.body.prefer_oneline() && node.body.has_single_attribute() {
            self.visit_decorated(
                &mut node.body,
                |prefix| prefix.inline().padding(Padding::Both),
                visit_body_mut,
                |suffix| suffix.inline().padding(Padding::Both),
            );
        } else {
            self.visit_decorated(
                &mut node.body,
                |prefix| prefix.indent_first_line(false).padding(Padding::Start),
                |fmt, node| {
                    let mut fmt = fmt.indented();
                    visit_body_mut(&mut *fmt, node);
                },
                |suffix| suffix.indent_first_line(true).padding(Padding::Both),
            );
        }
    }

    fn visit_block_label_mut(&mut self, node: &mut BlockLabel) {
        self.visit_decor(
            node.decor_mut(),
            |prefix| prefix.inline().padding(Padding::End),
            |suffix| suffix.inline().padding(Padding::Both),
        );
    }

    fn visit_expr_mut(&mut self, node: &mut Expression) {
        self.visit(node, visit_expr_mut);
    }

    fn visit_array_mut(&mut self, node: &mut Array) {
        if has_multiline_elements(node.iter()) || node.trailing.is_multiline() {
            multiline_exprs(self, node.iter_mut());
            node.trailing.modify().leading_newline().format(self);
        } else {
            for (i, expr) in node.iter_mut().enumerate() {
                self.visit_decorated(
                    expr,
                    |prefix| prefix.padding(if i == 0 { Padding::End } else { Padding::Both }),
                    visit_expr_mut,
                    |suffix| suffix.padding(Padding::Start),
                );
            }

            let padding = if node.trailing_comma() {
                Padding::Both
            } else {
                Padding::Start
            };

            node.trailing.modify().padding(padding).format(self);
        }
    }

    fn visit_object_mut(&mut self, node: &mut Object) {
        if has_multiline_items(node.iter()) || node.trailing.is_multiline() {
            multiline_items(self, node.iter_mut());
            node.trailing.modify().leading_newline().format(self);
        } else {
            visit_object_mut(self, node);
            node.trailing.modify().padding(Padding::Both).format(self);
        }
    }

    fn visit_object_key_mut(&mut self, mut node: ObjectKeyMut<'_>) {
        self.visit_decor(
            node.decor_mut(),
            |prefix| prefix.padding(Padding::Both),
            |suffix| suffix.inline().padding(Padding::Both),
        );
    }

    fn visit_object_value_mut(&mut self, node: &mut ObjectValue) {
        node.set_assignment(ObjectValueAssignment::Equals);

        self.visit_decorated(
            node.expr_mut(),
            |prefix| prefix.inline().padding(Padding::Both),
            visit_expr_mut,
            |suffix| suffix.inline().padding(Padding::Start),
        );
    }

    fn visit_func_args_mut(&mut self, node: &mut FuncArgs) {
        if has_multiline_elements(node.iter()) || node.trailing.is_multiline() {
            multiline_exprs(self, node.iter_mut());
            node.trailing.modify().leading_newline().format(self);
        } else {
            for (i, expr) in node.iter_mut().enumerate() {
                self.visit_decorated(
                    expr,
                    |prefix| prefix.padding(if i == 0 { Padding::End } else { Padding::Both }),
                    visit_expr_mut,
                    |suffix| suffix.padding(Padding::Start),
                );
            }

            let padding = if node.trailing_comma() {
                Padding::Both
            } else {
                Padding::Start
            };

            node.trailing.modify().padding(padding).format(self);
        }
    }
}

fn multiline_exprs<'a>(fmt: &'a mut Formatter, iter: impl Iterator<Item = &'a mut Expression>) {
    let mut fmt = fmt.indented();

    for expr in iter {
        fmt.visit_decorated(
            expr,
            |prefix| prefix.leading_newline().padding(Padding::End),
            visit_expr_mut,
            |suffix| suffix.padding(Padding::Start),
        );
    }
}

fn multiline_items<'a>(
    fmt: &'a mut Formatter,
    iter: impl Iterator<Item = (ObjectKeyMut<'a>, &'a mut ObjectValue)>,
) {
    let mut fmt = fmt.indented();

    for (mut key, value) in iter {
        fmt.visit_decor(
            key.decor_mut(),
            |prefix| prefix.leading_newline().padding(Padding::End),
            |suffix| suffix.inline().padding(Padding::Both),
        );

        value.set_terminator(ObjectValueTerminator::None);

        fmt.visit_object_value_mut(value);
    }
}

fn has_multiline_items<'a>(
    mut iter: impl Iterator<Item = (&'a ObjectKey, &'a ObjectValue)>,
) -> bool {
    iter.any(|(k, v)| k.decor().is_multiline() || v.expr().decor().is_multiline())
}

fn has_multiline_elements<'a, T>(mut iter: impl Iterator<Item = &'a T>) -> bool
where
    T: Decorate + 'a,
{
    iter.any(|v| v.decor().is_multiline())
}
