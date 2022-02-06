use crate::solidlang::ast::{ASTFunctionDef, ASTItem, ASTItemKind, ASTStructDef, ASTTemplate};
use crate::solidlang::lexer::{Token, TokenKind};
use crate::solidlang::parser::{Parser, ParserResult};

impl<'a, T: Iterator<Item = Token>> Parser<'a, T> {
    pub (in crate::solidlang::parser) fn parse_item(&mut self) -> ParserResult<ASTItem> {
        self.start_span();

        if self.check(TokenKind::KwFn) {
            // Function def
            self.advance();
            let name = self.expect_ident()?;

            let mut params = vec![];
            self.expect(TokenKind::LParen)?;
            if !self.check(TokenKind::RParen) {
                params.push(self.parse_name_and_type()?);
                while self.check(TokenKind::Comma) {
                    self.advance();
                    params.push(self.parse_name_and_type()?);
                }
                self.expect(TokenKind::RParen)?;
            }
            else {
                self.advance();
            }

            let return_type = if self.check(TokenKind::Arrow) {
                self.advance();
                Some(self.parse_type()?)
            }
            else {
                None
            };

            let statement_block = self.parse_statement_block()?;

            return Ok(ASTItem {
                kind: ASTItemKind::FunctionDef(ASTFunctionDef {
                    name,
                    return_type,
                    params,
                    statement_block,
                    span: self.close_span()
                })
            })
        }

        if self.check(TokenKind::KwStruct) {
            // Struct def
            self.advance();

            let name = self.expect_ident()?;

            let mut fields = vec![];
            self.expect(TokenKind::LCBracket)?;
            if !self.check(TokenKind::RCBracket) {
                fields.push(self.parse_name_and_type()?);
                while self.check(TokenKind::Comma) {
                    self.advance();
                    fields.push(self.parse_name_and_type()?);
                }
                self.expect(TokenKind::RCBracket)?;
            }
            else {
                self.advance();
            }

            return Ok(ASTItem {
                kind: ASTItemKind::StructDef(ASTStructDef {
                    name,
                    fields,
                    span: self.close_span()
                })
            });
        }

        if self.check(TokenKind::KwTemplate) {
            // Template
            self.advance();
            self.expect(TokenKind::LABracket)?;
            let mut params = vec![self.expect_ident()?];
            while self.check(TokenKind::Comma) {
                self.advance();
                params.push(self.expect_ident()?);
            }
            self.expect(TokenKind::RABracket)?;

            let items;
            if self.check(TokenKind::LCBracket) {
                self.advance();
                items = self.parse_items(TokenKind::RCBracket)?;
                self.advance();
            }
            else {
                items = vec![self.parse_item()?];
            }

            return Ok(ASTItem {
                kind: ASTItemKind::Template(ASTTemplate {
                    params,
                    items,
                    span: self.close_span()
                })
            });
        }

        Err(self.error_unexpected_current())
    }

    pub (in crate::solidlang::parser) fn parse_items(&mut self, closing_delim: TokenKind) -> ParserResult<Vec<ASTItem>> {
        let mut items = vec![];
        while !self.check(closing_delim) {
            items.push(self.parse_item()?);
        }

        Ok(items)
    }
}