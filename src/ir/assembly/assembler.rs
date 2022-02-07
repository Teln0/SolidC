use crate::globals::SessionGlobals;
use crate::ir::comp::{
    IRComp, IRCompBinaryOperation, IRCompBinaryOperationKind, IRCompConstant, IRCompFunctionCall,
    IRCompKind, IRCompUnaryOperation, IRCompUnaryOperationKind,
};
use crate::ir::{IRItem, IRItemFunctionDef, IRItemKind, IRModule, IRType, IRValue};
use std::iter::Peekable;
use std::str::Chars;

const EOF_CHAR: char = '\0';

struct IRAsssmblyLexerCursor<'a> {
    initial_len: usize,
    lines_consumed: usize,
    chars: Chars<'a>,
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || c == '_'
        || c == '+'
        || c == '-'
        || c == '*'
        || c == '/'
        || c == '='
        || c == '!'
        || c == '<'
        || c == '>'
        || c == '&'
        || c == '|'
}

impl<'a> IRAsssmblyLexerCursor<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            initial_len: input.len(),
            lines_consumed: 0,
            chars: input.chars(),
        }
    }

    fn consumed(&self) -> usize {
        self.initial_len - self.chars.as_str().len()
    }

    fn bump(&mut self) -> char {
        let c = self.chars.next().unwrap_or(EOF_CHAR);
        if c == '\n' {
            self.lines_consumed += 1
        }
        c
    }

    fn nth(&self, n: usize) -> char {
        self.chars.clone().nth(n).unwrap_or(EOF_CHAR)
    }

    fn next_token(&mut self) -> IRAssemblyThinToken {
        while self.nth(0).is_ascii_whitespace() {
            self.bump();
        }

        while self.nth(0) == ';' {
            loop {
                let c = self.bump();
                if c == '\n' || c == EOF_CHAR {
                    break;
                }
            }

            while self.nth(0).is_ascii_whitespace() {
                self.bump();
            }
        }

        let offset = self.consumed();

        if self.nth(0) == EOF_CHAR {
            return IRAssemblyThinToken {
                kind: IRAssemblyTokenKind::Eof,
                offset,
            };
        }

        let kind = match self.bump() {
            '%' => IRAssemblyTokenKind::Percent,
            ':' => IRAssemblyTokenKind::Colon,
            '-' if self.nth(0) == '>' => {
                self.bump();
                IRAssemblyTokenKind::Arrow
            }
            '(' => IRAssemblyTokenKind::LParen,
            ')' => IRAssemblyTokenKind::RParen,

            c if c.is_ascii_digit() => {
                while self.nth(0).is_ascii_digit() {
                    self.bump();
                }
                if is_word_char(self.nth(0)) {
                    IRAssemblyTokenKind::UnexpectedCharacter
                } else {
                    IRAssemblyTokenKind::Integer
                }
            }
            _ => {
                while is_word_char(self.nth(0)) {
                    self.bump();
                }
                IRAssemblyTokenKind::Word
            }
        };

        IRAssemblyThinToken { kind, offset }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum IRAssemblyTokenKind {
    Word,
    Integer,
    Percent,
    Colon,
    Arrow,
    LParen,
    RParen,

    UnexpectedCharacter,

    Eof,
}

struct IRAssemblyThinToken {
    kind: IRAssemblyTokenKind,
    offset: usize,
}

#[derive(Debug)]
struct IRAssemblyToken {
    kind: IRAssemblyTokenKind,
    start: usize,
    len: usize,
    line: usize,
}

fn assembly_token_stream(src: &str) -> impl Iterator<Item = IRAssemblyToken> + '_ {
    let mut cursor = IRAsssmblyLexerCursor::new(src);
    let mut current_pos = 0;
    let mut current_line = 0;

    std::iter::from_fn(move || {
        let token = cursor.next_token();
        let consumed = cursor.consumed();
        let lines_consumed = cursor.lines_consumed;
        let token = IRAssemblyToken {
            kind: token.kind,
            start: current_pos + token.offset,
            len: cursor.consumed() - token.offset,
            line: current_line,
        };
        current_pos += consumed;
        current_line += lines_consumed;
        cursor = IRAsssmblyLexerCursor::new(&src[current_pos..]);

        Some(token)
    })
}

