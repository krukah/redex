use examples::*;
use explore::*;
use rules::*;
use strategy::*;
use subst::*;
use syntax::*;
use term::*;

/// The cases of `test/engine.test.js`, ported one for one.
struct Engine;

impl Engine {
    fn run(src: &str, strategy: Strategy, cap: usize) -> Run {
        let p = Program::parse(&Source::from(src)).expect("parses");
        Run::new(p.term(), p.env(), strategy, Cap::new(cap))
    }

    fn src(id: &str) -> String {
        Example::find(id).expect("an example").src().to_string()
    }

    fn kinds(run: &Run) -> Vec<Kind> {
        run.rows().iter().skip(1).filter_map(strategy::Row::kind).collect()
    }

    fn uids(t: &Term) -> Vec<Uid> {
        std::iter::once(t.uid())
            .chain(t.kids().into_iter().flat_map(|(_, k)| Self::uids(k)))
            .collect()
    }
}

#[test]
fn every_example_and_race_term_parses() {
    Example::all().for_each(|e| assert!(Program::parse(&e.src()).is_ok(), "{}", e.id()));
    Race::ALL
        .iter()
        .filter_map(Race::src)
        .for_each(|s| assert!(Program::parse(&s).is_ok(), "{s}"));
}

#[test]
fn normal_forms_of_the_examples() {
    [
        ("alpha", "λy₁. y"),
        ("beta", "((a, a), (a, a))"),
        ("eta", "λz. g z"),
        ("delta", "a"),
        ("zeta", "(b, a)"),
        ("iota", "4"),
        ("proj", "(b, a)"),
        ("etas", "(a, b)"),
        ("diamond", "a a"),
        ("lazy", "z"),
        ("capture", "λy₁. λz₁. y z y₁ z₁"),
        ("church", "λf. λx. f (f (f (f x)))"),
        ("mix", "2"),
    ]
    .into_iter()
    .for_each(|(id, nf)| {
        let r = Engine::run(&Engine::src(id), Strategy::Normal, 200);
        assert_eq!(r.status(), Status::Done, "{id}");
        assert_eq!(r.last().text(), nf, "{id}");
    });
}

#[test]
fn omega_has_no_normal_form() {
    assert_eq!(Engine::run(&Engine::src("omega"), Strategy::Normal, 30).status(), Status::Cap);
}

#[test]
fn each_rule_fires_in_its_own_example() {
    assert_eq!(
        Engine::kinds(&Engine::run(&Engine::src("delta"), Strategy::Normal, 200)),
        [
            Kind::Delta,
            Kind::Beta,
            Kind::Beta,
            Kind::Delta,
            Kind::Beta,
            Kind::Delta,
            Kind::Beta
        ]
    );
    assert_eq!(
        Engine::kinds(&Engine::run(&Engine::src("zeta"), Strategy::Normal, 200)),
        [Kind::Zeta, Kind::Proj, Kind::Proj]
    );
    assert!(Engine::kinds(&Engine::run(&Engine::src("iota"), Strategy::Normal, 200)).contains(&Kind::Iota));
}

#[test]
fn strategies_duplicate_loop_and_stop_at_the_head() {
    let dup = "(λx. x x x) ((λy. y) a)";
    assert_eq!(Engine::run(dup, Strategy::Normal, 200).steps(), Step::new(4));
    assert_eq!(Engine::run(dup, Strategy::Applicative, 200).steps(), Step::new(2));
    assert_eq!(Engine::run(dup, Strategy::Whnf, 200).last().text(), "a ((λy. y) a) ((λy. y) a)");
    let lazy = "(λx. z) ((λw. w w) (λw. w w))";
    assert_eq!(Engine::run(lazy, Strategy::Normal, 200).last().text(), "z");
    assert_eq!(Engine::run(lazy, Strategy::Applicative, 24).status(), Status::Cap);
    assert_eq!(Engine::run("λz. (λx. x) ((λy. y) z)", Strategy::Whnf, 200).steps(), Step::START);
}

