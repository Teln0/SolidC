use crate::globals::{SessionGlobals, Symbol};
use crate::solidlang::ast::ASTModule;
use crate::solidlang::lexer::{Token, TokenKind};
use crate::solidlang::span::Span;
use backtrace::Backtrace;
use std::iter::Peekable;

pub mod asttype;
pub mod expression;
pub mod item;
pub mod statement;

#[derive(Debug)]
pub enum ParserErrorKind {
    UnexpectedToken {
        expected: Vec<TokenKind>,
        got: TokenKind,
    },
}

#[derive(Debug)]
pub struct ParserError {
    pub kind: ParserErrorKind,
    pub backtrace: Backtrace,
}

pub type ParserResult<T> = Result<T, ParserError>;

pub struct Parser<'a, T: Iterator<Item = Token>> {
    tokens: Peekable<T>,
    expected_tokens: Vec<TokenKind>,
    src: &'a str,
    span_starts: Vec<usize>,
    ending_span: usize,
}

impl<'a, T: Iterator<Item = Token>> Parser<'a, T> {
    pub fn new(iter: T, src: &'a str) -> Self {
        Self {
            tokens: iter.peekable(),
            expected_tokens: vec![],
            src,
            span_starts: vec![],
            ending_span: 0,
        }
    }

    fn peek(&mut self) -> &Token {
        self.tokens.peek().unwrap()
    }

    fn check(&mut self, kind: TokenKind) -> bool {
        self.expected_tokens.push(kind);
        let p = self.peek();
        p.kind == kind
    }

    fn advance(&mut self) -> Token {
        self.expected_tokens.clear();
        let next = self.tokens.next().unwrap();
        self.ending_span = next.start + next.len;
        next
    }

    fn error_unexpected(&mut self, got: TokenKind) -> ParserError {
        ParserError {
            kind: ParserErrorKind::UnexpectedToken {
                got,
                expected: self.expected_tokens.clone(),
            },
            backtrace: Backtrace::new(),
        }
    }

    fn error_unexpected_current(&mut self) -> ParserError {
        let kind = self.peek().kind;
        self.error_unexpected(kind)
    }

    fn expect(&mut self, kind: TokenKind) -> ParserResult<Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error_unexpected_current())
        }
    }

    fn advance_symbol(&mut self) -> Symbol {
        let token = self.advance();
        SessionGlobals::with_interner_mut(|interner| {
            if token.kind == TokenKind::EOF || token.kind == TokenKind::Error {
                interner.intern("")
            } else {
                let str = &self.src[token.start..(token.start + token.len)];
                interner.intern(str)
            }
        })
    }

    fn expect_symbol(&mut self, kind: TokenKind) -> ParserResult<Symbol> {
        if self.check(kind) {
            Ok(self.advance_symbol())
        } else {
            Err(self.error_unexpected_current())
        }
    }

    fn expect_ident(&mut self) -> ParserResult<Symbol> {
        self.expect_symbol(TokenKind::Ident)
    }

    fn start_span(&mut self) {
        let start = self.peek().start;
        self.span_starts.push(start);
    }

    fn clone_span(&mut self) {
        let last = self.span_starts.last().unwrap().clone();
        self.span_starts.push(last);
    }

    fn close_span(&mut self) -> Span {
        let start = self.span_starts.pop().unwrap();
        Span {
            start,
            len: self.ending_span - start,
        }
    }

    pub fn parse_module(&mut self) -> ParserResult<ASTModule> {
        self.start_span();
        let items = self.parse_items(TokenKind::EOF)?;

        let result = Ok(ASTModule {
            items,
            span: self.close_span(),
        });

        if !self.span_starts.is_empty() {
            panic!("Span stack is not empty !");
        }

        result
    }
}
