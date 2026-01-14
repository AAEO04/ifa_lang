use ifa_core::oracle::oracle::verify_equivalence;

#[test]
fn test_equivalence_arithmetic() {
    let source = r#"
    ayanmo a = 10;
    ayanmo b = 20;
    ayanmo c = a + b;
    Irosu.fo(c);
    
    ayanmo d = a * b;
    Irosu.fo(d);
    
    ayanmo e = b / a;
    Irosu.fo(e);
    
    ayanmo f = b - a;
    Irosu.fo(f);
    "#;
    verify_equivalence(source);
}

#[test]
fn test_equivalence_logic_and_branching() {
    let source = r#"
    ayanmo t = òtítọ́;
    ayanmo f = èké;
    
    if t {
        Irosu.fo("Truth prevails");
    }
    
    if f {
        Irosu.fo("Error: false is true");
    } else {
        Irosu.fo("False is false");
    }
    
    if t && f {
        Irosu.fo("Error: t AND f is true");
    }
    
    if t || f {
        Irosu.fo("t OR f is true");
    }
    "#;
    verify_equivalence(source);
}

#[test]
fn test_equivalence_loops() {
    let source = r#"
    ayanmo x = 0;
    while x < 5 {
        Irosu.fo(x);
        x = x + 1;
    }
    
    ayanmo list = [1, 2, 3];
    fun i ninu list {
        Irosu.fo(i);
    }
    "#;
    verify_equivalence(source);
}

#[test]
fn test_equivalence_nesting() {
    let source = r#"
    if òtítọ́ {
        if òtítọ́ {
            while 1 > 0 {
                Irosu.fo("Nested loop");
                ase;
            }
        }
    }
    "#;
    verify_equivalence(source);
}

#[test]
fn test_equivalence_recursion_fibonacci() {
    let source = r#"
    ese fib(n) {
        if n <= 1 {
            return n;
        }
        return fib(n - 1) + fib(n - 2);
    }
    
    ayanmo result = fib(10);
    Irosu.fo(result);
    "#;
    verify_equivalence(source);
}

#[cfg(feature = "proptest")]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_equivalence_random_arithmetic(a in -1000i64..1000i64, b in -1000i64..1000i64) {
            let ops = ["+", "-", "*", "/"];
            for op in ops {
                if op == "/" && b == 0 { continue; }
                let source = format!("Irosu.fo({} {} {});", a, op, b);
                verify_equivalence(&source);
            }
        }

        #[test]
        fn test_equivalence_random_logic(a in proptest::bool::ANY, b in proptest::bool::ANY) {
            let as_ifa = |v: bool| if v { "òtítọ́" } else { "èké" };
            let ops = ["&&", "||", "==", "!="];
            for op in ops {
                let source = format!("Irosu.fo({} {} {});", as_ifa(a), op, as_ifa(b));
                verify_equivalence(&source);
            }
        }
    }
}