#[test]
fn beta_renames_binders_that_would_capture_and_naive_substitution_captures() {
    let t = Parser::parse("(λx. λy. x) y").expect("parses");
    let Shape::App(f, a) = t.shape() else {
        panic!("an application")
    };
    let Shape::Lam(x, b) = f.shape() else { panic!("a λ") };
    let mut log = Vec::new();
    let body = b.prepare(x, &a.fv(), &mut log);
    assert_eq!(log, [(Name::from("y"), Name::from("y₁"))]);
    assert_eq!(body.text(), "λy₁. x");
    assert_eq!(b.naive(x, a).text(), "λy. y");
}

#[test]
fn alpha_equivalent_terms_share_a_key() {
    let key = |s| Parser::parse(s).expect("parses").key();
    assert_eq!(key("λx. λy. x y"), key("λa. λb. a b"));
    assert_ne!(key("λx. λy. x"), key("λx. λy. y"));
}

#[test]
fn confluence_no_example_graph_has_two_normal_forms() {
    Example::all().for_each(|e| {
        let p = Program::parse(&e.src()).expect("parses");
        let g = Graph::explore(p.term(), p.env(), e.eta(), Cap::new(70));
        assert!(g.normal() <= 1, "{}", e.id());
    });
}

#[test]
fn parser_accepts_lean_syntax() {
    assert_eq!(Parser::parse("fun x y => x").expect("parses").text(), "λx. λy. x");
    assert_eq!(Parser::parse("let p := (a, b, c); p.2.1").expect("parses").text(), "let p := (a, (b, c)); p.2.1");
    assert_eq!(
        Program::parse(&Source::from("Nat.rec 0 (fun k r => Nat.succ r) 3"))
            .expect("parses")
            .term()
            .text(),
        "Nat.rec 0 (λk. λr. succ r) 3"
    );
    assert!(
        Parser::parse("λ. x")
            .expect_err("fails")
            .to_string()
            .contains("variable after λ")
    );
}

#[test]
fn provenance_beta_copies_point_back_to_the_argument() {
    let t = Parser::parse("(λf. f (f a)) (λy. y)").expect("parses");
    let r = t.contract(Kind::Beta, &Env::default());
    let at = |t: &Term, p: &str| t.at(&Path::from(p)).expect("a node").uid();
    assert_eq!(r.from(), Some(at(&t, "fb")));
    assert_eq!(r.kid('f').and_then(Term::from), Some(at(&t, "a")));
    assert_eq!(r.at(&Path::from("af")).and_then(Term::from), Some(at(&t, "a")));
    let names = Kind::Beta.bindings(&t).iter().map(Binding::var).collect::<Vec<Var>>();
    assert_eq!(names, Kind::Beta.schema(&t).vars());
}

#[test]
fn every_rule_binds_exactly_the_pattern_variables_its_schema_mentions() {
    Example::all().for_each(|e| {
        let p = Program::parse(&e.src()).expect("parses");
        p.term().redexes(p.env()).iter().for_each(|r| {
            let node = p.term().at(r.path()).expect("a redex inside its term");
            let vars = r.kind().schema(node).vars();
            r.kind()
                .bindings(node)
                .iter()
                .for_each(|b| assert!(vars.contains(&b.var()), "{} {:?} {}", e.id(), r.kind(), b.var()));
        });
    });
}

/// The workbench finds a node on screen by its uid, before and after a step, so no contraction may
/// leave one node in two places: a duplicated piece is a copy that records `from`.
#[test]
fn a_step_never_puts_one_node_in_two_places() {
    Example::all().for_each(|e| {
        let p = Program::parse(&e.src()).expect("parses");
        Run::new(p.term(), p.env(), Strategy::Normal, Cap::new(40))
            .rows()
            .iter()
            .flat_map(|row| {
                row.term()
                    .redexes(p.env())
                    .into_iter()
                    .map(|r| row.term().step(&r, p.env()))
            })
            .map(|t| Engine::uids(&t))
            .for_each(|uids| {
                assert_eq!(uids.iter().collect::<std::collections::HashSet<&Uid>>().len(), uids.len(), "{}", e.id());
            });
    });
}
