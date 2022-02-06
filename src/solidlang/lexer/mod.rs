use std::str::Chars;

const EOF_CHAR: char = '\0';

struct LexerCursor<'a> {
    initial_len: usize,
    chars: Chars<'a>
}

impl<'a> LexerCursor<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            initial_len: input.len(),
            chars: input.chars(),
        }
    }

    fn consumed(&self) -> usize {
        self.initial_len - self.chars.as_str().len()
    }

    fn bump(&mut self) -> char {
        self.chars.next().unwrap_or(EOF_CHAR)
    }

    fn nth(&self, n: usize) -> char {
        self.chars.clone().nth(n).unwrap_or(EOF_CHAR)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Identifiers

    Ident,

    // Literals

    IntegerLiteral,

    // Keywords

    KwFn,
    KwStruct,
    KwTemplate,
    KwLet,
    KwIf,
    KwElse,
    KwWhile,
    KwFor,
    KwIn,
    KwLoop,
    KwReturn,
    KwBreak,
    KwContinue,

    // Punctuation

    Semicolon,
    Colon,
    ColonColon,
    Arrow,
    Dot,
    Comma,
    LTurbofish,

    // Parentheses and brackets

    LParen,
    RParen,
    LSBracket,
    RSBracket,
    LCBracket,
    RCBracket,
    LABracket,
    RABracket,

    // Assignment & operators

    Assign,

    Plus,
    Minus,
    Mul,
    Div,
    Mod,

    BitAnd,
    BitOr,
    BitNot,
    BitRShift,
    BitLShift,

    BoolAnd,
    BoolOr,
    BoolNot,

    Equal,
    NotEqual,
    GreaterEqual,
    LesserEqual,

    // Special tokens
    EOF,
    Whitespace,
    Error
}

struct ThinToken {
    kind: TokenKind,
    len: usize
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub len: usize
}

fn is_valid_ident_char<const FIRST: bool>(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || (!FIRST && c.is_ascii_digit())
}

fn first_token(src: &str) -> ThinToken {
    let mut cursor = LexerCursor::new(src);

    let kind = match cursor.bump() {
        ';' => TokenKind::Semicolon,
        ':' => if cursor.nth(0) == ':' {
            cursor.bump();
            if cursor.nth(0) == '<' {
                cursor.bump();
                TokenKind::LTurbofish
            }
            else {
                TokenKind::ColonColon
            }
        }
        else { TokenKind::Colon }
        '.' => TokenKind::Dot,
        ',' => TokenKind::Comma,

        '(' => TokenKind::LParen,
        ')' => TokenKind::RParen,
        '[' => TokenKind::LSBracket,
        ']' => TokenKind::RSBracket,
        '{' => TokenKind::LCBracket,
        '}' => TokenKind::RCBracket,
        '<' => if cursor.nth(0) == '~' { cursor.bump(); TokenKind::BitLShift }
        else if cursor.nth(0) == '=' { cursor.bump(); TokenKind::LesserEqual }
        else { TokenKind::LABracket },
        '>' => if cursor.nth(0) == '=' { cursor.bump(); TokenKind::GreaterEqual }
        else { TokenKind::RABracket },

        '=' => if cursor.nth(0) == '=' { cursor.bump(); TokenKind::Equal }
        else { TokenKind::Assign }
        '+' => TokenKind::Plus,
        '-' => if cursor.nth(0) == '>' { cursor.bump(); TokenKind::Arrow }
        else { TokenKind::Minus }
        '*' => TokenKind::Mul,
        '/' => TokenKind::Div,
        '%' => TokenKind::Mod,

        '&' => if cursor.nth(0) == '&' { cursor.bump(); TokenKind::BoolAnd }
        else { TokenKind::BitAnd }
        '|' => if cursor.nth(0) == '|' { cursor.bump(); TokenKind::BoolOr }
        else { TokenKind::BitOr }
        '~' => if cursor.nth(0) == '>' { cursor.bump(); TokenKind::BitRShift }
        else { TokenKind::BitNot },
        '!' => if cursor.nth(0) == '=' { cursor.bump(); TokenKind::NotEqual }
        else { TokenKind::BoolNot },

        c if is_valid_ident_char::<true>(c) => {
            while is_valid_ident_char::<false>(cursor.nth(0)) {
                cursor.bump();
            }

            match &src[..cursor.consumed()] {
                "fn" => TokenKind::KwFn,
                "struct" => TokenKind::KwStruct,
                "template" => TokenKind::KwTemplate,
                "let" => TokenKind::KwLet,
                "if" => TokenKind::KwIf,
                "else" => TokenKind::KwElse,
                "while" => TokenKind::KwWhile,
                "for" => TokenKind::KwFor,
                "in" => TokenKind::KwIn,
                "loop" => TokenKind::KwLoop,
                "return" => TokenKind::KwReturn,
                "break" => TokenKind::KwBreak,
                "continue" => TokenKind::KwContinue,
                _ => TokenKind::Ident
            }
        }
        EOF_CHAR => TokenKind::EOF,
        c if c.is_ascii_digit() => {
            while cursor.nth(0).is_ascii_digit() {
                cursor.bump();
            }
            TokenKind::IntegerLiteral
        }
        c if c.is_ascii_whitespace() => {
            while cursor.nth(0).is_ascii_whitespace() {
                cursor.bump();
            }
            TokenKind::Whitespace
        }
        _ => TokenKind::Error
    };

    ThinToken {
        kind,
        len: cursor.consumed()
    }
}

pub fn lex(mut src: &str) -> impl Iterator<Item = Token> + '_ {
    let mut consumed = 0;
    std::iter::from_fn(move || {
        Some(loop {
            let token = first_token(src);
            let start = consumed;
            consumed += token.len;

            if token.kind != TokenKind::EOF { src = &src[token.len..]; }

            // Skip whitespace
            if token.kind == TokenKind::Whitespace { continue; }

            break Token {
                kind: token.kind,
                len: token.len,
                start
            };
        })
    })
}