#[derive(Debug)]
pub struct IRAssemblerError {
    pub message: String,
    pub start: usize,
    pub line: usize,
}

#[derive(Debug)]
enum IRAssemblerExpected<'a> {
    Keyword(&'a str),
    Kind(IRAssemblyTokenKind),
}

type IRAssemblerResult<T> = Result<T, IRAssemblerError>;

struct IRAssembler<'a, T: Iterator<Item = IRAssemblyToken>> {
    token_stream: Peekable<T>,
    src: &'a str,
    expected: Vec<IRAssemblerExpected<'a>>,
}

impl<'a, T: Iterator<Item = IRAssemblyToken>> IRAssembler<'a, T> {
    fn new(token_stream: T, src: &'a str) -> Self {
        Self {
            token_stream: token_stream.peekable(),
            src,
            expected: vec![],
        }
    }

    fn advance_token(&mut self) {
        self.expected.clear();
        self.token_stream.next();
    }

    fn check_kind(&mut self, kind: IRAssemblyTokenKind) -> bool {
        self.expected.push(IRAssemblerExpected::Kind(kind));
        let token = self.token_stream.peek().unwrap();
        token.kind == kind
    }

    fn expect_kind(&mut self, kind: IRAssemblyTokenKind) -> IRAssemblerResult<IRAssemblyToken> {
        self.expected.push(IRAssemblerExpected::Kind(kind));
        let token = self.token_stream.next().unwrap();
        if token.kind == kind {
            self.expected.clear();
            Ok(token)
        } else {
            Err(IRAssemblerError {
                message: format!(
                    "Expected \"{:?}\" got \"{}\"",
                    self.expected,
                    &self.src[token.start..(token.start + token.len)]
                ),
                start: token.start,
                line: token.line,
            })
        }
    }

    fn check_keyword(&mut self, kw: &'a str) -> bool {
        self.expected.push(IRAssemblerExpected::Keyword(kw));
        let token = self.token_stream.peek().unwrap();
        token.kind == IRAssemblyTokenKind::Word
            && self.src[token.start..(token.start + token.len)].eq(kw)
    }

