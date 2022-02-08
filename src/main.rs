use solidc::globals::SessionGlobals;
use solidc::ir::assembly::assembly_for_ir_modules;
use solidc::solidlang::lexer::lex;
use solidc::solidlang::lowerer::Lowerer;
use solidc::solidlang::parser::Parser;

fn main() {
    SessionGlobals::create(|| {
        /*
        let src = "\
        fn fib: %iterations := (1 1) -> (1 1)
            %constant_1 := const 1 1
            %constant_0 := const 1 0

            %sequence_numbers := alloc (2 1)
            %iterator := alloc (1 1)

            offsetstore (1 1) %sequence_numbers %constant_1 1
            offsetstore (1 1) %sequence_numbers %constant_0 0
            store (1 1) %iterator %constant_0

            ; While condition
            :while_condition

            %i := load (1 1) %iterator
            %comparison := binop >= %i %iterations
            if %comparison while_end

            ; While body

            %a := offsetload (1 1) %sequence_numbers 0
            %b := offsetload (1 1) %sequence_numbers 1
            %c := binop + %a %b
            offsetstore (1 1) %sequence_numbers %b 0
            offsetstore (1 1) %sequence_numbers %c 1

            %iplusplus := binop + %i %constant_1
            store (1 1) %iterator %iplusplus

            jmp while_condition

            ; While end
            :while_end

            %result := offsetload (1 1) %sequence_numbers 1
            return %result
        endfn
        ";
        let module = assemble_ir_module(src).unwrap();
        let mut interpreter = IRInterpreter::new();
        interpreter.load_module(module);
        let function_name = SessionGlobals::with_interner_mut(|i| i.intern("fib"));
        let returned = unsafe { interpreter.call_function(function_name, &[IRInterpreterValue::from_u8(10)]) };
        println!("{}", returned.into_u8());
        */

        let src = "
template<T>
struct Test {
    field_1: T,
    field_2: T,
    field_3: T
}

fn main(a: Test<u8>, b: Test<u16>) -> i32 {
    fn test(t: Test<u8>) -> i32 { 0 }
    fn test(t: Test<u16>) -> bool { true }

    if { return test(a); } {
        return test(a);
    }

    0
}
    ";

        let mut parser = Parser::new(lex(src), src);
        let module = parser.parse_module().unwrap();
        let lowerer = Lowerer::new();
        let ir_module = lowerer.lower(&module);

        println!("{}", assembly_for_ir_modules(&ir_module));
    });
}
