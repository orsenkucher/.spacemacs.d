//! FIXME: write short doc here

use crate::completion::{CompletionContext, Completions};

pub(super) fn complete_macro_in_item_position(acc: &mut Completions, ctx: &CompletionContext) {
    // Show only macros in top level.
    if ctx.is_new_item {
        ctx.scope().process_all_names(&mut |name, res| {
            if let hir::ScopeDef::MacroDef(mac) = res {
                acc.add_macro(ctx, Some(name.to_string()), mac);
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use crate::completion::{test_utils::do_completion, CompletionItem, CompletionKind};

    fn do_reference_completion(code: &str) -> Vec<CompletionItem> {
        do_completion(code, CompletionKind::Reference)
    }

    #[test]
    fn completes_macros_as_item() {
        assert_debug_snapshot!(
            do_reference_completion(
                "
                //- /main.rs
                macro_rules! foo {
                    () => {}
                }

                fn foo() {}

                <|>
                "
            ),
            @r###"
        [
            CompletionItem {
                label: "foo!(…)",
                source_range: 48..48,
                delete: 48..48,
                insert: "foo!($0)",
                kind: Macro,
                detail: "macro_rules! foo",
            },
        ]
        "###
        );
    }

    #[test]
    fn completes_vec_macros_with_square_brackets() {
        assert_debug_snapshot!(
            do_reference_completion(
                "
                //- /main.rs
                /// Creates a [`Vec`] containing the arguments.
                ///
                /// - Create a [`Vec`] containing a given list of elements:
                ///
                /// ```
                /// let v = vec![1, 2, 3];
                /// assert_eq!(v[0], 1);
                /// assert_eq!(v[1], 2);
                /// assert_eq!(v[2], 3);
                /// ```
                macro_rules! vec {
                    () => {}
                }

                fn foo() {}

                <|>
                "
            ),
            @r###"
        [
            CompletionItem {
                label: "vec![…]",
                source_range: 282..282,
                delete: 282..282,
                insert: "vec![$0]",
                kind: Macro,
                detail: "macro_rules! vec",
                documentation: Documentation(
                    "Creates a [`Vec`] containing the arguments.\n\n- Create a [`Vec`] containing a given list of elements:\n\n```\nlet v = vec![1, 2, 3];\nassert_eq!(v[0], 1);\nassert_eq!(v[1], 2);\nassert_eq!(v[2], 3);\n```",
                ),
            },
        ]
        "###
        );
    }

    #[test]
    fn completes_macros_braces_guessing() {
        assert_debug_snapshot!(
            do_reference_completion(
                "
                //- /main.rs
                /// Foo
                ///
                /// Not call `fooo!()` `fooo!()`, or `_foo![]` `_foo![]`.
                /// Call as `let _=foo!  { hello world };`
                macro_rules! foo {
                    () => {}
                }

                fn main() {
                    <|>
                }
                "
            ),
            @r###"
        [
            CompletionItem {
                label: "foo! {…}",
                source_range: 164..164,
                delete: 164..164,
                insert: "foo! {$0}",
                kind: Macro,
                detail: "macro_rules! foo",
                documentation: Documentation(
                    "Foo\n\nNot call `fooo!()` `fooo!()`, or `_foo![]` `_foo![]`.\nCall as `let _=foo!  { hello world };`",
                ),
            },
            CompletionItem {
                label: "main()",
                source_range: 164..164,
                delete: 164..164,
                insert: "main()$0",
                kind: Function,
                lookup: "main",
                detail: "fn main()",
            },
        ]
        "###
        );
    }
}
