use crate::solidlang::ast::{ASTStatement, ASTStatementBlock, ASTStatementKind};
use crate::solidlang::lexer::{Token, TokenKind};
use crate::solidlang::parser::{Parser, ParserResult};

impl<'a, T: Iterator<Item = Token>> Parser<'a, T> {
    pub (in crate::solidlang::parser) fn parse_statement(&mut self) -> ParserResult<ASTStatement> {
        self.start_span();

        if self.check(TokenKind::Semicolon) {
            self.advance();
            return Ok(ASTStatement {
                kind: ASTStatementKind::Semicolon,
                span: self.close_span()
            });
        }

        if self.check(TokenKind::KwLet) {
            // Local binding
            self.advance();

            let name = self.expect_ident()?;

            // Type hint
            let type_hint = if self.check(TokenKind::Colon) {
                self.advance();
                Some(self.parse_type()?)
            }
            else {
                None
            };

            // Expression
            let expression = if self.check(TokenKind::Assign) {
                self.advance();
                Some(self.parse_expression()?)
            }
            else {
                None
            };

            return Ok(ASTStatement {
                kind: ASTStatementKind::LocalBinding(name, type_hint, expression),
                span: self.close_span()
            });
        }

        if self.check(TokenKind::KwReturn) {
            // Return
            self.advance();

            let expression = self.parse_expression()?;

            return Ok(ASTStatement {
                kind: ASTStatementKind::Return(expression),
                span: self.close_span()
            });
        }

        if self.check(TokenKind::KwBreak) {
            // Break
            self.advance();

            return Ok(ASTStatement { kind: ASTStatementKind::Break, span: self.close_span() });
        }

        if self.check(TokenKind::KwContinue) {
            // Break
            self.advance();

            return Ok(ASTStatement { kind: ASTStatementKind::Continue, span: self.close_span() });
        }

        if self.check(TokenKind::KwStruct)
            || self.check(TokenKind::KwFn)
            || self.check(TokenKind::KwTemplate) {
            return Ok(ASTStatement { kind: ASTStatementKind::Item(self.parse_item()?), span: self.close_span() });
        }

        Ok(ASTStatement {
            kind: ASTStatementKind::Expression(self.parse_expression()?),
            span: self.close_span()
        })
    }

    pub (in crate::solidlang::parser) fn parse_statement_block(&mut self) -> ParserResult<ASTStatementBlock> {
        self.start_span();

        self.expect(TokenKind::LCBracket)?;

        let mut statements = vec![];

        let mut require_semi = false;
        while !self.check(TokenKind::RCBracket) {
            if require_semi {
                self.expect(TokenKind::Semicolon)?;
            }

            if self.check(TokenKind::RCBracket) {
                break;
            }

            let statement = self.parse_statement()?;
            require_semi = statement.requires_semi();

            statements.push(statement);
        }
        self.advance();

        Ok(ASTStatementBlock {
            statements,
            span: self.close_span()
        })
    }
}