use crate::solidlang::ast::{ASTExpression, ASTExpressionKind, ASTOperator};
use crate::solidlang::lexer::{Token, TokenKind};
use crate::solidlang::parser::{Parser, ParserResult};

impl<'a, T: Iterator<Item = Token>> Parser<'a, T> {
    fn parse_primary_expression(&mut self) -> ParserResult<ASTExpression> {
        self.start_span();

        if self.check(TokenKind::Ident) {
            // Identifiers
            return Ok(ASTExpression {
                kind: ASTExpressionKind::Ident(self.advance_symbol()),
                span: self.close_span(),
            });
        }

        if self.check(TokenKind::IntegerLiteral) {
            // Integer literals
            let token = self.advance();
            let start = token.start;
            let end = token.start + token.len;
            let literal = &self.src[start..end];
            // FIXME : replace with more appropriate error
            let literal = literal
                .parse()
                .map_err(|_| self.error_unexpected_current())?;

            return Ok(ASTExpression {
                kind: ASTExpressionKind::IntegerLiteral(literal),
                span: self.close_span(),
            });
        }

        if self.check(TokenKind::BooleanTrue) {
            self.advance();

            return Ok(ASTExpression {
                kind: ASTExpressionKind::Boolean(true),
                span: self.close_span(),
            });
        }

        if self.check(TokenKind::BooleanFalse) {
            self.advance();

            return Ok(ASTExpression {
                kind: ASTExpressionKind::Boolean(false),
                span: self.close_span(),
            });
        }

        if self.check(TokenKind::LParen) {
            // Parenthesised expressions
            self.advance();
            let expression = self.parse_expression()?;
            self.expect(TokenKind::RParen)?;

            self.close_span();
            return Ok(expression);
        }

        if self.check(TokenKind::LCBracket) {
            // Block expressions
            let block = self.parse_statement_block()?;

            return Ok(ASTExpression {
                kind: ASTExpressionKind::Block(block),
                span: self.close_span(),
            });
        }

        if self.check(TokenKind::KwIf) {
            // Ifs and if-elses
            self.advance();
            let condition = self.parse_expression()?;
            let block = self.parse_statement_block()?;
            let else_block = if self.check(TokenKind::KwElse) {
                self.advance();
                Some(self.parse_statement_block()?)
            } else {
                None
            };

            return Ok(ASTExpression {
                kind: ASTExpressionKind::If(Box::new(condition), block, else_block),
                span: self.close_span(),
            });
        }

        if self.check(TokenKind::KwWhile) {
            // While loops
            self.advance();
            let condition = self.parse_expression()?;
            let block = self.parse_statement_block()?;

            return Ok(ASTExpression {
                kind: ASTExpressionKind::While(Box::new(condition), block),
                span: self.close_span(),
            });
        }

        if self.check(TokenKind::KwLoop) {
            // Loop blocks
            self.advance();
            let block = self.parse_statement_block()?;

            return Ok(ASTExpression {
                kind: ASTExpressionKind::Loop(block),
                span: self.close_span(),
            });
        }

        if self.check(TokenKind::KwFor) {
            // For loops
            self.advance();
            let var = self.expect_ident()?;
            self.expect(TokenKind::KwIn)?;
            let iter = self.parse_expression()?;
            let block = self.parse_statement_block()?;

            return Ok(ASTExpression {
                kind: ASTExpressionKind::For(var, Box::new(iter), block),
                span: self.close_span(),
            });
        }

        Err(self.error_unexpected_current())
    }

