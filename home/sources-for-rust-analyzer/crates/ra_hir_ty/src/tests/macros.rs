use std::fs;

use insta::assert_snapshot;
use ra_db::fixture::WithFixture;
use test_utils::project_dir;

use crate::test_db::TestDB;

use super::{infer, type_at, type_at_pos};

#[test]
fn cfg_impl_def() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs crate:main deps:foo cfg:test
use foo::S as T;
struct S;

#[cfg(test)]
impl S {
    fn foo1(&self) -> i32 { 0 }
}

#[cfg(not(test))]
impl S {
    fn foo2(&self) -> i32 { 0 }
}

fn test() {
    let t = (S.foo1(), S.foo2(), T.foo3(), T.foo4());
    t<|>;
}

//- /foo.rs crate:foo
struct S;

#[cfg(not(test))]
impl S {
    fn foo3(&self) -> i32 { 0 }
}

#[cfg(test)]
impl S {
    fn foo4(&self) -> i32 { 0 }
}
"#,
    );
    assert_eq!("(i32, {unknown}, i32, {unknown})", type_at_pos(&db, pos));
}

#[test]
fn infer_macros_expanded() {
    assert_snapshot!(
        infer(r#"
struct Foo(Vec<i32>);

macro_rules! foo {
    ($($item:expr),*) => {
            {
                Foo(vec![$($item,)*])
            }
    };
}

fn main() {
    let x = foo!(1,2);
}
"#),
        @r###"
    !0..17 '{Foo(v...,2,])}': Foo
    !1..4 'Foo': Foo({unknown}) -> Foo
    !1..16 'Foo(vec![1,2,])': Foo
    !5..15 'vec![1,2,]': {unknown}
    155..181 '{     ...,2); }': ()
    165..166 'x': Foo
    "###
    );
}

#[test]
fn infer_legacy_textual_scoped_macros_expanded() {
    assert_snapshot!(
        infer(r#"
struct Foo(Vec<i32>);

#[macro_use]
mod m {
    macro_rules! foo {
        ($($item:expr),*) => {
            {
                Foo(vec![$($item,)*])
            }
        };
    }
}

fn main() {
    let x = foo!(1,2);
    let y = crate::foo!(1,2);
}
"#),
        @r###"
    !0..17 '{Foo(v...,2,])}': Foo
    !1..4 'Foo': Foo({unknown}) -> Foo
    !1..16 'Foo(vec![1,2,])': Foo
    !5..15 'vec![1,2,]': {unknown}
    194..250 '{     ...,2); }': ()
    204..205 'x': Foo
    227..228 'y': {unknown}
    231..247 'crate:...!(1,2)': {unknown}
    "###
    );
}

#[test]
fn infer_path_qualified_macros_expanded() {
    assert_snapshot!(
        infer(r#"
#[macro_export]
macro_rules! foo {
    () => { 42i32 }
}

mod m {
    pub use super::foo as bar;
}

fn main() {
    let x = crate::foo!();
    let y = m::bar!();
}
"#),
        @r###"
    !0..5 '42i32': i32
    !0..5 '42i32': i32
    110..163 '{     ...!(); }': ()
    120..121 'x': i32
    147..148 'y': i32
    "###
    );
}

#[test]
fn expr_macro_expanded_in_various_places() {
    assert_snapshot!(
        infer(r#"
macro_rules! spam {
    () => (1isize);
}

fn spam() {
    spam!();
    (spam!());
    spam!().spam(spam!());
    for _ in spam!() {}
    || spam!();
    while spam!() {}
    break spam!();
    return spam!();
    match spam!() {
        _ if spam!() => spam!(),
    }
    spam!()(spam!());
    Spam { spam: spam!() };
    spam!()[spam!()];
    await spam!();
    spam!() as usize;
    &spam!();
    -spam!();
    spam!()..spam!();
    spam!() + spam!();
}
"#),
        @r###"
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    !0..6 '1isize': isize
    53..456 '{     ...!(); }': ()
    87..108 'spam!(...am!())': {unknown}
    114..133 'for _ ...!() {}': ()
    118..119 '_': {unknown}
    131..133 '{}': ()
    138..148 '|| spam!()': || -> isize
    154..170 'while ...!() {}': ()
    168..170 '{}': ()
    175..188 'break spam!()': !
    194..208 'return spam!()': !
    214..268 'match ...     }': isize
    238..239 '_': isize
    273..289 'spam!(...am!())': {unknown}
    295..317 'Spam {...m!() }': {unknown}
    323..339 'spam!(...am!()]': {unknown}
    364..380 'spam!(... usize': usize
    386..394 '&spam!()': &isize
    400..408 '-spam!()': isize
    414..430 'spam!(...pam!()': {unknown}
    436..453 'spam!(...pam!()': isize
    "###
    );
}

#[test]
fn infer_type_value_macro_having_same_name() {
    assert_snapshot!(
        infer(r#"
#[macro_export]
macro_rules! foo {
    () => {
        mod foo {
            pub use super::foo;
        }
    };
    ($x:tt) => {
        $x
    };
}

foo!();

fn foo() {
    let foo = foo::foo!(42i32);
}
"#),
        @r###"
    !0..5 '42i32': i32
    170..205 '{     ...32); }': ()
    180..183 'foo': i32
    "###
    );
}

#[test]
fn processes_impls_generated_by_macros() {
    let t = type_at(
        r#"
//- /main.rs
macro_rules! m {
    ($ident:ident) => (impl Trait for $ident {})
}
trait Trait { fn foo(self) -> u128 {} }
struct S;
m!(S);
fn test() { S.foo()<|>; }
"#,
    );
    assert_eq!(t, "u128");
}

#[test]
fn infer_assoc_items_generated_by_macros() {
    let t = type_at(
        r#"
//- /main.rs
macro_rules! m {
    () => (fn foo(&self) -> u128 {0})
}
struct S;
impl S {
    m!();
}

fn test() { S.foo()<|>; }
"#,
    );
    assert_eq!(t, "u128");
}

#[test]
fn infer_assoc_items_generated_by_macros_chain() {
    let t = type_at(
        r#"
//- /main.rs
macro_rules! m_inner {
    () => {fn foo(&self) -> u128 {0}}
}
macro_rules! m {
    () => {m_inner!();}
}

struct S;
impl S {
    m!();
}

fn test() { S.foo()<|>; }
"#,
    );
    assert_eq!(t, "u128");
}

#[test]
fn infer_macro_with_dollar_crate_is_correct_in_expr() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs crate:main deps:foo
fn test() {
    let x = (foo::foo!(1), foo::foo!(2));
    x<|>;
}

//- /lib.rs crate:foo
#[macro_export]
macro_rules! foo {
    (1) => { $crate::bar!() };
    (2) => { 1 + $crate::baz() };
}

#[macro_export]
macro_rules! bar {
    () => { 42 }
}

pub fn baz() -> usize { 31usize }
"#,
    );
    assert_eq!("(i32, usize)", type_at_pos(&db, pos));
}

#[test]
fn infer_macro_with_dollar_crate_is_correct_in_trait_associate_type() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs crate:main deps:foo
use foo::Trait;

fn test() {
    let msg = foo::Message(foo::MessageRef);
    let r = msg.deref();
    r<|>;
}

//- /lib.rs crate:foo
pub struct MessageRef;
pub struct Message(MessageRef);

pub trait Trait {
    type Target;
    fn deref(&self) -> &Self::Target;
}

#[macro_export]
macro_rules! expand {
    () => {
        impl Trait for Message {
            type Target = $crate::MessageRef;
            fn deref(&self) ->  &Self::Target {
                &self.0
            }
        }
    }
}

expand!();
"#,
    );
    assert_eq!("&MessageRef", type_at_pos(&db, pos));
}

#[test]
fn infer_type_value_non_legacy_macro_use_as() {
    assert_snapshot!(
        infer(r#"
mod m {
    macro_rules! _foo {
        ($x:ident) => { type $x = u64; }
    }
    pub(crate) use _foo as foo;
}

m::foo!(foo);
use foo as bar;
fn f() -> bar { 0 }
fn main() {
    let _a  = f();
}
"#),
        @r###"
    158..163 '{ 0 }': u64
    160..161 '0': u64
    174..196 '{     ...f(); }': ()
    184..186 '_a': u64
    190..191 'f': fn f() -> u64
    190..193 'f()': u64
    "###
    );
}

#[test]
fn infer_local_macro() {
    assert_snapshot!(
        infer(r#"
fn main() {
    macro_rules! foo {
        () => { 1usize }
    }
    let _a  = foo!();
}
"#),
        @r###"
    !0..6 '1usize': usize
    10..89 '{     ...!(); }': ()
    16..65 'macro_...     }': {unknown}
    74..76 '_a': usize
    "###
    );
}

#[test]
fn infer_local_inner_macros() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs crate:main deps:foo
fn test() {
    let x = foo::foo!(1);
    x<|>;
}

//- /lib.rs crate:foo
#[macro_export(local_inner_macros)]
macro_rules! foo {
    (1) => { bar!() };
}

#[macro_export]
macro_rules! bar {
    () => { 42 }
}

"#,
    );
    assert_eq!("i32", type_at_pos(&db, pos));
}

#[test]
fn infer_builtin_macros_line() {
    assert_snapshot!(
        infer(r#"
#[rustc_builtin_macro]
macro_rules! line {() => {}}

fn main() {
    let x = line!();
}
"#),
        @r###"
    !0..1 '0': i32
    63..87 '{     ...!(); }': ()
    73..74 'x': i32
    "###
    );
}

#[test]
fn infer_builtin_macros_file() {
    assert_snapshot!(
        infer(r#"
#[rustc_builtin_macro]
macro_rules! file {() => {}}

fn main() {
    let x = file!();
}
"#),
        @r###"
    !0..2 '""': &str
    63..87 '{     ...!(); }': ()
    73..74 'x': &str
    "###
    );
}

#[test]
fn infer_builtin_macros_column() {
    assert_snapshot!(
        infer(r#"
#[rustc_builtin_macro]
macro_rules! column {() => {}}

fn main() {
    let x = column!();
}
"#),
        @r###"
    !0..1 '0': i32
    65..91 '{     ...!(); }': ()
    75..76 'x': i32
    "###
    );
}

#[test]
fn infer_builtin_macros_concat() {
    assert_snapshot!(
        infer(r#"
#[rustc_builtin_macro]
macro_rules! concat {() => {}}

fn main() {
    let x = concat!("hello", concat!("world", "!"));
}
"#),
        @r###"
    !0..13 '"helloworld!"': &str
    65..121 '{     ...")); }': ()
    75..76 'x': &str
    "###
    );
}

#[test]
fn infer_builtin_macros_include() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs
#[rustc_builtin_macro]
macro_rules! include {() => {}}

include!("foo.rs");

fn main() {
    bar()<|>;
}

//- /foo.rs
fn bar() -> u32 {0}
"#,
    );
    assert_eq!("u32", type_at_pos(&db, pos));
}

#[test]
#[ignore]
fn include_accidentally_quadratic() {
    let file = project_dir().join("crates/ra_syntax/test_data/accidentally_quadratic");
    let big_file = fs::read_to_string(file).unwrap();
    let big_file = vec![big_file; 10].join("\n");

    let fixture = r#"
//- /main.rs
#[rustc_builtin_macro]
macro_rules! include {() => {}}

include!("foo.rs");

fn main() {
    RegisterBlock { }<|>;
}
    "#;
    let fixture = format!("{}\n//- /foo.rs\n{}", fixture, big_file);

    let (db, pos) = TestDB::with_position(&fixture);
    assert_eq!("RegisterBlock", type_at_pos(&db, pos));
}

#[test]
fn infer_builtin_macros_include_concat() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs
#[rustc_builtin_macro]
macro_rules! include {() => {}}

#[rustc_builtin_macro]
macro_rules! concat {() => {}}

include!(concat!("f", "oo.rs"));

fn main() {
    bar()<|>;
}

//- /foo.rs
fn bar() -> u32 {0}
"#,
    );
    assert_eq!("u32", type_at_pos(&db, pos));
}

#[test]
fn infer_builtin_macros_include_concat_with_bad_env_should_failed() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs
#[rustc_builtin_macro]
macro_rules! include {() => {}}

#[rustc_builtin_macro]
macro_rules! concat {() => {}}

#[rustc_builtin_macro]
macro_rules! env {() => {}}

include!(concat!(env!("OUT_DIR"), "/foo.rs"));

fn main() {
    bar()<|>;
}

//- /foo.rs
fn bar() -> u32 {0}
"#,
    );
    assert_eq!("{unknown}", type_at_pos(&db, pos));
}

