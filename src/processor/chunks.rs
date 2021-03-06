//! Utilities for dealing with annotated strings.
//!
//! This module contains the `split` and `join` function to destructure and recombine strings by
//! redaction remarks. This allows to quickly inspect modified sections of a string.
//!
//! ### Example
//!
//! ```
//! use general::types::{Meta, Remark, RemarkType};
//! use general::processor;
//!
//! let remarks = vec![Remark::with_range(
//!     RemarkType::Substituted,
//!     "myrule",
//!     (7, 17),
//! )];
//!
//! let chunks = processor::split_chunks("Hello, [redacted]!", &remarks);
//! let (joined, join_remarks) = processor::join_chunks(chunks);
//!
//! assert_eq!(joined, "Hello, [redacted]!");
//! assert_eq!(join_remarks, remarks);
//! ```

use std::fmt;

use crate::types::{Remark, RemarkType};

/// A type for dealing with chunks of annotated text.
#[derive(Clone, Debug, PartialEq)]
pub enum Chunk {
    /// Unmodified text chunk.
    Text {
        /// The text value of the chunk
        text: String,
    },
    /// Redacted text chunk with a note.
    Redaction {
        /// The redacted text value
        text: String,
        /// The rule that crated this redaction
        rule_id: String,
        /// Type type of remark for this redaction
        ty: RemarkType,
    },
}

impl Chunk {
    /// The text of this chunk.
    pub fn as_str(&self) -> &str {
        match *self {
            Chunk::Text { ref text } => &text,
            Chunk::Redaction { ref text, .. } => &text,
        }
    }

    /// Effective length of the text in this chunk.
    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    /// The number of chars in this chunk.
    pub fn chars(&self) -> usize {
        self.as_str().chars().count()
    }

    /// Determines whether this chunk is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Chunks the given text based on remarks.
pub fn split_chunks<'a, S, I>(text: S, remarks: I) -> Vec<Chunk>
where
    S: AsRef<str>,
    I: IntoIterator<Item = &'a Remark>,
{
    let text = text.as_ref();

    let mut rv = vec![];
    let mut pos = 0;

    for remark in remarks {
        let (from, to) = match remark.range() {
            Some(range) => *range,
            None => continue,
        };

        if from > pos {
            if let Some(piece) = text.get(pos..from) {
                rv.push(Chunk::Text {
                    text: piece.to_string(),
                });
            } else {
                break;
            }
        }
        if let Some(piece) = text.get(from..to) {
            rv.push(Chunk::Redaction {
                text: piece.to_string(),
                rule_id: remark.rule_id().into(),
                ty: remark.ty(),
            });
        } else {
            break;
        }
        pos = to;
    }

    if pos < text.len() {
        if let Some(piece) = text.get(pos..) {
            rv.push(Chunk::Text {
                text: piece.to_string(),
            });
        }
    }

    rv
}

/// Concatenates chunks into a string and emits remarks for redacted sections.
pub fn join_chunks<I>(chunks: I) -> (String, Vec<Remark>)
where
    I: IntoIterator<Item = Chunk>,
{
    let mut rv = String::new();
    let mut remarks = vec![];
    let mut pos = 0;

    for chunk in chunks {
        let new_pos = pos + chunk.len();
        rv.push_str(chunk.as_str());

        match chunk {
            Chunk::Redaction { rule_id, ty, .. } => {
                remarks.push(Remark::with_range(ty, rule_id.clone(), (pos, new_pos)))
            }
            Chunk::Text { .. } => {
                // Plain text segments do not need remarks
            }
        }

        pos = new_pos;
    }

    (rv, remarks)
}

#[test]
fn test_chunk_split() {
    let remarks = vec![Remark::with_range(
        RemarkType::Masked,
        "@email:strip",
        (33, 47),
    )];

    let chunks = vec![
        Chunk::Text {
            text: "Hello Peter, my email address is ".into(),
        },
        Chunk::Redaction {
            ty: RemarkType::Masked,
            text: "****@*****.com".into(),
            rule_id: "@email:strip".into(),
        },
        Chunk::Text {
            text: ". See you".into(),
        },
    ];

    assert_eq_dbg!(
        split_chunks(
            "Hello Peter, my email address is ****@*****.com. See you",
            &remarks,
        ),
        chunks
    );

    assert_eq_dbg!(
        join_chunks(chunks),
        (
            "Hello Peter, my email address is ****@*****.com. See you".into(),
            remarks
        )
    );
}
