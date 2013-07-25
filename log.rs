extern mod extra;

use std::int;
use std::io;
use std::os;
use std::path;
use std::str;

use extra::getopts::*;

struct UnGetReader {
    r:    @Reader,
    char: Option<char>,
    ofs:  int,
}

impl UnGetReader {
    fn new(r: @Reader) -> UnGetReader {
        UnGetReader {
            r: r,
            char: None,
            ofs: 0,
        }
    }

    fn unread_char(&mut self, char: char) {
        match self.char {
            Some(_) => fail!("multiple unreads"),
            None => self.char = Some(char)
        }
    }

    fn read_char(&mut self) -> Option<char> {
        match self.char {
            Some(_) => self.char.take(),
            None => {
                match self.r.read_byte() {
                    b if b < 0 => None,
                    b => {
                        self.ofs += 1;
                        Some(b as char)
                    }
                }
            }
        }
    }

    fn must_read_char(&mut self) -> char {
        match self.read_char() {
            None => fail!("eof"),
            Some(c) => c
        }
    }

    fn prod(&mut self) {
        match self.read_char() {
            Some(c) => self.unread_char(c),
            None => {}
        }
    }
}

enum LogField {
    Unused,
    Source,
    Date,
    Request,
    Status,
    Size,
    Referer,
    UserAgent
}

static combined_log_format: [LogField, ..9] = [
    Source, Unused, Unused,
    Date, Request, Status, Size,
    Referer, UserAgent
];

struct LogEntry {
    source: ~str,
    date: ~str,
    request: ~str,
    status: int,
    size: Option<int>,
    referer: ~str,
    user_agent: ~str,
}

fn read_quoted(r: &mut UnGetReader) -> ~str {
    let mut str = str::with_capacity(64);
    loop {
        match r.must_read_char() {
            '"' => return str,
            '\\' => str.push_char(r.must_read_char()),
            c => str.push_char(c),

        }
    }
}

fn read_braced(r: &mut UnGetReader) -> ~str {
    let mut str = str::with_capacity(64);
    loop {
        match r.must_read_char() {
            ']' => return str,
            c => str.push_char(c),
        }
    }
}

fn read_plain(r: &mut UnGetReader) -> ~str {
    let mut str = str::with_capacity(64);
    loop {
        let c = r.must_read_char();
        match c {
            ' ' | '\n' => {
                r.unread_char(c);
                return str
            }
            c => str.push_char(c)
        }
    }
}

fn read_tok(r: &mut UnGetReader) -> ~str {
    let tok = match r.read_char() {
        None => fail!("eof at offset " + r.ofs.to_str()),
        Some(c) => {
            match c {
                '"' => read_quoted(r),
                '[' => read_braced(r),
                ' ' | '\n' => fail!("bad syntax"),
                c => {
                    r.unread_char(c);
                    read_plain(r)
                }
            }
        }
    };
    match r.must_read_char() {
        ' ' => {},
        c => r.unread_char(c)
    }
    return tok
}

fn tok_to_option(tok: ~str) -> Option<~str> {
    match tok {
        ~"-" => None,
        _ => Some(tok)
    }
}

fn parse(r: @Reader) {
    let fmt = &combined_log_format;
    let mut ur = ~UnGetReader::new(r);
    while !ur.r.eof() {
        let mut e = LogEntry {
            source: ~"", date: ~"", request: ~"",
            status: -1, size: None,
            referer: ~"", user_agent: ~""
        };
        for fmt.iter().advance |field| {
            match *field {
                Unused => { read_tok(ur); }
                Source => { e.source = read_tok(ur); }
                Date => { e.date = read_tok(ur); }
                Request => { e.request = read_tok(ur); }
                Status => {
                    let tok = read_tok(ur);
                    e.status = match int::from_str(tok) {
                        Some(s) => s,
                        None => fail!("bad status: " + tok),
                    };
                }
                Size => {
                    e.size = match read_tok(ur) {
                        ~"-" => None,
                        s => match int::from_str(s) {
                            None => fail!("bad size"),
                            s => s,
                        }
                    }
                }
                Referer => { e.referer = read_tok(ur); }
                UserAgent => { e.user_agent = read_tok(ur); }
            }
        };
        assert!(ur.must_read_char() == '\n');
        //println(fmt!("%?", e));
        ur.prod();
    }
}

fn main() {
    let opts = ~[];
    let args = os::args();
    let m = getopts(args.tail(), opts).get();
    let path = match m.free {
        [path] => path,
        _ => fail!("path missing")
    };
        
    match io::file_reader(~path::Path(path)) {
        Ok(r) => parse(r),
        Err(msg) => println(msg)
    }
}
