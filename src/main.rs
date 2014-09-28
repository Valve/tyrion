/*
 * JavaScript tokenizer/parser
 * Copyright Valentin Vasilyev (valentin.vasilyev@outlook.com) 2014
 * https://github.com/Valve/tyrion
 * Ideas heavily borrowed from other JavaScript analyzers/parsers
 * (esprima/acorn/uglifyjs/typescript compiler)
 * Code is MIT licensed
 */

//TODO: remove this line later
#![allow(dead_code)]
#![feature(phase)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

use std::char;

static STRICT_MODE_BAD_WORDS: [&'static str,..2] = ["eval", "arguments"];
static ECMA3_RESERVED_WORDS: [&'static str,..29] = ["abstract", "boolean", "byte", "char", "class", "double", "enum", "export", "extends", "final", "float", "goto", "implements", "import", "int", "interface", "long", "native", "package", "private", "protected", "public", "short", "static", "super", "synchronized", "throws", "transient", "volatile"];
static ECMA5_RESERVED_WORDS: [&'static str,..7] = ["class", "enum", "extends", "super", "const", "export", "import"];
static ECMA5_KEYWORDS: [&'static str,..29] = ["break", "case", "catch", "continue", "debugger", "default", "do", "else", "finally", "for", "function", "if", "return", "switch", "throw", "try", "var", "while", "with", "null", "true", "false", "instanceof", "typeof", "void", "delete", "new", "in", "this"];
static ECMA6_KEYWORDS: [&'static str,..36] = ["break", "case", "catch", "continue", "debugger", "default", "do", "else", "finally", "for", "function", "if", "return", "switch", "throw", "try", "var", "while", "with", "null", "true", "false", "instanceof", "typeof", "void", "delete", "new", "in", "this", "let", "const", "class", "extends", "export", "import", "yield"];

// keyword data
static BREAK: KeywordData = KeywordData { keyword: "break", is_loop: false, before_expr: false };
static CASE: KeywordData = KeywordData { keyword: "case", is_loop: false, before_expr: true };
static CATCH: KeywordData = KeywordData { keyword: "catch", is_loop: false, before_expr: false };
static CLASS: KeywordData = KeywordData { keyword: "class", is_loop: false, before_expr: false };
static CONST: KeywordData = KeywordData { keyword: "const", is_loop: false, before_expr: false };
static CONTINUE: KeywordData = KeywordData { keyword: "continue", is_loop: false, before_expr: false };
static DEBUGGER: KeywordData = KeywordData { keyword: "debugger", is_loop: false, before_expr: false };
static DEFAULT: KeywordData = KeywordData { keyword: "default", is_loop: false, before_expr: false };
static DO: KeywordData = KeywordData { keyword: "do", is_loop: true, before_expr: false };
static ELSE: KeywordData = KeywordData { keyword: "else", is_loop: false, before_expr: true };
static EXPORT: KeywordData = KeywordData { keyword: "export", is_loop: false, before_expr: false };
static EXTENDS: KeywordData = KeywordData { keyword: "extends", is_loop: false, before_expr: true };
static FINALLY: KeywordData = KeywordData { keyword: "finally", is_loop: false, before_expr: false };
static IMPORT: KeywordData = KeywordData { keyword: "import", is_loop: false, before_expr: false };
static FOR: KeywordData = KeywordData { keyword: "for", is_loop: true, before_expr: false };
static FUNCTION: KeywordData = KeywordData { keyword: "function", is_loop: false, before_expr: false };
static IF: KeywordData = KeywordData { keyword: "if", is_loop: false, before_expr: false };
static LET: KeywordData = KeywordData { keyword: "let", is_loop: false, before_expr: false };
static NEW: KeywordData = KeywordData { keyword: "new", is_loop: false, before_expr: true };
static RETURN: KeywordData = KeywordData { keyword: "return", is_loop: false, before_expr: true };
static SWITCH: KeywordData = KeywordData { keyword: "switch", is_loop: false, before_expr: false };
static THIS: KeywordData = KeywordData { keyword: "this", is_loop: false, before_expr: false };
static THROW: KeywordData = KeywordData { keyword: "throw", is_loop: false, before_expr: true };
static TRY: KeywordData = KeywordData { keyword: "try", is_loop: false, before_expr: false };
static VAR: KeywordData = KeywordData { keyword: "var", is_loop: false, before_expr: false };
static WHILE: KeywordData = KeywordData { keyword: "while", is_loop: true, before_expr: false };
static WITH: KeywordData = KeywordData { keyword: "with", is_loop: false, before_expr: false };
static YIELD: KeywordData = KeywordData { keyword: "yield", is_loop: false, before_expr: true };

// values
static NULL: ValueData = ValueData { keyword: "null", atom_value: None };
static TRUE: ValueData = ValueData { keyword: "true", atom_value: Some(true) };
static FALSE: ValueData = ValueData { keyword: "false", atom_value: Some(false) };

// punc data
static ARROW: PuncData = PuncData { punc_type: "=>", before_expr: true };
static BQUOTE: PuncData = PuncData { punc_type: "`", before_expr: false };
static BRAKET_L: PuncData = PuncData { punc_type: "[", before_expr: true };
static BRAKET_R: PuncData = PuncData { punc_type: "]", before_expr: false };
static BRACE_L: PuncData = PuncData { punc_type: "{", before_expr: true };
static BRACE_R: PuncData = PuncData { punc_type: "}", before_expr: false };
static COLON: PuncData = PuncData { punc_type: ":", before_expr: true };
static COMMA: PuncData = PuncData { punc_type: ",", before_expr: true };
static DOLLAR_BRACE_L: PuncData = PuncData { punc_type: "${", before_expr: true };
static DOT: PuncData = PuncData { punc_type: ".", before_expr: false };
static ELLIPSIS: PuncData = PuncData { punc_type: "...", before_expr: false };
static PAREN_L: PuncData = PuncData { punc_type: "(", before_expr: true };
static PAREN_R: PuncData = PuncData { punc_type: ")", before_expr: false };
static QUESTION: PuncData = PuncData { punc_type: "?", before_expr: true };
static SEMI: PuncData = PuncData { punc_type: ";", before_expr: true };

// operators
static SLASH: OperatorData = OperatorData { binop: 10, before_expr: true, is_assign: false, is_update: false, postfix: false, prefix: false };
static EQ: OperatorData = OperatorData { binop: 255, before_expr: true, is_assign: true, is_update: false, postfix: false, prefix: false };
static ASSIGN: OperatorData = OperatorData { binop: 255, before_expr: true, is_assign: true, is_update: false, postfix: false, prefix: false };
static INC_DEC: OperatorData = OperatorData { binop: 255, before_expr: false, is_assign: false, is_update: true, postfix: true, prefix: true };
static PREFIX: OperatorData = OperatorData { binop: 255, before_expr: true, is_assign: false, is_update: false, postfix: false, prefix: true };
static LOGICAL_OR: OperatorData = OperatorData { binop: 1, before_expr: true, is_assign: false, is_update: false, postfix: false, prefix: false };
static LOGICIAL_AND: OperatorData = OperatorData { binop: 2, before_expr: true, is_assign: false, is_update: false, postfix: false, prefix: false };
static BITWISE_OR: OperatorData = OperatorData { binop: 3, before_expr: true, is_assign: false, is_update: false, postfix: false, prefix: false };
static BITWISE_XOR: OperatorData = OperatorData { binop: 4, before_expr: true, is_assign: false, is_update: false, postfix: false, prefix: false };
static BITWISE_AND: OperatorData = OperatorData { binop: 5, before_expr: true, is_assign: false, is_update: false, postfix: false, prefix: false };
static EQUALITY: OperatorData = OperatorData { binop: 6, before_expr: true, is_assign: false, is_update: false, postfix: false, prefix: false };
static RELATIONAL: OperatorData = OperatorData { binop: 7, before_expr: true, is_assign: false, is_update: false, postfix: false, prefix: false };
static BIT_SHIFT: OperatorData = OperatorData { binop: 8, before_expr: true, is_assign: false, is_update: false, postfix: false, prefix: false };
static PLUS_MIN: OperatorData = OperatorData { binop: 9, before_expr: true, is_assign: false, is_update: false, postfix: false, prefix: true };
static MODULO: OperatorData = OperatorData { binop: 10, before_expr: true, is_assign: false, is_update: false, postfix: false, prefix: false };
// '*' may be multiply or have special meaning in ES6
static STAR: OperatorData = OperatorData { binop: 10, before_expr: true, is_assign: false, is_update: false, postfix: false, prefix: false };


fn create_tokenizer(input: &str, options: Options) -> Tokenizer {
    Tokenizer::new(input, options)
}

fn main() {
    let options = Options{version: Ecma6};
    let mut tokenizer = create_tokenizer("(function($){'use strict';})(jQuery);", options);
    for token in tokenizer {
        println!("{}", token);
    }
}

struct Options {
    version: EcmaVersion
}

enum EcmaVersion {
    Ecma3,
    Ecma5,
    Ecma6
}

#[deriving(Show)]
struct Token {
    value: Option<String>,
    token_type: TokenType,
    start: uint,
    end: uint,
}

#[deriving(Show)]
enum TokenType {
    Name,
    Keyword(KeywordData),
    Punc(PuncData),
    Value(ValueData),
    Operator(OperatorData),
    Eof
}

#[deriving(Show)]
struct KeywordData {
    keyword: &'static str,
    is_loop: bool,
    before_expr: bool
}

#[deriving(Show)]
struct ValueData {
    keyword: &'static str,
    atom_value: Option<bool>
}

#[deriving(Show)]
struct PuncData {
    punc_type: &'static str,
    before_expr: bool
}

#[deriving(Show)]
struct OperatorData {
    binop: u8,
    before_expr: bool,
    is_assign: bool,
    postfix: bool,
    prefix: bool,
    is_update: bool
}

struct Tokenizer {
    options: Options,
    contains_esc: bool,
    input: String,
    input_len: uint,
    tok_pos: uint,
    // start and end of current token
    tok_start: uint,
    tok_end: uint
}

impl Tokenizer {
    fn new(input: &str, options: Options) -> Tokenizer {
        Tokenizer {
            options: options,
            contains_esc: false,
            input: input.to_string(),
            input_len: input.len(),
            tok_pos: 0,
            tok_start: 0,
            tok_end: 0
        }
    }

    fn init_token_state(&mut self) {
        self.skip_space();
    }

    fn curr_char(&self) -> char {
        self.char_at(self.tok_pos)
    }

    fn char_at(&self, pos: uint) -> char {
        self.input.as_slice().char_at(pos)
    }

    fn skip_space(&mut self) {
        while self.tok_pos < self.input_len {
            let original_ch = self.curr_char();
            let ch = original_ch as u32;
            if ch == 32 {
                self.tok_pos +=1;
            } else if ch == 13 {
                self.tok_pos +=1;
                let next = self.curr_char() as u32;
                if next == 10 {
                    self.tok_pos +=1;
                }
            } else if ch == 10 || ch == 8232 || ch == 8233 {
                self.tok_pos +=1;
            } else if ch > 8 && ch < 18 {
                self.tok_pos +=1;
            } else if ch == 47 { // '/'
                let next = self.char_at(self.tok_pos + 1) as u32;
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
        match index_of_with_offset(self.input.as_slice(), "*/", self.tok_pos) {
            Some(i) => self.tok_pos = i + 2,
            None => fail!("Unterminated comment at: {}", self.tok_pos - 2)
        };
    }

    fn skip_line_comment(&mut self, start_skip: uint){
        self.tok_pos += start_skip;
        let mut code = self.curr_char() as u32;
        while self.tok_pos < self.input_len && code != 10 && code != 13 && code != 8232 && code != 8233 {
            self.tok_pos += 1;
            code = self.curr_char() as u32;
        }
    }

    fn read_token(&mut self) -> Token {
        self.tok_start = self.tok_pos;
        if self.tok_pos >= self.input_len {
            return Token {value: None, token_type: Eof, start: self.tok_start, end: self.tok_start}
        }
        let code = self.curr_char() as u32;

        // Identifier or keyword. '\uXXXX' sequences are allowed in
        // identifiers, so '\' also goes to that.
        if Tokenizer::is_identifier_start(code) || code == 92 /* '/' */ {
            return self.read_word()
        }
        match self.read_token_from_code(code) {
            Some(token) => token,
            None => {
                  // If we are here, we either found a non-ASCII identifier
                  // character, or something that's entirely disallowed.
                  let ch = char::from_u32(code).unwrap();
                  if ch == '\\' || Tokenizer::is_non_ascii_identifier_start(code) {
                      self.read_word()
                  } else {
                      fail!("Unexpected character '{}'", ch);
                  }
            }
        }
    }

    fn read_token_from_code(&mut self, code: u32) -> Option<Token> {
        match code {
            // The interpretation of a dot depends on whether it is followed
            // by a digit or another two dots.
            //46 => self.read_token_dot(), // '.'
            40 => {
                self.tok_pos += 1;
                Some(self.finish_token(Punc(PAREN_L)))
            },
            _ => None
        }
    }

    //fn read_token_dot(&mut self) -> Token {
        //let next = self.char_at(self.tok_pos + 1) as u32;
        //if next >= 48  && next <= 57 {return self.read_number(true);}
        //else
    //}

    //fn read_number(&mut self, starts_with_dot: bool) -> Result<f64>  {
        //let start = self.tok_pos;
        //let is_float = false;
        //let octal = self.curr_char() == 48; // '0'
        //Ok(3.14f64)
    //}

    fn read_word(&mut self) -> Token {
        let word = self.read_word_in_loop();
        let token_type = if !self.contains_esc && self.is_keyword(word.as_slice()) {
            let keyword = self.get_keyword(word.as_slice());
            Keyword(keyword)
        } else { Name };
        self.finish_token_with_value(token_type, word.as_slice())
    }

    fn get_keyword(&self, keyword: &str) -> KeywordData {
        match keyword {
            "break" => BREAK,
            "case" => CASE,
            "catch" => CATCH,
            "continue" => CONTINUE,
            "debugger" => DEBUGGER,
            "default" => DEFAULT,
            "do" => DO,
            "else" => ELSE,
            "export" => EXPORT,
            "extends" => EXTENDS,
            "finally" => FINALLY,
            "import" => IMPORT,
            "for" => FOR,
            "function" => FUNCTION,
            "if" => IF,
            "let" => LET,
            "new" => NEW,
            "return" => RETURN,
            "switch" => SWITCH,
            "this" => THIS,
            "throw" => THROW,
            "try" => TRY,
            "var" => VAR,
            "while" => WHILE,
            "with" => WITH,
            _  => YIELD
        }
    }

    fn read_word_in_loop(&mut self) -> String {
        let start = self.tok_pos;
        let mut first = true;
        self.contains_esc = false;
        let mut word = "".to_string();
        loop {
            let code = self.curr_char() as u32;
            if Tokenizer::is_identifier_char(code) {
                if self.contains_esc {
                    word.push(self.curr_char());
                }
                self.tok_pos += 1;
            } else if code == 92 { /* '\'  */
                if !self.contains_esc {
                    word = self.input.as_slice().slice_chars(start, self.tok_pos).to_string();
                }
                self.contains_esc = true;
                self.tok_pos += 1;
                if self.curr_char() as u32 != 117 { // 'u'
                    fail!("{}: Expected Unicode escape sequence \\uXXXX", self.tok_pos);
                }
                self.tok_pos += 1;
                // TODO: better error handling
                let esc_code = self.read_hex_char(4).unwrap();
                match char::from_u32(esc_code) {
                    Some(ch) => {
                        let is_identifier_char = if first {
                                Tokenizer::is_identifier_start(esc_code)
                            } else {
                                Tokenizer::is_identifier_char(esc_code)
                            };
                        if is_identifier_char {
                            word.push(ch);
                        } else {
                            fail!("Invalid Unicode escape: {}", self.tok_pos - 4);
                        }

                    },
                    None => fail!("Invalid Unicode escape: {}", self.tok_pos - 1)
                };
            } else { break; }
            first = false;
        };

        if self.contains_esc {
            word
        } else {
            self.input.as_slice().slice_chars(start, self.tok_pos).to_string()
        }
    }

    fn read_hex_char(&mut self, len: uint) -> Option<u32> {
        match self.read_u32_of_len(16, len) {
            Some(n) => Some(n),
            None => fail!("Bad character escape sequence: {}", self.tok_pos)
        }
    }

    // Reads an unsigned integer in given radix of `len` length
    // if zero digits were read, returns None.
    // If integer is not of length `len`, None is returned
    fn read_u32_of_len(&mut self, radix: u32, len: uint) -> Option<u32> {
        let start = self.tok_pos;
        let mut total = 0;
        for _ in range(0, len) {
            let  code = self.curr_char() as u32;
            let val = if code >= 97 {
                code - 97 + 10 // a
            } else if code >= 65 {
                code - 65 + 10 // A
            } else if code >= 48 && code <= 57 { //0-9
                code - 48
            } else {
                fail!("Invalid value at: {}", self.tok_pos);
            };
            if val >= radix { break; }
            self.tok_pos += 1;
            total = total * radix + val;
        }
        if self.tok_pos - start != len {
            None
        } else {
            Some(total)
        }
    }

    fn finish_token_with_value(&mut self, token_type: TokenType, value: &str) -> Token {
        self.tok_end = self.tok_pos;
        Token { value: Some(value.to_string()), token_type: token_type, start: self.tok_start, end: self.tok_end }
    }

    fn finish_token(&mut self, token_type: TokenType) -> Token {
        self.tok_end = self.tok_pos;
        Token {value: None, token_type: token_type, start: self.tok_start, end: self.tok_end }
    }

    /// test if char code can start an identifier
    fn is_identifier_start(code: u32) -> bool {
        if code < 65 {return code == 36;}
        if code < 91 {return true;}
        if code < 97 {return code == 95;}
        if code < 123 {return true;}
        code >= 0xAA && Tokenizer::is_non_ascii_identifier_start(code)
    }

    /// test if char code can be in identifier
    fn is_identifier_char(code: u32) -> bool {
        if code < 48 { return code == 36; }
        if code < 58 { return true; }
        if code < 65 { return false; }
        if code < 91 { return true; }
        if code < 97 { return code == 95; }
        if code < 123 { return true; }
        code >= 0xAA && Tokenizer::is_non_ascii_identifier_char(code)
    }

    fn is_non_ascii_identifier_start(code: u32) -> bool {
        let ch = char::from_u32(code).unwrap();//it's safe here
        // regex taken from esprima.js
        // TODO: move to static?
        let re = regex!("[\xAA\xB5\xBA\xC0-\xD6\xD8-\xF6\xF8-\u02C1\u02C6-\u02D1\u02E0-\u02E4\u02EC\u02EE\u0370-\u0374\u0376\u0377\u037A-\u037D\u037F\u0386\u0388-\u038A\u038C\u038E-\u03A1\u03A3-\u03F5\u03F7-\u0481\u048A-\u052F\u0531-\u0556\u0559\u0561-\u0587\u05D0-\u05EA\u05F0-\u05F2\u0620-\u064A\u066E\u066F\u0671-\u06D3\u06D5\u06E5\u06E6\u06EE\u06EF\u06FA-\u06FC\u06FF\u0710\u0712-\u072F\u074D-\u07A5\u07B1\u07CA-\u07EA\u07F4\u07F5\u07FA\u0800-\u0815\u081A\u0824\u0828\u0840-\u0858\u08A0-\u08B2\u0904-\u0939\u093D\u0950\u0958-\u0961\u0971-\u0980\u0985-\u098C\u098F\u0990\u0993-\u09A8\u09AA-\u09B0\u09B2\u09B6-\u09B9\u09BD\u09CE\u09DC\u09DD\u09DF-\u09E1\u09F0\u09F1\u0A05-\u0A0A\u0A0F\u0A10\u0A13-\u0A28\u0A2A-\u0A30\u0A32\u0A33\u0A35\u0A36\u0A38\u0A39\u0A59-\u0A5C\u0A5E\u0A72-\u0A74\u0A85-\u0A8D\u0A8F-\u0A91\u0A93-\u0AA8\u0AAA-\u0AB0\u0AB2\u0AB3\u0AB5-\u0AB9\u0ABD\u0AD0\u0AE0\u0AE1\u0B05-\u0B0C\u0B0F\u0B10\u0B13-\u0B28\u0B2A-\u0B30\u0B32\u0B33\u0B35-\u0B39\u0B3D\u0B5C\u0B5D\u0B5F-\u0B61\u0B71\u0B83\u0B85-\u0B8A\u0B8E-\u0B90\u0B92-\u0B95\u0B99\u0B9A\u0B9C\u0B9E\u0B9F\u0BA3\u0BA4\u0BA8-\u0BAA\u0BAE-\u0BB9\u0BD0\u0C05-\u0C0C\u0C0E-\u0C10\u0C12-\u0C28\u0C2A-\u0C39\u0C3D\u0C58\u0C59\u0C60\u0C61\u0C85-\u0C8C\u0C8E-\u0C90\u0C92-\u0CA8\u0CAA-\u0CB3\u0CB5-\u0CB9\u0CBD\u0CDE\u0CE0\u0CE1\u0CF1\u0CF2\u0D05-\u0D0C\u0D0E-\u0D10\u0D12-\u0D3A\u0D3D\u0D4E\u0D60\u0D61\u0D7A-\u0D7F\u0D85-\u0D96\u0D9A-\u0DB1\u0DB3-\u0DBB\u0DBD\u0DC0-\u0DC6\u0E01-\u0E30\u0E32\u0E33\u0E40-\u0E46\u0E81\u0E82\u0E84\u0E87\u0E88\u0E8A\u0E8D\u0E94-\u0E97\u0E99-\u0E9F\u0EA1-\u0EA3\u0EA5\u0EA7\u0EAA\u0EAB\u0EAD-\u0EB0\u0EB2\u0EB3\u0EBD\u0EC0-\u0EC4\u0EC6\u0EDC-\u0EDF\u0F00\u0F40-\u0F47\u0F49-\u0F6C\u0F88-\u0F8C\u1000-\u102A\u103F\u1050-\u1055\u105A-\u105D\u1061\u1065\u1066\u106E-\u1070\u1075-\u1081\u108E\u10A0-\u10C5\u10C7\u10CD\u10D0-\u10FA\u10FC-\u1248\u124A-\u124D\u1250-\u1256\u1258\u125A-\u125D\u1260-\u1288\u128A-\u128D\u1290-\u12B0\u12B2-\u12B5\u12B8-\u12BE\u12C0\u12C2-\u12C5\u12C8-\u12D6\u12D8-\u1310\u1312-\u1315\u1318-\u135A\u1380-\u138F\u13A0-\u13F4\u1401-\u166C\u166F-\u167F\u1681-\u169A\u16A0-\u16EA\u16EE-\u16F8\u1700-\u170C\u170E-\u1711\u1720-\u1731\u1740-\u1751\u1760-\u176C\u176E-\u1770\u1780-\u17B3\u17D7\u17DC\u1820-\u1877\u1880-\u18A8\u18AA\u18B0-\u18F5\u1900-\u191E\u1950-\u196D\u1970-\u1974\u1980-\u19AB\u19C1-\u19C7\u1A00-\u1A16\u1A20-\u1A54\u1AA7\u1B05-\u1B33\u1B45-\u1B4B\u1B83-\u1BA0\u1BAE\u1BAF\u1BBA-\u1BE5\u1C00-\u1C23\u1C4D-\u1C4F\u1C5A-\u1C7D\u1CE9-\u1CEC\u1CEE-\u1CF1\u1CF5\u1CF6\u1D00-\u1DBF\u1E00-\u1F15\u1F18-\u1F1D\u1F20-\u1F45\u1F48-\u1F4D\u1F50-\u1F57\u1F59\u1F5B\u1F5D\u1F5F-\u1F7D\u1F80-\u1FB4\u1FB6-\u1FBC\u1FBE\u1FC2-\u1FC4\u1FC6-\u1FCC\u1FD0-\u1FD3\u1FD6-\u1FDB\u1FE0-\u1FEC\u1FF2-\u1FF4\u1FF6-\u1FFC\u2071\u207F\u2090-\u209C\u2102\u2107\u210A-\u2113\u2115\u2119-\u211D\u2124\u2126\u2128\u212A-\u212D\u212F-\u2139\u213C-\u213F\u2145-\u2149\u214E\u2160-\u2188\u2C00-\u2C2E\u2C30-\u2C5E\u2C60-\u2CE4\u2CEB-\u2CEE\u2CF2\u2CF3\u2D00-\u2D25\u2D27\u2D2D\u2D30-\u2D67\u2D6F\u2D80-\u2D96\u2DA0-\u2DA6\u2DA8-\u2DAE\u2DB0-\u2DB6\u2DB8-\u2DBE\u2DC0-\u2DC6\u2DC8-\u2DCE\u2DD0-\u2DD6\u2DD8-\u2DDE\u2E2F\u3005-\u3007\u3021-\u3029\u3031-\u3035\u3038-\u303C\u3041-\u3096\u309D-\u309F\u30A1-\u30FA\u30FC-\u30FF\u3105-\u312D\u3131-\u318E\u31A0-\u31BA\u31F0-\u31FF\u3400-\u4DB5\u4E00-\u9FCC\uA000-\uA48C\uA4D0-\uA4FD\uA500-\uA60C\uA610-\uA61F\uA62A\uA62B\uA640-\uA66E\uA67F-\uA69D\uA6A0-\uA6EF\uA717-\uA71F\uA722-\uA788\uA78B-\uA78E\uA790-\uA7AD\uA7B0\uA7B1\uA7F7-\uA801\uA803-\uA805\uA807-\uA80A\uA80C-\uA822\uA840-\uA873\uA882-\uA8B3\uA8F2-\uA8F7\uA8FB\uA90A-\uA925\uA930-\uA946\uA960-\uA97C\uA984-\uA9B2\uA9CF\uA9E0-\uA9E4\uA9E6-\uA9EF\uA9FA-\uA9FE\uAA00-\uAA28\uAA40-\uAA42\uAA44-\uAA4B\uAA60-\uAA76\uAA7A\uAA7E-\uAAAF\uAAB1\uAAB5\uAAB6\uAAB9-\uAABD\uAAC0\uAAC2\uAADB-\uAADD\uAAE0-\uAAEA\uAAF2-\uAAF4\uAB01-\uAB06\uAB09-\uAB0E\uAB11-\uAB16\uAB20-\uAB26\uAB28-\uAB2E\uAB30-\uAB5A\uAB5C-\uAB5F\uAB64\uAB65\uABC0-\uABE2\uAC00-\uD7A3\uD7B0-\uD7C6\uD7CB-\uD7FB\uF900-\uFA6D\uFA70-\uFAD9\uFB00-\uFB06\uFB13-\uFB17\uFB1D\uFB1F-\uFB28\uFB2A-\uFB36\uFB38-\uFB3C\uFB3E\uFB40\uFB41\uFB43\uFB44\uFB46-\uFBB1\uFBD3-\uFD3D\uFD50-\uFD8F\uFD92-\uFDC7\uFDF0-\uFDFB\uFE70-\uFE74\uFE76-\uFEFC\uFF21-\uFF3A\uFF41-\uFF5A\uFF66-\uFFBE\uFFC2-\uFFC7\uFFCA-\uFFCF\uFFD2-\uFFD7\uFFDA-\uFFDC]");
        re.is_match(ch.to_string().as_slice())
    }

    fn is_non_ascii_identifier_char(code: u32) -> bool {
        let ch = char::from_u32(code).unwrap();//it's safe here
        // regex taken from esprima.js
        // regex is a combination of non-ascii identifier start + non-ascii identifier
        // first unicode char from 2nd group is \u0300
        let re = regex!("[\xAA\xB5\xBA\xC0-\xD6\xD8-\xF6\xF8-\u02C1\u02C6-\u02D1\u02E0-\u02E4\u02EC\u02EE\u0370-\u0374\u0376\u0377\u037A-\u037D\u037F\u0386\u0388-\u038A\u038C\u038E-\u03A1\u03A3-\u03F5\u03F7-\u0481\u048A-\u052F\u0531-\u0556\u0559\u0561-\u0587\u05D0-\u05EA\u05F0-\u05F2\u0620-\u064A\u066E\u066F\u0671-\u06D3\u06D5\u06E5\u06E6\u06EE\u06EF\u06FA-\u06FC\u06FF\u0710\u0712-\u072F\u074D-\u07A5\u07B1\u07CA-\u07EA\u07F4\u07F5\u07FA\u0800-\u0815\u081A\u0824\u0828\u0840-\u0858\u08A0-\u08B2\u0904-\u0939\u093D\u0950\u0958-\u0961\u0971-\u0980\u0985-\u098C\u098F\u0990\u0993-\u09A8\u09AA-\u09B0\u09B2\u09B6-\u09B9\u09BD\u09CE\u09DC\u09DD\u09DF-\u09E1\u09F0\u09F1\u0A05-\u0A0A\u0A0F\u0A10\u0A13-\u0A28\u0A2A-\u0A30\u0A32\u0A33\u0A35\u0A36\u0A38\u0A39\u0A59-\u0A5C\u0A5E\u0A72-\u0A74\u0A85-\u0A8D\u0A8F-\u0A91\u0A93-\u0AA8\u0AAA-\u0AB0\u0AB2\u0AB3\u0AB5-\u0AB9\u0ABD\u0AD0\u0AE0\u0AE1\u0B05-\u0B0C\u0B0F\u0B10\u0B13-\u0B28\u0B2A-\u0B30\u0B32\u0B33\u0B35-\u0B39\u0B3D\u0B5C\u0B5D\u0B5F-\u0B61\u0B71\u0B83\u0B85-\u0B8A\u0B8E-\u0B90\u0B92-\u0B95\u0B99\u0B9A\u0B9C\u0B9E\u0B9F\u0BA3\u0BA4\u0BA8-\u0BAA\u0BAE-\u0BB9\u0BD0\u0C05-\u0C0C\u0C0E-\u0C10\u0C12-\u0C28\u0C2A-\u0C39\u0C3D\u0C58\u0C59\u0C60\u0C61\u0C85-\u0C8C\u0C8E-\u0C90\u0C92-\u0CA8\u0CAA-\u0CB3\u0CB5-\u0CB9\u0CBD\u0CDE\u0CE0\u0CE1\u0CF1\u0CF2\u0D05-\u0D0C\u0D0E-\u0D10\u0D12-\u0D3A\u0D3D\u0D4E\u0D60\u0D61\u0D7A-\u0D7F\u0D85-\u0D96\u0D9A-\u0DB1\u0DB3-\u0DBB\u0DBD\u0DC0-\u0DC6\u0E01-\u0E30\u0E32\u0E33\u0E40-\u0E46\u0E81\u0E82\u0E84\u0E87\u0E88\u0E8A\u0E8D\u0E94-\u0E97\u0E99-\u0E9F\u0EA1-\u0EA3\u0EA5\u0EA7\u0EAA\u0EAB\u0EAD-\u0EB0\u0EB2\u0EB3\u0EBD\u0EC0-\u0EC4\u0EC6\u0EDC-\u0EDF\u0F00\u0F40-\u0F47\u0F49-\u0F6C\u0F88-\u0F8C\u1000-\u102A\u103F\u1050-\u1055\u105A-\u105D\u1061\u1065\u1066\u106E-\u1070\u1075-\u1081\u108E\u10A0-\u10C5\u10C7\u10CD\u10D0-\u10FA\u10FC-\u1248\u124A-\u124D\u1250-\u1256\u1258\u125A-\u125D\u1260-\u1288\u128A-\u128D\u1290-\u12B0\u12B2-\u12B5\u12B8-\u12BE\u12C0\u12C2-\u12C5\u12C8-\u12D6\u12D8-\u1310\u1312-\u1315\u1318-\u135A\u1380-\u138F\u13A0-\u13F4\u1401-\u166C\u166F-\u167F\u1681-\u169A\u16A0-\u16EA\u16EE-\u16F8\u1700-\u170C\u170E-\u1711\u1720-\u1731\u1740-\u1751\u1760-\u176C\u176E-\u1770\u1780-\u17B3\u17D7\u17DC\u1820-\u1877\u1880-\u18A8\u18AA\u18B0-\u18F5\u1900-\u191E\u1950-\u196D\u1970-\u1974\u1980-\u19AB\u19C1-\u19C7\u1A00-\u1A16\u1A20-\u1A54\u1AA7\u1B05-\u1B33\u1B45-\u1B4B\u1B83-\u1BA0\u1BAE\u1BAF\u1BBA-\u1BE5\u1C00-\u1C23\u1C4D-\u1C4F\u1C5A-\u1C7D\u1CE9-\u1CEC\u1CEE-\u1CF1\u1CF5\u1CF6\u1D00-\u1DBF\u1E00-\u1F15\u1F18-\u1F1D\u1F20-\u1F45\u1F48-\u1F4D\u1F50-\u1F57\u1F59\u1F5B\u1F5D\u1F5F-\u1F7D\u1F80-\u1FB4\u1FB6-\u1FBC\u1FBE\u1FC2-\u1FC4\u1FC6-\u1FCC\u1FD0-\u1FD3\u1FD6-\u1FDB\u1FE0-\u1FEC\u1FF2-\u1FF4\u1FF6-\u1FFC\u2071\u207F\u2090-\u209C\u2102\u2107\u210A-\u2113\u2115\u2119-\u211D\u2124\u2126\u2128\u212A-\u212D\u212F-\u2139\u213C-\u213F\u2145-\u2149\u214E\u2160-\u2188\u2C00-\u2C2E\u2C30-\u2C5E\u2C60-\u2CE4\u2CEB-\u2CEE\u2CF2\u2CF3\u2D00-\u2D25\u2D27\u2D2D\u2D30-\u2D67\u2D6F\u2D80-\u2D96\u2DA0-\u2DA6\u2DA8-\u2DAE\u2DB0-\u2DB6\u2DB8-\u2DBE\u2DC0-\u2DC6\u2DC8-\u2DCE\u2DD0-\u2DD6\u2DD8-\u2DDE\u2E2F\u3005-\u3007\u3021-\u3029\u3031-\u3035\u3038-\u303C\u3041-\u3096\u309D-\u309F\u30A1-\u30FA\u30FC-\u30FF\u3105-\u312D\u3131-\u318E\u31A0-\u31BA\u31F0-\u31FF\u3400-\u4DB5\u4E00-\u9FCC\uA000-\uA48C\uA4D0-\uA4FD\uA500-\uA60C\uA610-\uA61F\uA62A\uA62B\uA640-\uA66E\uA67F-\uA69D\uA6A0-\uA6EF\uA717-\uA71F\uA722-\uA788\uA78B-\uA78E\uA790-\uA7AD\uA7B0\uA7B1\uA7F7-\uA801\uA803-\uA805\uA807-\uA80A\uA80C-\uA822\uA840-\uA873\uA882-\uA8B3\uA8F2-\uA8F7\uA8FB\uA90A-\uA925\uA930-\uA946\uA960-\uA97C\uA984-\uA9B2\uA9CF\uA9E0-\uA9E4\uA9E6-\uA9EF\uA9FA-\uA9FE\uAA00-\uAA28\uAA40-\uAA42\uAA44-\uAA4B\uAA60-\uAA76\uAA7A\uAA7E-\uAAAF\uAAB1\uAAB5\uAAB6\uAAB9-\uAABD\uAAC0\uAAC2\uAADB-\uAADD\uAAE0-\uAAEA\uAAF2-\uAAF4\uAB01-\uAB06\uAB09-\uAB0E\uAB11-\uAB16\uAB20-\uAB26\uAB28-\uAB2E\uAB30-\uAB5A\uAB5C-\uAB5F\uAB64\uAB65\uABC0-\uABE2\uAC00-\uD7A3\uD7B0-\uD7C6\uD7CB-\uD7FB\uF900-\uFA6D\uFA70-\uFAD9\uFB00-\uFB06\uFB13-\uFB17\uFB1D\uFB1F-\uFB28\uFB2A-\uFB36\uFB38-\uFB3C\uFB3E\uFB40\uFB41\uFB43\uFB44\uFB46-\uFBB1\uFBD3-\uFD3D\uFD50-\uFD8F\uFD92-\uFDC7\uFDF0-\uFDFB\uFE70-\uFE74\uFE76-\uFEFC\uFF21-\uFF3A\uFF41-\uFF5A\uFF66-\uFFBE\uFFC2-\uFFC7\uFFCA-\uFFCF\uFFD2-\uFFD7\uFFDA-\uFFDC\u0300-\u036F\u0483-\u0487\u0591-\u05BD\u05BF\u05C1\u05C2\u05C4\u05C5\u05C7\u0610-\u061A\u064B-\u0669\u0670\u06D6-\u06DC\u06DF-\u06E4\u06E7\u06E8\u06EA-\u06ED\u06F0-\u06F9\u0711\u0730-\u074A\u07A6-\u07B0\u07C0-\u07C9\u07EB-\u07F3\u0816-\u0819\u081B-\u0823\u0825-\u0827\u0829-\u082D\u0859-\u085B\u08E4-\u0903\u093A-\u093C\u093E-\u094F\u0951-\u0957\u0962\u0963\u0966-\u096F\u0981-\u0983\u09BC\u09BE-\u09C4\u09C7\u09C8\u09CB-\u09CD\u09D7\u09E2\u09E3\u09E6-\u09EF\u0A01-\u0A03\u0A3C\u0A3E-\u0A42\u0A47\u0A48\u0A4B-\u0A4D\u0A51\u0A66-\u0A71\u0A75\u0A81-\u0A83\u0ABC\u0ABE-\u0AC5\u0AC7-\u0AC9\u0ACB-\u0ACD\u0AE2\u0AE3\u0AE6-\u0AEF\u0B01-\u0B03\u0B3C\u0B3E-\u0B44\u0B47\u0B48\u0B4B-\u0B4D\u0B56\u0B57\u0B62\u0B63\u0B66-\u0B6F\u0B82\u0BBE-\u0BC2\u0BC6-\u0BC8\u0BCA-\u0BCD\u0BD7\u0BE6-\u0BEF\u0C00-\u0C03\u0C3E-\u0C44\u0C46-\u0C48\u0C4A-\u0C4D\u0C55\u0C56\u0C62\u0C63\u0C66-\u0C6F\u0C81-\u0C83\u0CBC\u0CBE-\u0CC4\u0CC6-\u0CC8\u0CCA-\u0CCD\u0CD5\u0CD6\u0CE2\u0CE3\u0CE6-\u0CEF\u0D01-\u0D03\u0D3E-\u0D44\u0D46-\u0D48\u0D4A-\u0D4D\u0D57\u0D62\u0D63\u0D66-\u0D6F\u0D82\u0D83\u0DCA\u0DCF-\u0DD4\u0DD6\u0DD8-\u0DDF\u0DE6-\u0DEF\u0DF2\u0DF3\u0E31\u0E34-\u0E3A\u0E47-\u0E4E\u0E50-\u0E59\u0EB1\u0EB4-\u0EB9\u0EBB\u0EBC\u0EC8-\u0ECD\u0ED0-\u0ED9\u0F18\u0F19\u0F20-\u0F29\u0F35\u0F37\u0F39\u0F3E\u0F3F\u0F71-\u0F84\u0F86\u0F87\u0F8D-\u0F97\u0F99-\u0FBC\u0FC6\u102B-\u103E\u1040-\u1049\u1056-\u1059\u105E-\u1060\u1062-\u1064\u1067-\u106D\u1071-\u1074\u1082-\u108D\u108F-\u109D\u135D-\u135F\u1712-\u1714\u1732-\u1734\u1752\u1753\u1772\u1773\u17B4-\u17D3\u17DD\u17E0-\u17E9\u180B-\u180D\u1810-\u1819\u18A9\u1920-\u192B\u1930-\u193B\u1946-\u194F\u19B0-\u19C0\u19C8\u19C9\u19D0-\u19D9\u1A17-\u1A1B\u1A55-\u1A5E\u1A60-\u1A7C\u1A7F-\u1A89\u1A90-\u1A99\u1AB0-\u1ABD\u1B00-\u1B04\u1B34-\u1B44\u1B50-\u1B59\u1B6B-\u1B73\u1B80-\u1B82\u1BA1-\u1BAD\u1BB0-\u1BB9\u1BE6-\u1BF3\u1C24-\u1C37\u1C40-\u1C49\u1C50-\u1C59\u1CD0-\u1CD2\u1CD4-\u1CE8\u1CED\u1CF2-\u1CF4\u1CF8\u1CF9\u1DC0-\u1DF5\u1DFC-\u1DFF\u200C\u200D\u203F\u2040\u2054\u20D0-\u20DC\u20E1\u20E5-\u20F0\u2CEF-\u2CF1\u2D7F\u2DE0-\u2DFF\u302A-\u302F\u3099\u309A\uA620-\uA629\uA66F\uA674-\uA67D\uA69F\uA6F0\uA6F1\uA802\uA806\uA80B\uA823-\uA827\uA880\uA881\uA8B4-\uA8C4\uA8D0-\uA8D9\uA8E0-\uA8F1\uA900-\uA909\uA926-\uA92D\uA947-\uA953\uA980-\uA983\uA9B3-\uA9C0\uA9D0-\uA9D9\uA9E5\uA9F0-\uA9F9\uAA29-\uAA36\uAA43\uAA4C\uAA4D\uAA50-\uAA59\uAA7B-\uAA7D\uAAB0\uAAB2-\uAAB4\uAAB7\uAAB8\uAABE\uAABF\uAAC1\uAAEB-\uAAEF\uAAF5\uAAF6\uABE3-\uABEA\uABEC\uABED\uABF0-\uABF9\uFB1E\uFE00-\uFE0F\uFE20-\uFE2D\uFE33\uFE34\uFE4D-\uFE4F\uFF10-\uFF19\uFF3F]");
        re.is_match(ch.to_string().as_slice())
    }

    fn is_keyword(&self, word: &str) -> bool {
        match self.options.version {
            Ecma6 => ECMA6_KEYWORDS.contains(&word),
            _ => ECMA5_KEYWORDS.contains(&word)
        }
    }
}

impl Iterator<Token> for Tokenizer {
    fn next(&mut self) -> Option<Token> {
        let token = self.read_token();
        match token.token_type {
            Eof => None,
            _ => Some(token)
        }
    }
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

