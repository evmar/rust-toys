fn main() {
    let stdin = std::io::stdin();
    let mut line_count = 0;
    let mut word_count = 0;
    let mut byte_count = 0;
    let mut in_whitespace = true;

    for stdin.each_byte() |b| {
        byte_count += 1;
        match b as char {
            '\n' => { line_count += 1; in_whitespace = true; }
            ' ' | '\t' => { in_whitespace = true; }
            _ => {
                if in_whitespace {
                    word_count += 1;
                    in_whitespace = false;
                }
            }
        }
    }
    println(fmt!("%d %d %d", line_count, word_count, byte_count));
}