    fn get_token_string(&self, token: &IRAssemblyToken) -> &'a str {
        &self.src[token.start..(token.start + token.len)]
    }

    fn error_unexpected(&mut self) -> IRAssemblerError {
        let token = self.token_stream.next().unwrap();
        IRAssemblerError {
            message: format!(
                "Expected \"{:?}\" got \"{}\"",
                self.expected,
                &self.src[token.start..(token.start + token.len)]
            ),
            start: token.start,
            line: token.line,
        }
    }

    fn parse_integer_u64(&mut self) -> IRAssemblerResult<u64> {
        let token = self.expect_kind(IRAssemblyTokenKind::Integer)?;
        self.get_token_string(&token)
            .parse()
            .map_err(|_| IRAssemblerError {
                message: format!("Expected integer to fit into 64 bits"),
                start: token.start,
                line: token.line,
            })
    }

    fn parse_integer_u8(&mut self) -> IRAssemblerResult<u8> {
        let token = self.expect_kind(IRAssemblyTokenKind::Integer)?;
        self.get_token_string(&token)
            .parse()
            .map_err(|_| IRAssemblerError {
                message: format!("Expected integer to fit into 8 bits"),
                start: token.start,
                line: token.line,
            })
    }

    fn parse_ir_type(&mut self) -> IRAssemblerResult<IRType> {
        self.expect_kind(IRAssemblyTokenKind::LParen)?;
        let size = self.parse_integer_u64()?;
        let align = self.parse_integer_u64()?;
        self.expect_kind(IRAssemblyTokenKind::RParen)?;

        return Ok(IRType { size, align });
    }

    fn parse_ir_value(&mut self) -> IRAssemblerResult<IRValue> {
        self.expect_kind(IRAssemblyTokenKind::Percent)?;
        Ok(IRValue {
            index: self.parse_integer_u64()?,
        })
    }

    fn parse_ir_comp(&mut self) -> IRAssemblerResult<IRComp> {
        if self.check_keyword("call") {
            // Function call
            self.advance_token();

            // Name
            let name = self.expect_kind(IRAssemblyTokenKind::Word)?;
            let name = self.get_token_string(&name);
            let name = SessionGlobals::with_interner_mut(|i| i.intern(name));

            // Args
            let args_amount = self.parse_integer_u64()?;
            let mut args = vec![];
            for _ in 0..args_amount {
                args.push(self.parse_ir_value()?);
            }

            return Ok(IRComp {
                kind: IRCompKind::FunctionCall(IRCompFunctionCall { name, args }),
            });
        }
        if self.check_keyword("binop") {
            // Binary operation
            self.advance_token();

            let mut operation_kind = None;

            if self.check_keyword("+") {
                operation_kind = Some(IRCompBinaryOperationKind::Plus)
            } else if self.check_keyword("-") {
                operation_kind = Some(IRCompBinaryOperationKind::Minus)
            } else if self.check_keyword("*") {
                operation_kind = Some(IRCompBinaryOperationKind::Mul)
            } else if self.check_keyword("/") {
                operation_kind = Some(IRCompBinaryOperationKind::Div)
            } else if self.check_keyword("mod") {
                operation_kind = Some(IRCompBinaryOperationKind::Mod)
            } else if self.check_keyword("&") {
                operation_kind = Some(IRCompBinaryOperationKind::BitAnd)
            } else if self.check_keyword("|") {
                operation_kind = Some(IRCompBinaryOperationKind::BitOr)
            } else if self.check_keyword("<<") {
                operation_kind = Some(IRCompBinaryOperationKind::BitLShift)
            } else if self.check_keyword(">>") {
                operation_kind = Some(IRCompBinaryOperationKind::BitRShift)
            } else if self.check_keyword("==") {
                operation_kind = Some(IRCompBinaryOperationKind::Equal)
            } else if self.check_keyword("!=") {
                operation_kind = Some(IRCompBinaryOperationKind::NotEqual)
            } else if self.check_keyword("<") {
                operation_kind = Some(IRCompBinaryOperationKind::Lesser)
            } else if self.check_keyword(">") {
                operation_kind = Some(IRCompBinaryOperationKind::Greater)
            } else if self.check_keyword("<=") {
                operation_kind = Some(IRCompBinaryOperationKind::LesserEqual)
            } else if self.check_keyword(">=") {
                operation_kind = Some(IRCompBinaryOperationKind::GreaterEqual)
            }
            self.advance_token();

            let left_operand = self.parse_ir_value()?;
            let right_operand = self.parse_ir_value()?;

            if let Some(operation_kind) = operation_kind {
                return Ok(IRComp {
                    kind: IRCompKind::BinaryOperation(IRCompBinaryOperation {
                        kind: operation_kind,
                        left_operand,
                        right_operand,
                    }),
                });
            } else {
                return Err(self.error_unexpected());
            }
        }
        if self.check_keyword("unop") {
            // Unary operation
            self.advance_token();
            let mut operation_kind = None;

            if self.check_keyword("not") {
                operation_kind = Some(IRCompUnaryOperationKind::BoolNot)
            } else if self.check_keyword("neg") {
                operation_kind = Some(IRCompUnaryOperationKind::SignedNegation)
            }

            let operand = self.parse_ir_value()?;

            if let Some(operation_kind) = operation_kind {
                return Ok(IRComp {
                    kind: IRCompKind::UnaryOperation(IRCompUnaryOperation {
                        kind: operation_kind,
                        operand,
                    }),
                });
            } else {
                return Err(self.error_unexpected());
            }
        }
        if self.check_keyword("const") {
            // Constant
            self.advance_token();

            let len = self.parse_integer_u64()?;
            let mut bytes = vec![];
            for _ in 0..len {
                bytes.push(self.parse_integer_u8()?);
            }

            return Ok(IRComp {
                kind: IRCompKind::Constant(IRCompConstant { bytes }),
            });
        }
        if self.check_keyword("alloc") {
            // Stack allocation
            self.advance_token();

            let ir_type = self.parse_ir_type()?;
            return Ok(IRComp {
                kind: IRCompKind::Alloc(ir_type),
            });
        }
        if self.check_keyword("store") {
            // Store in pointer
            self.advance_token();

            let ir_type = self.parse_ir_type()?;
            let location = self.parse_ir_value()?;
            let value = self.parse_ir_value()?;

            return Ok(IRComp {
                kind: IRCompKind::Store(ir_type, location, value),
            });
        }
        if self.check_keyword("load") {
            // Load from pointer
            self.advance_token();

            let ir_type = self.parse_ir_type()?;
            let location = self.parse_ir_value()?;

            return Ok(IRComp {
                kind: IRCompKind::Load(ir_type, location),
            });
        }
        if self.check_keyword("offsetstore") {
            // Store in pointer
            self.advance_token();

            let ir_type = self.parse_ir_type()?;
            let location = self.parse_ir_value()?;
            let value = self.parse_ir_value()?;
            let offset = self.parse_integer_u64()?;

            return Ok(IRComp {
                kind: IRCompKind::OffsetStore(ir_type, location, value, offset),
            });
        }
        if self.check_keyword("offsetload") {
            // Load from pointer
            self.advance_token();

            let ir_type = self.parse_ir_type()?;
            let location = self.parse_ir_value()?;
            let offset = self.parse_integer_u64()?;

            return Ok(IRComp {
                kind: IRCompKind::OffsetLoad(ir_type, location, offset),
            });
        }
        if self.check_keyword("return") {
            // Return from function
            self.advance_token();

            let value = self.parse_ir_value()?;

            return Ok(IRComp {
                kind: IRCompKind::Return(value),
            });
        }
        if self.check_keyword("if") {
            // If branching
            self.advance_token();

            let value = self.parse_ir_value()?;
            let location = self.parse_integer_u64()?;

            return Ok(IRComp {
                kind: IRCompKind::If(value, location),
            });
        }
        if self.check_keyword("jmp") {
            // Jump
            self.advance_token();

            let location = self.parse_integer_u64()?;

            return Ok(IRComp {
                kind: IRCompKind::Jmp(location),
            });
        }

        Err(self.error_unexpected())
    }

    fn parse_ir_module(&mut self) -> IRAssemblerResult<IRModule> {
        let mut items = vec![];

        loop {
            if self.check_keyword("fn") {
                // Function definition
                self.advance_token();

                let name = self.expect_kind(IRAssemblyTokenKind::Word)?;
                let name = self.get_token_string(&name);

                self.expect_kind(IRAssemblyTokenKind::Colon)?;

                // Params
                let mut params = vec![];
                while !self.check_kind(IRAssemblyTokenKind::Arrow) {
                    params.push(self.parse_ir_type()?);
                }
                self.advance_token();

                let return_type = self.parse_ir_type()?;

                // Computations
                let mut comps = vec![];
                while !self.check_keyword("endfn") {
                    comps.push(self.parse_ir_comp()?);
                }
                self.advance_token();

                items.push(IRItem {
                    kind: IRItemKind::FunctionDef(IRItemFunctionDef {
                        name: SessionGlobals::with_interner_mut(|i| i.intern(name)),
                        params,
                        return_type,
                        comps,
                    }),
                });

                continue;
            }

            if self.check_kind(IRAssemblyTokenKind::Eof) {
                return Ok(IRModule { items });
            }

            return Err(self.error_unexpected());
        }
    }
}

pub fn assemble_ir_module(src: &str) -> Result<IRModule, IRAssemblerError> {
    let token_stream = assembly_token_stream(src);
    let mut assembler = IRAssembler::new(token_stream, src);
    assembler.parse_ir_module()
}
