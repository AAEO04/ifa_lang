use ifa_core::compiler::Compiler;
use ifa_core::parser::parse;
use ifa_types::OpCode;

fn disassemble_opcodes(code: &[u8]) -> Vec<OpCode> {
    let mut out = Vec::new();
    let mut ip = 0usize;
    while ip < code.len() {
        let op = OpCode::from_u8(code[ip]).expect("invalid opcode byte");
        out.push(op);
        ip += 1;

        let operand_bytes = op
            .operand_bytes()
            .expect("variable-length opcode in test disassembly");
        ip += operand_bytes;
    }
    out
}

#[test]
fn return_direct_call_emits_tailcall() {
    let source = r#"
    ese g(x) {
        pada x + 1;
    }

    ese f(x) {
        pada g(x);
    }

    ayanmo y = f(41);
    "#;

    let program = parse(source).expect("parse failed");
    let compiler = Compiler::new("tailcall_test");
    let bytecode = compiler.compile(&program).expect("compile failed");

    let ops = disassemble_opcodes(&bytecode.code);
    assert!(
        ops.iter().any(|op| *op == OpCode::TailCall),
        "expected TailCall to be emitted somewhere; ops={ops:?}"
    );
}
