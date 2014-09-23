#![feature(phase)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
fn main() {
    let tokenizer = create_tokenizer("(function($){'use strict';})(jQuery);");
    //for token in tokenizer {
        //println!("{}", token);
    //}
}

fn create_tokenizer(input: &str) -> Tokenizer {
    Tokenizer::new(input)
}

struct Tokenizer<'a> {
    input: &'a str,
    input_len: uint,
    tok_cur_line: uint,
    tok_pos: uint,
    tok_start: uint,
    tok_end: uint
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            input: input,
            input_len: input.len(),
            tok_pos: 0,
            tok_cur_line: 1,
            tok_start: 0,
            tok_end: 0
        }
    }

    fn init_token_state(&mut self) {
        self.skip_space();
    }

    fn skip_space(&mut self) {
        while self.tok_pos < self.input_len {
            let original_ch = self.input.char_at(self.tok_pos);
            let ch = original_ch as u32;
            if ch == 32 {
                self.tok_pos +=1;
            } else if ch == 13 {
                self.tok_pos +=1;
                let next = self.input.char_at(self.tok_pos) as u32;
                if next == 10 {
                    self.tok_pos +=1;
                }
            } else if ch == 10 || ch == 8232 || ch == 8233 {
                self.tok_pos +=1;
            } else if ch > 8 && ch < 18 {
                self.tok_pos +=1;
            } else if ch == 47 { // '/'
                let next = self.input.char_at(self.tok_pos + 1) as u32;
                if next == 42 { // '*'
                    self.skip_block_comment();
                } else if next == 47 { // '/'
                    self.skip_line_comment(2);
                } else {
                    break;
                }
            } else if ch == 160 { // '\xa0'
                self.tok_pos +=1;
            } else if ch >= 5760 && original_ch.is_whitespace() {
                self.tok_pos +=1;
            } else {
                break;
            }
        }
    }

    fn skip_block_comment(&mut self) {
        self.tok_pos +=2;
        match index_of_with_offset(self.input, "*/", self.tok_pos) {
            Some(i) => self.tok_pos = i + 2,
            None => fail!("Unterminated comment at: {}", self.tok_pos - 2)
        };
    }

    fn skip_line_comment(&mut self, start_skip: uint){
        self.tok_pos += start_skip;
        let mut ch = self.input.char_at(self.tok_pos) as u32;
        while(self.tok_pos < self.input_len && ch != 10 && ch != 13 && ch != 8232 && ch != 8233) {
            self.tok_pos += 1;
            ch = self.input.char_at(self.tok_pos) as u32;
        }
    }

    fn read_token<'a>(&mut self) -> Token<'a> {
        self.tok_start = self.tok_pos;
        // TODO: uncomment
        //if self.tok_pos >= self.input_len {
            //return Token {value: None, token_type: Eof, start: self.tok_start, end: self.tok_start}
        //}
        // TODO: remove
        return Token {value: None, token_type: Eof, start: self.tok_start, end: self.tok_start}

    }

    //TODO: return token maybe?
    fn finish_token(&mut self) {
        self.tok_end = self.tok_pos;
    }

    fn is_identifier_start(code: u32) -> bool {
        if code < 65 {return code == 36;}
        if code < 91 {return true;}
        if code < 97 {return code == 95;}
        if code < 123 {return true;}
        code >= 0xAA
    }
}

impl<'a> Iterator<Token<'a>> for Tokenizer<'a> {
    fn next(&mut self) -> Option<Token<'a>> {
        let token = self.read_token();
        match token.token_type {
            Eof => None,
            _ => Some(token)
        }
    }
}

#[deriving(Show)]
struct Token<'a> {
    value: Option<&'a str>,
    token_type: TokenType,
    start: uint,
    end: uint,
}

#[deriving(Show)]
enum TokenType {
    Identifier,
    Eof
}

fn index_of(haystack: &str, needle: &str) -> Option<uint> {
    haystack.find_str(needle)
}

fn index_of_with_offset(haystack: &str, needle: &str, offset: uint) -> Option<uint> {
    match index_of(haystack, needle.slice_from(offset)) {
        Some(index) => Some(index + offset),
        _ => None
    }
}