    fn parse_application_and_access(&mut self) -> ParserResult<ASTExpression> {
        self.start_span();
        let mut expression = self.parse_primary_expression()?;

        loop {
            if self.check(TokenKind::Dot) {
                // Member access
                self.clone_span();
                self.advance();
                let sym = self.expect_ident()?;
                expression = ASTExpression {
                    kind: ASTExpressionKind::MemberAccess(Box::new(expression), sym),
                    span: self.close_span(),
                };

                continue;
            }

            if self.check(TokenKind::ColonColon) {
                // Static access
                self.clone_span();
                self.advance();
                let sym = self.expect_ident()?;
                expression = ASTExpression {
                    kind: ASTExpressionKind::StaticAccess(Box::new(expression), sym),
                    span: self.close_span(),
                };

                continue;
            }

            if self.check(TokenKind::LTurbofish) {
                // Template application
                self.clone_span();
                self.advance();

                let mut args = vec![self.parse_type()?];
                while self.check(TokenKind::Comma) {
                    self.advance();
                    args.push(self.parse_type()?);
                }
                self.expect(TokenKind::RABracket)?;

                expression = ASTExpression {
                    kind: ASTExpressionKind::TemplateApplication(Box::new(expression), args),
                    span: self.close_span(),
                };

                continue;
            }

            if self.check(TokenKind::LParen) {
                // Function call
                self.clone_span();
                self.advance();

                let mut args = vec![];
                if !self.check(TokenKind::RParen) {
                    args.push(self.parse_expression()?);
                    while self.check(TokenKind::Comma) {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.expect(TokenKind::RParen)?;
                } else {
                    self.advance();
                }

                expression = ASTExpression {
                    kind: ASTExpressionKind::Call(Box::new(expression), args),
                    span: self.close_span(),
                };

                continue;
            }

            if self.check(TokenKind::LSBracket) {
                // Indexing
                self.clone_span();
                self.advance();
                let index = self.parse_expression()?;
                self.expect(TokenKind::RSBracket)?;

                expression = ASTExpression {
                    kind: ASTExpressionKind::Index(Box::new(expression), Box::new(index)),
                    span: self.close_span(),
                };

                continue;
            }

            break;
        }
        self.close_span();

        Ok(expression)
    }

    fn parse_unary_expression(&mut self) -> ParserResult<ASTExpression> {
        // TODO : tidy this up
        if self.check(TokenKind::BitNot) {
            self.start_span();
            self.advance();
            return Ok(ASTExpression {
                kind: ASTExpressionKind::UnaryOperation(
                    ASTOperator::BitNot,
                    Box::new(self.parse_unary_expression()?),
                ),
                span: self.close_span(),
            });
        }
        if self.check(TokenKind::BoolNot) {
            self.start_span();
            self.advance();
            return Ok(ASTExpression {
                kind: ASTExpressionKind::UnaryOperation(
                    ASTOperator::BoolNot,
                    Box::new(self.parse_unary_expression()?),
                ),
                span: self.close_span(),
            });
        }
        if self.check(TokenKind::Minus) {
            self.start_span();
            self.advance();
            return Ok(ASTExpression {
                kind: ASTExpressionKind::UnaryOperation(
                    ASTOperator::Minus,
                    Box::new(self.parse_unary_expression()?),
                ),
                span: self.close_span(),
            });
        }

        self.parse_application_and_access()
    }

    fn parse_binary_operation_with_precedence(
        &mut self,
        precedence: u8,
    ) -> ParserResult<ASTExpression> {
        self.start_span();
        let mut lhs = self.parse_unary_expression()?;

        while let Some((operator, p)) = self.check_operator() {
            if p < precedence {
                break;
            }
            self.clone_span();
            self.advance();

            let rhs = self.parse_binary_operation_with_precedence(p)?;
            lhs = ASTExpression {
                kind: ASTExpressionKind::BinaryOperation(operator, Box::new(lhs), Box::new(rhs)),
                span: self.close_span(),
            };
        }

        self.close_span();

        Ok(lhs)
    }

    fn check_operator(&mut self) -> Option<(ASTOperator, u8)> {
        // Member and static access are handled separately, as well as BitNot and BoolNot

        if self.check(TokenKind::Assign) {
            Some((ASTOperator::Assign, 0))
        } else if self.check(TokenKind::BitRShift) {
            Some((ASTOperator::BitRShift, 1))
        } else if self.check(TokenKind::BitLShift) {
            Some((ASTOperator::BitLShift, 1))
        } else if self.check(TokenKind::Equal) {
            Some((ASTOperator::Equal, 2))
        } else if self.check(TokenKind::NotEqual) {
            Some((ASTOperator::NotEqual, 2))
        } else if self.check(TokenKind::RABracket) {
            Some((ASTOperator::Greater, 2))
        } else if self.check(TokenKind::LABracket) {
            Some((ASTOperator::Lesser, 2))
        } else if self.check(TokenKind::GreaterEqual) {
            Some((ASTOperator::GreaterEqual, 2))
        } else if self.check(TokenKind::LesserEqual) {
            Some((ASTOperator::LesserEqual, 2))
        } else if self.check(TokenKind::BoolOr) {
            Some((ASTOperator::BoolOr, 3))
        } else if self.check(TokenKind::BoolAnd) {
            Some((ASTOperator::BoolAnd, 4))
        } else if self.check(TokenKind::BitOr) {
            Some((ASTOperator::BitOr, 5))
        } else if self.check(TokenKind::BitAnd) {
            Some((ASTOperator::BitAnd, 6))
        } else if self.check(TokenKind::Plus) {
            Some((ASTOperator::Plus, 7))
        } else if self.check(TokenKind::Minus) {
            Some((ASTOperator::Minus, 7))
        } else if self.check(TokenKind::Mul) {
            Some((ASTOperator::Mul, 8))
        } else if self.check(TokenKind::Div) {
            Some((ASTOperator::Div, 8))
        } else if self.check(TokenKind::Mod) {
            Some((ASTOperator::Mod, 8))
        } else {
            None
        }
    }

    pub fn parse_expression(&mut self) -> ParserResult<ASTExpression> {
        self.parse_binary_operation_with_precedence(0)
    }
}
