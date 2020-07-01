use insta::assert_snapshot;
use ra_db::fixture::WithFixture;
use test_utils::mark;

use crate::test_db::TestDB;

use super::infer;

#[test]
fn bug_484() {
    assert_snapshot!(
        infer(r#"
fn test() {
   let x = if true {};
}
"#),
        @r###"
    10..36 '{    l... {}; }': ()
    19..20 'x': ()
    23..33 'if true {}': ()
    26..30 'true': bool
    31..33 '{}': ()
    "###
    );
}

#[test]
fn no_panic_on_field_of_enum() {
    assert_snapshot!(
        infer(r#"
enum X {}

fn test(x: X) {
    x.some_field;
}
"#),
        @r###"
    19..20 'x': X
    25..46 '{     ...eld; }': ()
    31..32 'x': X
    31..43 'x.some_field': {unknown}
    "###
    );
}

#[test]
fn bug_585() {
    assert_snapshot!(
        infer(r#"
fn test() {
    X {};
    match x {
        A::B {} => (),
        A::Y() => (),
    }
}
"#),
        @r###"
    10..88 '{     ...   } }': ()
    16..20 'X {}': {unknown}
    26..86 'match ...     }': ()
    32..33 'x': {unknown}
    44..51 'A::B {}': {unknown}
    55..57 '()': ()
    67..73 'A::Y()': {unknown}
    77..79 '()': ()
    "###
    );
}

#[test]
fn bug_651() {
    assert_snapshot!(
        infer(r#"
fn quux() {
    let y = 92;
    1 + y;
}
"#),
        @r###"
    10..40 '{     ...+ y; }': ()
    20..21 'y': i32
    24..26 '92': i32
    32..33 '1': i32
    32..37 '1 + y': i32
    36..37 'y': i32
    "###
    );
}

#[test]
fn recursive_vars() {
    mark::check!(type_var_cycles_resolve_completely);
    mark::check!(type_var_cycles_resolve_as_possible);
    assert_snapshot!(
        infer(r#"
fn test() {
    let y = unknown;
    [y, &y];
}
"#),
        @r###"
    10..47 '{     ...&y]; }': ()
    20..21 'y': &{unknown}
    24..31 'unknown': &{unknown}
    37..44 '[y, &y]': [&&{unknown}; _]
    38..39 'y': &{unknown}
    41..43 '&y': &&{unknown}
    42..43 'y': &{unknown}
    "###
    );
}

#[test]
fn recursive_vars_2() {
    assert_snapshot!(
        infer(r#"
fn test() {
    let x = unknown;
    let y = unknown;
    [(x, y), (&y, &x)];
}
"#),
        @r###"
    10..79 '{     ...x)]; }': ()
    20..21 'x': &&{unknown}
    24..31 'unknown': &&{unknown}
    41..42 'y': &&{unknown}
    45..52 'unknown': &&{unknown}
    58..76 '[(x, y..., &x)]': [(&&&{unknown}, &&&{unknown}); _]
    59..65 '(x, y)': (&&&{unknown}, &&&{unknown})
    60..61 'x': &&{unknown}
    63..64 'y': &&{unknown}
    67..75 '(&y, &x)': (&&&{unknown}, &&&{unknown})
    68..70 '&y': &&&{unknown}
    69..70 'y': &&{unknown}
    72..74 '&x': &&&{unknown}
    73..74 'x': &&{unknown}
    "###
    );
}

#[test]
fn infer_std_crash_1() {
    // caused stack overflow, taken from std
    assert_snapshot!(
        infer(r#"
enum Maybe<T> {
    Real(T),
    Fake,
}

fn write() {
    match something_unknown {
        Maybe::Real(ref mut something) => (),
    }
}
"#),
        @r###"
    53..138 '{     ...   } }': ()
    59..136 'match ...     }': ()
    65..82 'someth...nknown': Maybe<{unknown}>
    93..123 'Maybe:...thing)': Maybe<{unknown}>
    105..122 'ref mu...ething': &mut {unknown}
    127..129 '()': ()
    "###
    );
}

#[test]
fn infer_std_crash_2() {
    mark::check!(type_var_resolves_to_int_var);
    // caused "equating two type variables, ...", taken from std
    assert_snapshot!(
        infer(r#"
fn test_line_buffer() {
    &[0, b'\n', 1, b'\n'];
}
"#),
        @r###"
    22..52 '{     ...n']; }': ()
    28..49 '&[0, b...b'\n']': &[u8; _]
    29..49 '[0, b'...b'\n']': [u8; _]
    30..31 '0': u8
    33..38 'b'\n'': u8
    40..41 '1': u8
    43..48 'b'\n'': u8
    "###
    );
}

#[test]
fn infer_std_crash_3() {
    // taken from rustc
    assert_snapshot!(
        infer(r#"
pub fn compute() {
    match nope!() {
        SizeSkeleton::Pointer { non_zero: true, tail } => {}
    }
}
"#),
        @r###"
    17..107 '{     ...   } }': ()
    23..105 'match ...     }': ()
    29..36 'nope!()': {unknown}
    47..93 'SizeSk...tail }': {unknown}
    81..85 'true': bool
    81..85 'true': bool
    87..91 'tail': {unknown}
    97..99 '{}': ()
    "###
    );
}

#[test]
fn infer_std_crash_4() {
    // taken from rustc
    assert_snapshot!(
        infer(r#"
pub fn primitive_type() {
    match *self {
        BorrowedRef { type_: Primitive(p), ..} => {},
    }
}
"#),
        @r###"
    24..105 '{     ...   } }': ()
    30..103 'match ...     }': ()
    36..41 '*self': {unknown}
    37..41 'self': {unknown}
    52..90 'Borrow...), ..}': {unknown}
    73..85 'Primitive(p)': {unknown}
    83..84 'p': {unknown}
    94..96 '{}': ()
    "###
    );
}

#[test]
fn infer_std_crash_5() {
    // taken from rustc
    assert_snapshot!(
        infer(r#"
fn extra_compiler_flags() {
    for content in doesnt_matter {
        let name = if doesnt_matter {
            first
        } else {
            &content
        };

        let content = if ICE_REPORT_COMPILER_FLAGS_STRIP_VALUE.contains(&name) {
            name
        } else {
            content
        };
    }
}
"#),
        @r###"
    26..322 '{     ...   } }': ()
    32..320 'for co...     }': ()
    36..43 'content': &{unknown}
    47..60 'doesnt_matter': {unknown}
    61..320 '{     ...     }': ()
    75..79 'name': &&{unknown}
    82..166 'if doe...     }': &&{unknown}
    85..98 'doesnt_matter': bool
    99..128 '{     ...     }': &&{unknown}
    113..118 'first': &&{unknown}
    134..166 '{     ...     }': &&{unknown}
    148..156 '&content': &&{unknown}
    149..156 'content': &{unknown}
    181..188 'content': &{unknown}
    191..313 'if ICE...     }': &{unknown}
    194..231 'ICE_RE..._VALUE': {unknown}
    194..247 'ICE_RE...&name)': bool
    241..246 '&name': &&&{unknown}
    242..246 'name': &&{unknown}
    248..276 '{     ...     }': &&{unknown}
    262..266 'name': &&{unknown}
    282..313 '{     ...     }': &{unknown}
    296..303 'content': &{unknown}
    "###
    );
}

#[test]
fn infer_nested_generics_crash() {
    // another crash found typechecking rustc
    assert_snapshot!(
        infer(r#"
struct Canonical<V> {
    value: V,
}
struct QueryResponse<V> {
    value: V,
}
fn test<R>(query_response: Canonical<QueryResponse<R>>) {
    &query_response.value;
}
"#),
        @r###"
    91..105 'query_response': Canonical<QueryResponse<R>>
    136..166 '{     ...lue; }': ()
    142..163 '&query....value': &QueryResponse<R>
    143..157 'query_response': Canonical<QueryResponse<R>>
    143..163 'query_....value': QueryResponse<R>
    "###
    );
}

#[test]
fn infer_paren_macro_call() {
    assert_snapshot!(
        infer(r#"
macro_rules! bar { () => {0u32} }
fn test() {
    let a = (bar!());
}
"#),
        @r###"
    !0..4 '0u32': u32
    44..69 '{     ...()); }': ()
    54..55 'a': u32
    "###
    );
}

#[test]
fn bug_1030() {
    assert_snapshot!(infer(r#"
struct HashSet<T, H>;
struct FxHasher;
type FxHashSet<T> = HashSet<T, FxHasher>;

impl<T, H> HashSet<T, H> {
    fn default() -> HashSet<T, H> {}
}

pub fn main_loop() {
    FxHashSet::default();
}
"#),
    @r###"
    143..145 '{}': ()
    168..197 '{     ...t(); }': ()
    174..192 'FxHash...efault': fn default<{unknown}, FxHasher>() -> HashSet<{unknown}, FxHasher>
    174..194 'FxHash...ault()': HashSet<{unknown}, FxHasher>
    "###
    );
}

#[test]
fn issue_2669() {
    assert_snapshot!(
        infer(
            r#"trait A {}
    trait Write {}
    struct Response<T> {}

    trait D {
        fn foo();
    }

    impl<T:A> D for Response<T> {
        fn foo() {
            end();
            fn end<W: Write>() {
                let _x: T =  loop {};
            }
        }
    }"#
        ),
        @r###"
    147..262 '{     ...     }': ()
    161..164 'end': fn end<{unknown}>()
    161..166 'end()': ()
    199..252 '{     ...     }': ()
    221..223 '_x': !
    230..237 'loop {}': !
    235..237 '{}': ()
    "###
    )
}

#[test]
fn issue_2705() {
    assert_snapshot!(
        infer(r#"
trait Trait {}
fn test() {
    <Trait<u32>>::foo()
}
"#),
        @r###"
    25..52 '{     ...oo() }': ()
    31..48 '<Trait...>::foo': {unknown}
    31..50 '<Trait...:foo()': ()
    "###
    );
}

#[test]
fn issue_2683_chars_impl() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs crate:main deps:std
fn test() {
    let chars: std::str::Chars<'_>;
    (chars.next(), chars.nth(1))<|>;
}

//- /std.rs crate:std
#[prelude_import]
use prelude::*;

pub mod prelude {
    pub use crate::iter::Iterator;
    pub use crate::option::Option;
}

pub mod iter {
    pub use self::traits::Iterator;
    pub mod traits {
        pub use self::iterator::Iterator;

        pub mod iterator {
            pub trait Iterator {
                type Item;
                fn next(&mut self) -> Option<Self::Item>;
                fn nth(&mut self, n: usize) -> Option<Self::Item> {}
            }
        }
    }
}

pub mod option {
    pub enum Option<T> {}
}

pub mod str {
    pub struct Chars<'a> {}
    impl<'a> Iterator for Chars<'a> {
        type Item = char;
        fn next(&mut self) -> Option<char> {}
    }
}
"#,
    );

    assert_eq!("(Option<char>, Option<char>)", super::type_at_pos(&db, pos));
}

#[test]
fn issue_3642_bad_macro_stackover() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs
#[macro_export]
macro_rules! match_ast {
    (match $node:ident { $($tt:tt)* }) => { match_ast!(match ($node) { $($tt)* }) };

    (match ($node:expr) {
        $( ast::$ast:ident($it:ident) => $res:expr, )*
        _ => $catch_all:expr $(,)?
    }) => {{
        $( if let Some($it) = ast::$ast::cast($node.clone()) { $res } else )*
        { $catch_all }
    }};
}

fn main() {
    let anchor<|> = match_ast! {
        match parent {
            as => {},
            _ => return None
        }
    };
}"#,
    );

    assert_eq!("()", super::type_at_pos(&db, pos));
}

#[test]
fn issue_3999_slice() {
    assert_snapshot!(
        infer(r#"
fn foo(params: &[usize]) {
    match params {
        [ps @ .., _] => {}
    }
}
"#),
        @r###"
    7..13 'params': &[usize]
    25..80 '{     ...   } }': ()
    31..78 'match ...     }': ()
    37..43 'params': &[usize]
    54..66 '[ps @ .., _]': [usize]
    55..62 'ps @ ..': &[usize]
    60..62 '..': [usize]
    64..65 '_': usize
    70..72 '{}': ()
    "###
    );
}

#[test]
fn issue_3999_struct() {
    // rust-analyzer should not panic on seeing this malformed
    // record pattern.
    assert_snapshot!(
        infer(r#"
struct Bar {
    a: bool,
}
fn foo(b: Bar) {
    match b {
        Bar { a: .. } => {},
    }
}
"#),
        @r###"
    35..36 'b': Bar
    43..95 '{     ...   } }': ()
    49..93 'match ...     }': ()
    55..56 'b': Bar
    67..80 'Bar { a: .. }': Bar
    76..78 '..': bool
    84..86 '{}': ()
    "###
    );
}

#[test]
fn issue_4235_name_conflicts() {
    assert_snapshot!(
        infer(r#"
struct FOO {}
static FOO:FOO = FOO {};

impl FOO {
    fn foo(&self) {}
}

fn main() {
    let a = &FOO;
    a.foo();
}
"#), @r###"
    31..37 'FOO {}': FOO
    63..67 'self': &FOO
    69..71 '{}': ()
    85..119 '{     ...o(); }': ()
    95..96 'a': &FOO
    99..103 '&FOO': &FOO
    100..103 'FOO': FOO
    109..110 'a': &FOO
    109..116 'a.foo()': ()
    "###
    );
}

#[test]
fn issue_4465_dollar_crate_at_type() {
    assert_snapshot!(
        infer(r#"
pub struct Foo {}
pub fn anything<T>() -> T {
    loop {}
}
macro_rules! foo {
    () => {{
        let r: $crate::Foo = anything();
        r
    }};
}
fn main() {
    let _a = foo!();
}
"#), @r###"
    44..59 '{     loop {} }': T
    50..57 'loop {}': !
    55..57 '{}': ()
    !0..31 '{letr:...g();r}': Foo
    !4..5 'r': Foo
    !18..26 'anything': fn anything<Foo>() -> Foo
    !18..28 'anything()': Foo
    !29..30 'r': Foo
    163..187 '{     ...!(); }': ()
    173..175 '_a': Foo
    "###);
}

#[test]
fn issue_4053_diesel_where_clauses() {
    assert_snapshot!(
        infer(r#"
trait BoxedDsl<DB> {
    type Output;
    fn internal_into_boxed(self) -> Self::Output;
}

struct SelectStatement<From, Select, Distinct, Where, Order, LimitOffset, GroupBy, Locking> {
    order: Order,
}

trait QueryFragment<DB: Backend> {}

trait Into<T> { fn into(self) -> T; }

impl<F, S, D, W, O, LOf, DB> BoxedDsl<DB>
    for SelectStatement<F, S, D, W, O, LOf, G>
where
    O: Into<dyn QueryFragment<DB>>,
{
    type Output = XXX;

    fn internal_into_boxed(self) -> Self::Output {
        self.order.into();
    }
}
"#),
        @r###"
    65..69 'self': Self
    267..271 'self': Self
    466..470 'self': SelectStatement<F, S, D, W, O, LOf, {unknown}, {unknown}>
    488..522 '{     ...     }': ()
    498..502 'self': SelectStatement<F, S, D, W, O, LOf, {unknown}, {unknown}>
    498..508 'self.order': O
    498..515 'self.o...into()': dyn QueryFragment<DB>
    "###
    );
}

#[test]
fn issue_4953() {
    assert_snapshot!(
        infer(r#"
pub struct Foo(pub i64);
impl Foo {
    fn test() -> Self { Self(0i64) }
}
"#),
        @r###"
    58..72 '{ Self(0i64) }': Foo
    60..64 'Self': Foo(i64) -> Foo
    60..70 'Self(0i64)': Foo
    65..69 '0i64': i64
    "###
    );
    assert_snapshot!(
        infer(r#"
pub struct Foo<T>(pub T);
impl Foo<i64> {
    fn test() -> Self { Self(0i64) }
}
"#),
        @r###"
    64..78 '{ Self(0i64) }': Foo<i64>
    66..70 'Self': Foo<i64>(i64) -> Foo<i64>
    66..76 'Self(0i64)': Foo<i64>
    71..75 '0i64': i64
    "###
    );
}

#[test]
fn issue_4931() {
    assert_snapshot!(
        infer(r#"
trait Div<T> {
    type Output;
}

trait CheckedDiv: Div<()> {}

trait PrimInt: CheckedDiv<Output = ()> {
    fn pow(self);
}

fn check<T: PrimInt>(i: T) {
    i.pow();
}
"#),
        @r###"
    117..121 'self': Self
    148..149 'i': T
    154..170 '{     ...w(); }': ()
    160..161 'i': T
    160..167 'i.pow()': ()
    "###
    );
}

#[test]
fn issue_4885() {
    assert_snapshot!(
        infer(r#"
#[lang = "coerce_unsized"]
pub trait CoerceUnsized<T> {}

trait Future {
    type Output;
}
trait Foo<R> {
    type Bar;
}
fn foo<R, K>(key: &K) -> impl Future<Output = K::Bar>
where
    K: Foo<R>,
{
    bar(key)
}
fn bar<R, K>(key: &K) -> impl Future<Output = K::Bar>
where
    K: Foo<R>,
{
}
"#),
        @r###"
    136..139 'key': &K
    198..214 '{     ...key) }': impl Future<Output = <K as Foo<R>>::Bar>
    204..207 'bar': fn bar<R, K>(&K) -> impl Future<Output = <K as Foo<R>>::Bar>
    204..212 'bar(key)': impl Future<Output = <K as Foo<R>>::Bar>
    208..211 'key': &K
    228..231 'key': &K
    290..293 '{ }': ()
    "###
    );
}

#[test]
fn issue_4800() {
    assert_snapshot!(
        infer(r#"
trait Debug {}

struct Foo<T>;

type E1<T> = (T, T, T);
type E2<T> = E1<E1<E1<(T, T, T)>>>;

impl Debug for Foo<E2<()>> {}

struct Request;

pub trait Future {
    type Output;
}

pub struct PeerSet<D>;

impl<D> Service<Request> for PeerSet<D>
where
    D: Discover,
    D::Key: Debug,
{
    type Error = ();
    type Future = dyn Future<Output = Self::Error>;

    fn call(&mut self) -> Self::Future {
        loop {}
    }
}

pub trait Discover {
    type Key;
}

pub trait Service<Request> {
    type Error;
    type Future: Future<Output = Self::Error>;
    fn call(&mut self) -> Self::Future;
}
"#),
        @r###"
    379..383 'self': &mut PeerSet<D>
    401..424 '{     ...     }': dyn Future<Output = ()>
    411..418 'loop {}': !
    416..418 '{}': ()
    575..579 'self': &mut Self
    "###
    );
}
