//! CSS utility classes need to be lexed into segments, which represent the variants and the utility,
//! and whether they are arbitrary or not. Some examples:
//! - `px-2`: utility `px-2`.
//! - `hover:px-2`: variant `hover`, utility `px-2`.
//! - `sm:hover:px-2`: variant `sm`, variant `hover`, utility `px-2`.
//! - `hover:[mask:circle]`: variant `hover`, utility `[mask:circle]` (arbitrary).
//! - `[&:nth-child(3)]:px-2`: variant `[&:nth-child(3)]` (arbitrary), utility `px-2`.
//! The results of the lexer are then used to process classes into `ClassInfo` structs, which are, in
//! turn, used to sort the classes.

/// Splits a string into segments based on a list of indexes. The characters at the indexes are not
/// included in the segments, as they are considered delimiters.
fn split_at_indexes<'a>(s: &'a str, indexes: &[usize]) -> Vec<&'a str> {
    let mut segments = Vec::new();
    let mut start_offset = 0;
    let mut start = 0;

    for &index in indexes {
        if index > s.len() {
            break; // Avoid panicking on out-of-bounds indexes
        }
        if index > start {
            segments.push(&s[start + start_offset..index]);
        }
        start_offset = 1;
        start = index;
    }

    if start + start_offset < s.len() {
        segments.push(&s[start + start_offset..]);
    }

    segments
}

// TODO: unit tests.

#[derive(Debug, Clone, PartialEq)]
enum Quote {
    Single,
    Double,
    Backtick,
}

impl Quote {
    fn from_char(c: char) -> Option<Quote> {
        match c {
            '\'' => Some(Quote::Single),
            '"' => Some(Quote::Double),
            '`' => Some(Quote::Backtick),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum CharKind {
    Other,
    Quote(Quote),
    Backslash,
}

/// Information about the structure of a segment of a CSS class (variant or utility).
#[derive(Debug)]
pub struct ClassSegmentStructure {
    pub arbitrary: bool,
    pub text: String,
}

/// Information about the structure of a CSS class.
#[derive(Debug)]
pub struct ClassStructure {
    pub variants: Vec<ClassSegmentStructure>,
    pub utility: ClassSegmentStructure,
}

/// Processes a CSS class into a class structure, containing a list of variants and the
/// utility itself.
pub fn tokenize_class(class_name: &str) -> Option<ClassStructure> {
    // TODO: add custom separator argument (currently hardcoded to `:`).
    let mut arbitrary_block_depth = 0;
    let mut at_arbitrary_block_start = false;
    let mut quoted_arbitrary_block_type: Option<Quote> = None;
    let mut last_char = CharKind::Other;
    let mut delimiter_indexes: Vec<usize> = Vec::new();

    for (index, c) in class_name.char_indices() {
        let mut next_last_char = CharKind::Other;
        let mut is_start_of_arbitrary_block = false;
        match c {
            '[' => {
                if arbitrary_block_depth == 0 {
                    arbitrary_block_depth = 1;
                    at_arbitrary_block_start = true;
                    is_start_of_arbitrary_block = true;
                } else if quoted_arbitrary_block_type.is_none() {
                    arbitrary_block_depth += 1;
                }
            }
            '\'' | '"' | '`' => {
                if at_arbitrary_block_start {
                    quoted_arbitrary_block_type = Quote::from_char(c);
                } else if let CharKind::Backslash = last_char {
                    // Escaped, ignore.
                } else {
                    let quote = Quote::from_char(c)
                        .expect("TODO: error message (this should never happen)");
                    next_last_char = CharKind::Quote(quote);
                }
            }
            '\\' => {
                if let CharKind::Backslash = last_char {
                    // Consider escaped backslashes as other characters.
                } else {
                    next_last_char = CharKind::Backslash;
                }
            }
            ']' => {
                if arbitrary_block_depth > 0 {
                    match &quoted_arbitrary_block_type {
                        // If in quoted arbitrary block...
                        Some(quote_type) => {
                            // and the last character was a quote...
                            if let CharKind::Quote(last_quote) = &last_char {
                                // of the same type as the current quote...
                                if quote_type == last_quote {
                                    // then we are no longer in an arbitrary block.
                                    arbitrary_block_depth = 0;
                                    quoted_arbitrary_block_type = None;
                                }
                            }
                        }
                        None => {
                            arbitrary_block_depth -= 1;
                            quoted_arbitrary_block_type = None;
                        }
                    }
                }
            }
            ':' => {
                if arbitrary_block_depth == 0 {
                    delimiter_indexes.push(index);
                }
            }
            _ => {}
        };
        if arbitrary_block_depth < 0 {
            panic!("TODO: error message (this should never happen)");
        };
        if at_arbitrary_block_start && !is_start_of_arbitrary_block {
            at_arbitrary_block_start = false;
        };
        last_char = next_last_char;
    }
    let mut variants: Vec<ClassSegmentStructure> = split_at_indexes(class_name, &delimiter_indexes)
        .iter()
        .map(|&s| ClassSegmentStructure {
            arbitrary: s.starts_with('['),
            text: s.to_string(),
        })
        .collect();
    let utility = variants.pop()?;

    Some(ClassStructure { variants, utility })
}

// TODO: unit tests.
