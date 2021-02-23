use radix_trie::Trie;

use super::helpers::{is_ident_continue, is_ident_end};
use super::{Keyword, Lexer, Token, TokenType};

use calypso_base::init_trie;
use calypso_base::streams::Stream;

init_trie!(pub KEYWORD_TRIE: Keyword => {
    "as"     => As,
    "break"  => Break,
    "case"   => Case,
    "del"    => Del,
    "do"     => Do,
    "else"   => Else,
    "end"    => End,
    "extern" => Extern,
    "false"  => False,
    "fn"     => Fn,
    "for"    => For,
    "if"     => If,
    "import" => Import,
    "in"     => In,
    "is"     => Is,
    "isa"    => Isa,
    "let"    => Let,
    "loop"   => Loop,
    "mod"    => Mod,
    "mut"    => Mut,
    "null"   => Null,
    "panic"  => Panic,
    "pub"    => Pub,
    "ret"    => Ret,
    "root"   => Root,
    "self"   => Zelf,
    "super"  => Super,
    "true"   => True,
    "try"    => Try,
    "while"  => While
});

impl<'lex> Lexer<'lex> {
    pub(super) fn handle_identifier(&mut self) -> Token<'lex> {
        let mut token_type = TokenType::Ident;

        // `_` is not an ident on its own, but all other [A-Za-z]{1} idents are.
        if self.prev().unwrap() == &'_' && self.peek_cond(is_ident_continue) != Some(true) {
            return self.new_token(TokenType::Under);
        }

        // Gorge while the character is a valid identifier character (and not an ident_end character).
        self.gorge_while(|sp, _| is_ident_continue(sp) && !is_ident_end(sp));

        // Allow `abc!`, `abc?`, and `abc!?` but not `abc?!`
        if self.peek_eq(&'!') == Some(true) {
            self.next();
        }
        if self.peek_eq(&'?') == Some(true) {
            self.next();
        }

        let keyword = KEYWORD_TRIE.get(&self.slice(self.new_span()).to_string());

        if let Some(&keyword) = keyword {
            token_type = TokenType::Keyword(keyword);
        }

        self.new_token(token_type)
    }
}
