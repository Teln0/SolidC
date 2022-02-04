use solidc::globals::SessionGlobals;
use solidc::ir::assembly::assembler::assemble_ir_module;
use solidc::ir::interpreter::{IRInterpreter, IRInterpreterValue};

fn main() {
    SessionGlobals::create(|| {
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

        println!("{:?}", returned.bytes);
    });
}