#[test]
fn infer_builtin_macros_include_itself_should_failed() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs
#[rustc_builtin_macro]
macro_rules! include {() => {}}

include!("main.rs");

fn main() {
    0<|>
}
"#,
    );
    assert_eq!("i32", type_at_pos(&db, pos));
}

#[test]
fn infer_builtin_macros_concat_with_lazy() {
    assert_snapshot!(
        infer(r#"
macro_rules! hello {() => {"hello"}}

#[rustc_builtin_macro]
macro_rules! concat {() => {}}

fn main() {
    let x = concat!(hello!(), concat!("world", "!"));
}
"#),
        @r###"
    !0..13 '"helloworld!"': &str
    103..160 '{     ...")); }': ()
    113..114 'x': &str
    "###
    );
}

#[test]
fn infer_builtin_macros_env() {
    assert_snapshot!(
        infer(r#"
//- /main.rs env:foo=bar
#[rustc_builtin_macro]
macro_rules! env {() => {}}

fn main() {
    let x = env!("foo");
}
"#),
        @r###"
    !0..22 '"__RA_...TED__"': &str
    62..90 '{     ...o"); }': ()
    72..73 'x': &str
    "###
    );
}

#[test]
fn infer_derive_clone_simple() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs crate:main deps:core
#[derive(Clone)]
struct S;
fn test() {
    S.clone()<|>;
}

//- /lib.rs crate:core
#[prelude_import]
use clone::*;
mod clone {
    trait Clone {
        fn clone(&self) -> Self;
    }
}
"#,
    );
    assert_eq!("S", type_at_pos(&db, pos));
}

