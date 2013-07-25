extern mod extra;

use std::int;
use std::io;
use std::os;
use std::path;
use std::str;

use extra::getopts::*;

/// BufReader wraps a Reader with a buffer, letting you read a char at a time.
/// (The Reader returned by io::file_reader uses calls libc's getc() for each
/// call to read_byte(), which is very slow.)
struct BufReader {
    r: @Reader,
    buf: [u8, ..4096],
    ofs: uint,
    len: uint,
}

impl BufReader {
    fn new(r: @Reader) -> BufReader {
        BufReader {r: r, buf: [0, ..4096], ofs: 0, len: 0}
    }

    fn fill(&mut self) {
        if self.ofs < self.len {
            fail!("must read_char() to the end before filling");
        }
        self.len = self.r.read(self.buf, 4096);
        self.ofs = 0;
    }

    fn unread_char(&mut self, char: char) {
        if self.ofs == 0 {
            fail!("cannot unread further")
        }
        self.ofs -= 1;
        self.buf[self.ofs] = char as u8;
        if self.len == 0 {
            self.len = 1;
        }
    }

    fn read_char(&mut self) -> Option<char> {
        if self.ofs == self.len {
            self.fill();
        }

        if self.ofs == self.len {
            return None;
        }

        let c = self.buf[self.ofs] as char;
        self.ofs += 1;
        return Some(c);
    }

    fn must_read_char(&mut self) -> char {
        // XXX 10% slower: self.read_char().expect("eof")
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

/// The common log format used by Apache logs.
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
    referer: Option<~str>,
    user_agent: Option<~str>,
}

fn read_quoted(r: &mut BufReader) -> ~str {
    let mut str = str::with_capacity(64);
    loop {
        match r.must_read_char() {
            '"' => return str,
            '\\' => str.push_char(r.must_read_char()),
            c => str.push_char(c),

        }
    }
}

fn read_braced(r: &mut BufReader) -> ~str {
    let mut str = str::with_capacity(64);
    loop {
        match r.must_read_char() {
            ']' => return str,
            c => str.push_char(c),
        }
    }
}

fn read_plain(r: &mut BufReader) -> ~str {
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

fn read_tok(r: &mut BufReader) -> ~str {
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

    // Swallow the following space, if any.
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

fn parse(r: @Reader) -> int {
    let fmt = &combined_log_format;
    let mut ur = ~BufReader::new(r);
    let mut count = 0;
    while !ur.r.eof() {
        let mut e = LogEntry {
            source: ~"", date: ~"", request: ~"",
            status: -1, size: None,
            referer: None, user_agent: None
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
                            None => fail!("bad size: " + s),
                            s => s,
                        }
                    }
                }
                Referer => { e.referer = tok_to_option(read_tok(ur)); }
                UserAgent => { e.user_agent = tok_to_option(read_tok(ur)); }
            }
        };
        assert!(ur.must_read_char() == '\n');
        //println(fmt!("%?", e));
        count += 1;
        ur.prod();
    }
    return count;
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
        Ok(r) => {
            println(fmt!("read %d entries", parse(r)));
        }
        Err(msg) => println(msg)
    }
}
