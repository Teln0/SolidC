use crate::solidlang::ast::{ASTNameAndType, ASTType, ASTTypeKind};
use crate::solidlang::lexer::{Token, TokenKind};
use crate::solidlang::parser::{Parser, ParserResult};

impl<'a, T: Iterator<Item = Token>> Parser<'a, T> {
    pub(in crate::solidlang::parser) fn parse_name_and_type(
        &mut self,
    ) -> ParserResult<ASTNameAndType> {
        self.start_span();

        let name = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;
        let ast_type = self.parse_type()?;

        Ok(ASTNameAndType {
            name,
            ast_type,
            span: self.close_span(),
        })
    }

    pub(in crate::solidlang::parser) fn parse_type(&mut self) -> ParserResult<ASTType> {
        self.start_span();

        if self.check(TokenKind::Ident) {
            let mut symbols = vec![self.advance_symbol()];
            while self.check(TokenKind::ColonColon) {
                self.advance();
                symbols.push(self.expect_ident()?);
            }

            let mut generic_args = vec![];
            if self.check(TokenKind::LABracket) {
                self.advance();
                generic_args.push(self.parse_type()?);
                while !self.check(TokenKind::RABracket) {
                    self.expect(TokenKind::Comma)?;
                    generic_args.push(self.parse_type()?);
                }
                self.advance();
            }

            return Ok(ASTType {
                kind: ASTTypeKind::Path {
                    symbols,
                    generic_args,
                },
                span: self.close_span(),
            });
        }

        if self.check(TokenKind::Mul) {
            self.advance();
            return Ok(ASTType {
                kind: ASTTypeKind::PointerTo(Box::new(self.parse_type()?)),
                span: self.close_span(),
            });
        }

        Err(self.error_unexpected_current())
    }
}
