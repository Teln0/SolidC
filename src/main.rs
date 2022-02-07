use solidc::globals::SessionGlobals;
use solidc::ir::assembly::assembly_for_ir_modules;
use solidc::solidlang::lexer::lex;
use solidc::solidlang::lowerer::Lowerer;
use solidc::solidlang::parser::Parser;

fn main() {
    SessionGlobals::create(|| {
        /*
                let src = "\
        fn fib: (1 1) -> (1 1)
            ; Seq numbers
            alloc (2 1) ;                  %1

            ; Iterator
            alloc (1 1) ;                  %2

            ; Initialization
            const 1 0 ;                    %3
            const 1 1 ;                    %4
            offsetstore (1 1) %1 %3 0 ;    %5
            offsetstore (1 1) %1 %4 1 ;    %6
            store (1 1) %2 %3 ;            %7

            ; While start
            load (1 1) %2 ;                %8
            binop == %8 %0 ;               %9
            if %9 18 ;                     %10

            binop + %8 %4 ;                %11
            store (1 1) %2 %11 ;           %12

            offsetload (1 1) %1 0 ;        %13
            offsetload (1 1) %1 1 ;        %14
            binop + %13 %14 ;              %15

            offsetstore (1 1) %1 %14 0 ;   %16
            offsetstore (1 1) %1 %15 1 ;   %17
            jmp 7 ;                        %18
            ; While end

            offsetload (1 1) %1 0 ;        %19
            return %19 ;                   %20
        endfn
        ";
                let module = assemble_ir_module(src).unwrap();
                let mut interpreter = IRInterpreter::new();
                interpreter.load_module(module);
                let function_name = SessionGlobals::with_interner_mut(|i| i.intern("fib"));
                let returned = unsafe { interpreter.call_function(function_name, &[IRInterpreterValue::from_u8(10)]) };
                 */

        let src = "
template<A> {
    template<B, C>
    struct Test {
        field_1: A,
        field_2: B,
        field_3: C
    }

    struct Test2 {
        field_1: Test<A, A, A>
    }
}

struct Test3 {
    field_1: Test2<u8>,
    field_2: Test<u8, u8, u64>
}

fn other_function(a: u32) -> u32 { a }

fn main(a: u32, b: u32, c: u32) -> u32 {
    return b;
}
    ";

        let mut parser = Parser::new(lex(src), src);
        let module = parser.parse_module().unwrap();
        let mut lowerer = Lowerer::new();
        let ir_module = lowerer.lower(&module);

        println!("{}", assembly_for_ir_modules(&ir_module));
    });
}