#[test]
fn infer_derive_clone_in_core() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /lib.rs crate:core
#[prelude_import]
use clone::*;
mod clone {
    trait Clone {
        fn clone(&self) -> Self;
    }
}
#[derive(Clone)]
pub struct S;

//- /main.rs crate:main deps:core
use core::S;
fn test() {
    S.clone()<|>;
}
"#,
    );
    assert_eq!("S", type_at_pos(&db, pos));
}

#[test]
fn infer_derive_clone_with_params() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs crate:main deps:core
#[derive(Clone)]
struct S;
#[derive(Clone)]
struct Wrapper<T>(T);
struct NonClone;
fn test() {
    (Wrapper(S).clone(), Wrapper(NonClone).clone())<|>;
}

//- /lib.rs crate:core
#[prelude_import]
use clone::*;
mod clone {
    trait Clone {
        fn clone(&self) -> Self;
    }
}
"#,
    );
    assert_eq!("(Wrapper<S>, {unknown})", type_at_pos(&db, pos));
}

#[test]
fn infer_custom_derive_simple() {
    // FIXME: this test current now do nothing
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs crate:main
use foo::Foo;

#[derive(Foo)]
struct S{}

fn test() {
    S{}<|>;
}
"#,
    );
    assert_eq!("S", type_at_pos(&db, pos));
}

#[test]
fn macro_in_arm() {
    assert_snapshot!(
        infer(r#"
macro_rules! unit {
    () => { () };
}

fn main() {
    let x = match () {
        unit!() => 92u32,
    };
}
"#),
        @r###"
    51..110 '{     ...  }; }': ()
    61..62 'x': u32
    65..107 'match ...     }': u32
    71..73 '()': ()
    84..91 'unit!()': ()
    95..100 '92u32': u32
    "###
    );
}
