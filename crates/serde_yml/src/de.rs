// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! YAML deserialization: parser and `from_str`.

use crate::{
    error::Error,
    mapping::Mapping,
    number::Number,
    value::{tagged::TaggedValue, Value},
};
use serde::{
    de::{DeserializeOwned, MapAccess, SeqAccess, Visitor},
    forward_to_deserialize_any,
};
use std::io::Read;

type Result<T> = std::result::Result<T, Error>;

// ---- Public API ----

/// Deserialize an instance of type `T` from a YAML string.
pub fn from_str<T>(s: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let value = parse_yaml(s)?;
    T::deserialize(value)
}

/// Deserialize from a byte slice.
pub fn from_slice<T>(v: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    let s = std::str::from_utf8(v)
        .map_err(|e| Error::msg(e.to_string()))?;
    from_str(s)
}

/// Deserialize from a reader.
pub fn from_reader<R, T>(mut rdr: R) -> Result<T>
where
    R: Read,
    T: DeserializeOwned,
{
    let mut s = String::new();
    rdr.read_to_string(&mut s)
        .map_err(|e| Error::msg(e.to_string()))?;
    from_str(&s)
}

// Re-export Deserializer as a type (minimal stub for API
// compatibility). Real deserialization goes through
// from_str -> parser -> Value -> T.
/// A YAML deserializer.
#[derive(Debug)]
pub struct Deserializer {
    _private: (),
}

// ---- YAML Parser ----

/// Parse a YAML string into a `Value`.
fn parse_yaml(input: &str) -> Result<Value> {
    let mut parser = Parser::new(input);
    parser.skip_blanks_and_comments();
    // Skip document start marker
    if parser.rest().starts_with("---") {
        parser.advance_by(3);
        parser.skip_to_eol();
    }
    let value = parser.parse_value(0)?;
    // Skip document end marker
    parser.skip_blanks_and_comments();
    if parser.rest().starts_with("...") {
        parser.advance_by(3);
    }
    Ok(value)
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, pos: 0 }
    }

    fn rest(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn advance_by(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.input.len());
    }

    fn skip_to_eol(&mut self) {
        if let Some(idx) = self.rest().find('\n') {
            self.advance_by(idx + 1);
        } else {
            self.pos = self.input.len();
        }
    }

    fn skip_blanks_and_comments(&mut self) {
        loop {
            // Skip whitespace (spaces, tabs, newlines)
            while let Some(ch) = self.peek() {
                if ch == ' ' || ch == '\t' || ch == '\n'
                    || ch == '\r'
                {
                    self.advance_by(ch.len_utf8());
                } else {
                    break;
                }
            }
            // Skip comment lines
            if self.peek() == Some('#') {
                self.skip_to_eol();
            } else {
                break;
            }
        }
    }

    fn peek_line_indent(&self) -> usize {
        let rest = self.rest();
        rest.len() - rest.trim_start_matches(' ').len()
    }

    fn current_line(&self) -> &'a str {
        let rest = self.rest();
        let end = rest
            .find('\n')
            .unwrap_or(rest.len());
        &rest[..end]
    }

    fn parse_value(
        &mut self,
        min_indent: usize,
    ) -> Result<Value> {
        self.skip_blanks_and_comments();
        if self.is_eof() {
            return Ok(Value::Null);
        }

        let indent = self.peek_line_indent();
        if indent < min_indent {
            return Ok(Value::Null);
        }

        // Move past leading spaces
        let rest = self.rest().trim_start_matches(' ');
        let first_char = match rest.chars().next() {
            Some(c) => c,
            None => return Ok(Value::Null),
        };

        match first_char {
            // Flow sequence
            '[' => {
                self.advance_to_content();
                self.parse_flow_sequence()
            }
            // Flow mapping
            '{' => {
                self.advance_to_content();
                self.parse_flow_mapping()
            }
            // Block sequence
            '-' if self.is_sequence_dash(indent) => {
                self.parse_block_sequence(indent)
            }
            // Tag
            '!' => {
                self.advance_to_content();
                self.parse_tagged_value(indent)
            }
            // Quoted string
            '\'' | '"' => {
                self.advance_to_content();
                let s = self.parse_quoted_string(
                    first_char,
                )?;
                // Check if this is a mapping key
                self.skip_inline_spaces();
                if self.peek() == Some(':')
                    && self.is_mapping_colon()
                {
                    self.parse_mapping_from_first_key(
                        Value::String(s),
                        indent,
                    )
                } else {
                    Ok(Value::String(s))
                }
            }
            // Block scalar
            '|' | '>' => {
                self.advance_to_content();
                self.parse_block_scalar(
                    first_char, indent,
                )
            }
            // Mapping or plain scalar
            _ => {
                if self.line_has_mapping_colon(indent) {
                    self.parse_block_mapping(indent)
                } else {
                    self.advance_to_content();
                    self.parse_plain_scalar()
                }
            }
        }
    }

    fn advance_to_content(&mut self) {
        let indent = self.peek_line_indent();
        self.advance_by(indent);
    }

    fn skip_inline_spaces(&mut self) {
        while self.peek() == Some(' ')
            || self.peek() == Some('\t')
        {
            self.advance_by(1);
        }
    }

    fn is_sequence_dash(
        &self,
        expected_indent: usize,
    ) -> bool {
        let rest = self.rest();
        let indent = rest.len()
            - rest.trim_start_matches(' ').len();
        if indent != expected_indent {
            return false;
        }
        let trimmed = rest.trim_start_matches(' ');
        trimmed.starts_with("- ")
            || trimmed == "-"
            || trimmed.starts_with("-\n")
            || trimmed.starts_with("-\r")
    }

    fn is_mapping_colon(&self) -> bool {
        let rest = self.rest();
        rest.starts_with(": ")
            || rest == ":"
            || rest.starts_with(":\n")
            || rest.starts_with(":\r")
    }

    fn line_has_mapping_colon(
        &self,
        expected_indent: usize,
    ) -> bool {
        let rest = self.rest();
        let indent = rest.len()
            - rest.trim_start_matches(' ').len();
        if indent != expected_indent {
            return false;
        }
        let trimmed = rest.trim_start_matches(' ');
        let line = match trimmed.find('\n') {
            Some(i) => &trimmed[..i],
            None => trimmed,
        };
        // Check for key: value pattern (not inside quotes)
        self.find_mapping_colon(line).is_some()
    }

    fn find_mapping_colon(
        &self,
        line: &str,
    ) -> Option<usize> {
        let mut in_single = false;
        let mut in_double = false;
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'\'' if !in_double => {
                    in_single = !in_single;
                }
                b'"' if !in_single => {
                    in_double = !in_double;
                }
                b':' if !in_single && !in_double => {
                    // Must be followed by space, EOL,
                    // or be at end
                    if i + 1 >= bytes.len()
                        || bytes[i + 1] == b' '
                        || bytes[i + 1] == b'\t'
                    {
                        return Some(i);
                    }
                }
                b'#' if !in_single && !in_double => {
                    // Rest is comment
                    return None;
                }
                _ => {}
            }
            i += 1;
        }
        None
    }

    fn parse_block_mapping(
        &mut self,
        indent: usize,
    ) -> Result<Value> {
        let mut mapping = Mapping::new();
        loop {
            self.skip_blanks_and_comments();
            if self.is_eof() {
                break;
            }
            let cur_indent = self.peek_line_indent();
            if cur_indent != indent {
                break;
            }
            // Check for document end
            let rest_trimmed =
                self.rest().trim_start_matches(' ');
            if rest_trimmed.starts_with("---")
                || rest_trimmed.starts_with("...")
            {
                break;
            }

            self.advance_by(indent);
            let (key, value) =
                self.parse_mapping_entry(indent)?;
            mapping.insert(key, value);
        }
        Ok(Value::Mapping(mapping))
    }

    fn parse_mapping_entry(
        &mut self,
        indent: usize,
    ) -> Result<(Value, Value)> {
        let key = self.parse_mapping_key()?;
        // Skip ':'
        if self.peek() == Some(':') {
            self.advance_by(1);
        }
        self.skip_inline_spaces();

        let value = if self.peek() == Some('\n')
            || self.peek() == Some('\r')
            || self.peek() == Some('#')
            || self.is_eof()
        {
            // Skip comment
            if self.peek() == Some('#') {
                self.skip_to_eol();
            }
            // Value on next line(s)
            self.parse_value(indent + 1)?
        } else {
            self.parse_inline_value(indent)?
        };

        Ok((key, value))
    }

    fn parse_mapping_key(&mut self) -> Result<Value> {
        match self.peek() {
            Some('\'') | Some('"') => {
                let q = self.peek().ok_or_else(|| {
                    Error::msg("unexpected EOF")
                })?;
                let s = self.parse_quoted_string(q)?;
                Ok(Value::String(s))
            }
            _ => {
                // Read until ':'
                let rest = self.rest();
                let line = match rest.find('\n') {
                    Some(i) => &rest[..i],
                    None => rest,
                };
                let colon_pos = self
                    .find_mapping_colon(line)
                    .ok_or_else(|| {
                        Error::msg(format!(
                            "expected ':' in mapping, \
                             got: {:?}",
                            &line
                                [..line.len().min(40)]
                        ))
                    })?;
                let key_str =
                    rest[..colon_pos].trim_end();
                let key = interpret_scalar(key_str);
                self.advance_by(colon_pos);
                Ok(key)
            }
        }
    }

    fn parse_mapping_from_first_key(
        &mut self,
        first_key: Value,
        indent: usize,
    ) -> Result<Value> {
        // We already have the key, now skip ':'
        if self.peek() == Some(':') {
            self.advance_by(1);
        }
        self.skip_inline_spaces();

        let first_value = if self.peek() == Some('\n')
            || self.peek() == Some('\r')
            || self.peek() == Some('#')
            || self.is_eof()
        {
            if self.peek() == Some('#') {
                self.skip_to_eol();
            }
            self.parse_value(indent + 1)?
        } else {
            self.parse_inline_value(indent)?
        };

        let mut mapping = Mapping::new();
        mapping.insert(first_key, first_value);

        // Continue parsing remaining entries
        loop {
            self.skip_blanks_and_comments();
            if self.is_eof() {
                break;
            }
            let cur_indent = self.peek_line_indent();
            if cur_indent != indent {
                break;
            }
            let rest_trimmed =
                self.rest().trim_start_matches(' ');
            if rest_trimmed.starts_with("---")
                || rest_trimmed.starts_with("...")
            {
                break;
            }
            self.advance_by(indent);
            let (key, value) =
                self.parse_mapping_entry(indent)?;
            mapping.insert(key, value);
        }
        Ok(Value::Mapping(mapping))
    }

    fn parse_inline_value(
        &mut self,
        parent_indent: usize,
    ) -> Result<Value> {
        match self.peek() {
            Some('[') => self.parse_flow_sequence(),
            Some('{') => self.parse_flow_mapping(),
            Some('\'') | Some('"') => {
                let q = self.peek().ok_or_else(|| {
                    Error::msg("unexpected EOF")
                })?;
                let s = self.parse_quoted_string(q)?;
                Ok(Value::String(s))
            }
            Some('|') | Some('>') => {
                let ch = self.peek().ok_or_else(|| {
                    Error::msg("unexpected EOF")
                })?;
                self.parse_block_scalar(
                    ch,
                    parent_indent,
                )
            }
            Some('!') => {
                self.parse_tagged_value(parent_indent)
            }
            _ => self.parse_plain_scalar(),
        }
    }

    fn parse_block_sequence(
        &mut self,
        indent: usize,
    ) -> Result<Value> {
        let mut items = Vec::new();
        loop {
            self.skip_blanks_and_comments();
            if self.is_eof() {
                break;
            }
            let cur_indent = self.peek_line_indent();
            if cur_indent != indent {
                break;
            }
            let rest_trimmed =
                self.rest().trim_start_matches(' ');
            if !rest_trimmed.starts_with("- ")
                && rest_trimmed != "-"
                && !rest_trimmed.starts_with("-\n")
                && !rest_trimmed.starts_with("-\r")
            {
                break;
            }
            // Skip indent + "- "
            self.advance_by(indent);
            self.advance_by(1); // '-'
            if self.peek() == Some(' ') {
                self.advance_by(1);
            }

            self.skip_inline_spaces();
            // Check if rest of line is empty (value on
            // next line)
            if self.peek() == Some('\n')
                || self.peek() == Some('\r')
                || self.peek() == Some('#')
                || self.is_eof()
            {
                if self.peek() == Some('#') {
                    self.skip_to_eol();
                }
                let item =
                    self.parse_value(indent + 2)?;
                items.push(item);
            } else {
                // Inline value after "- "
                // Check if it's a mapping
                let line = self.current_line();
                if self
                    .find_mapping_colon(line)
                    .is_some()
                {
                    // It's a mapping starting on this
                    // line. The indent for this mapping
                    // is indent + 2.
                    let item_indent = indent + 2;
                    let (key, value) = self
                        .parse_mapping_entry(
                            item_indent,
                        )?;
                    let mut m = Mapping::new();
                    m.insert(key, value);
                    // Continue reading more mapping
                    // entries at item_indent
                    loop {
                        self.skip_blanks_and_comments();
                        if self.is_eof() {
                            break;
                        }
                        let ci =
                            self.peek_line_indent();
                        if ci != item_indent {
                            break;
                        }
                        let rt = self
                            .rest()
                            .trim_start_matches(' ');
                        if !self
                            .find_mapping_colon(
                                match rt.find('\n') {
                                    Some(i) => {
                                        &rt[..i]
                                    }
                                    None => rt,
                                },
                            )
                            .is_some()
                        {
                            break;
                        }
                        self.advance_by(item_indent);
                        let (k, v) = self
                            .parse_mapping_entry(
                                item_indent,
                            )?;
                        m.insert(k, v);
                    }
                    items.push(Value::Mapping(m));
                } else {
                    let item =
                        self.parse_inline_value(indent)?;
                    items.push(item);
                }
            }
        }
        Ok(Value::Sequence(items))
    }

    fn parse_flow_sequence(&mut self) -> Result<Value> {
        // Skip '['
        self.advance_by(1);
        let mut items = Vec::new();
        loop {
            self.skip_flow_whitespace();
            match self.peek() {
                None => {
                    return Err(Error::msg(
                        "unterminated flow sequence",
                    ))
                }
                Some(']') => {
                    self.advance_by(1);
                    break;
                }
                Some(',') => {
                    self.advance_by(1);
                    continue;
                }
                _ => {}
            }
            let item = self.parse_flow_value()?;
            items.push(item);
        }
        Ok(Value::Sequence(items))
    }

    fn parse_flow_mapping(&mut self) -> Result<Value> {
        // Skip '{'
        self.advance_by(1);
        let mut mapping = Mapping::new();
        loop {
            self.skip_flow_whitespace();
            match self.peek() {
                None => {
                    return Err(Error::msg(
                        "unterminated flow mapping",
                    ))
                }
                Some('}') => {
                    self.advance_by(1);
                    break;
                }
                Some(',') => {
                    self.advance_by(1);
                    continue;
                }
                _ => {}
            }
            // Parse key
            let key = self.parse_flow_value()?;
            self.skip_flow_whitespace();
            if self.peek() != Some(':') {
                return Err(Error::msg(
                    "expected ':' in flow mapping",
                ));
            }
            self.advance_by(1);
            self.skip_flow_whitespace();
            // Parse value
            let value = self.parse_flow_value()?;
            mapping.insert(key, value);
        }
        Ok(Value::Mapping(mapping))
    }

    fn parse_flow_value(&mut self) -> Result<Value> {
        self.skip_flow_whitespace();
        match self.peek() {
            Some('[') => self.parse_flow_sequence(),
            Some('{') => self.parse_flow_mapping(),
            Some('\'') | Some('"') => {
                let q = self.peek().ok_or_else(|| {
                    Error::msg("unexpected EOF")
                })?;
                let s = self.parse_quoted_string(q)?;
                Ok(Value::String(s))
            }
            _ => {
                // Read until , ] } : or newline
                let rest = self.rest();
                let mut end = rest.len();
                for (i, ch) in rest.char_indices() {
                    if ch == ','
                        || ch == ']'
                        || ch == '}'
                        || ch == ':'
                    {
                        end = i;
                        break;
                    }
                }
                let token =
                    rest[..end].trim_end();
                if token.is_empty() {
                    return Ok(Value::Null);
                }
                let value = interpret_scalar(token);
                self.advance_by(end);
                Ok(value)
            }
        }
    }

    fn skip_flow_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' '
                || ch == '\t'
                || ch == '\n'
                || ch == '\r'
            {
                self.advance_by(ch.len_utf8());
            } else if ch == '#' {
                self.skip_to_eol();
            } else {
                break;
            }
        }
    }

    fn parse_quoted_string(
        &mut self,
        quote: char,
    ) -> Result<String> {
        // Skip opening quote
        self.advance_by(1);
        let mut result = String::new();
        loop {
            match self.peek() {
                None => {
                    return Err(Error::msg(
                        "unterminated quoted string",
                    ))
                }
                Some(ch) if ch == quote => {
                    self.advance_by(1);
                    // For single quotes, check for ''
                    if quote == '\''
                        && self.peek() == Some('\'')
                    {
                        result.push('\'');
                        self.advance_by(1);
                        continue;
                    }
                    break;
                }
                Some('\\') if quote == '"' => {
                    self.advance_by(1);
                    match self.peek() {
                        Some('n') => {
                            result.push('\n');
                            self.advance_by(1);
                        }
                        Some('t') => {
                            result.push('\t');
                            self.advance_by(1);
                        }
                        Some('r') => {
                            result.push('\r');
                            self.advance_by(1);
                        }
                        Some('\\') => {
                            result.push('\\');
                            self.advance_by(1);
                        }
                        Some('"') => {
                            result.push('"');
                            self.advance_by(1);
                        }
                        Some('/') => {
                            result.push('/');
                            self.advance_by(1);
                        }
                        Some(c) => {
                            result.push('\\');
                            result.push(c);
                            self.advance_by(
                                c.len_utf8(),
                            );
                        }
                        None => {
                            result.push('\\');
                        }
                    }
                }
                Some(ch) => {
                    result.push(ch);
                    self.advance_by(ch.len_utf8());
                }
            }
        }
        Ok(result)
    }

    fn parse_block_scalar(
        &mut self,
        style: char,
        _parent_indent: usize,
    ) -> Result<Value> {
        // Skip '|' or '>'
        self.advance_by(1);
        // Check for chomping indicator
        let chomp = match self.peek() {
            Some('-') => {
                self.advance_by(1);
                Chomp::Strip
            }
            Some('+') => {
                self.advance_by(1);
                Chomp::Keep
            }
            _ => Chomp::Clip,
        };
        // Skip rest of indicator line
        self.skip_to_eol();

        // Determine content indent from first
        // non-empty line
        let content_indent = {
            let mut look = self.pos;
            loop {
                if look >= self.input.len() {
                    break 0;
                }
                let rem = &self.input[look..];
                let line = match rem.find('\n') {
                    Some(i) => &rem[..i],
                    None => rem,
                };
                if line.trim().is_empty() {
                    look += line.len() + 1;
                    continue;
                }
                break line.len()
                    - line
                        .trim_start_matches(' ')
                        .len();
            }
        };

        if content_indent == 0 {
            return Ok(Value::String(String::new()));
        }

        let mut lines = Vec::new();
        loop {
            if self.is_eof() {
                break;
            }
            let line_rest = self.rest();
            let line = match line_rest.find('\n') {
                Some(i) => &line_rest[..i],
                None => line_rest,
            };

            if line.trim().is_empty() {
                lines.push(String::new());
                self.advance_by(line.len());
                if self.peek() == Some('\n') {
                    self.advance_by(1);
                }
                continue;
            }

            let line_indent = line.len()
                - line.trim_start_matches(' ').len();
            if line_indent < content_indent {
                break;
            }

            lines.push(
                line[content_indent..].to_owned(),
            );
            self.advance_by(line.len());
            if self.peek() == Some('\n') {
                self.advance_by(1);
            }
        }

        // Remove trailing empty lines for processing
        let trailing_empties = lines
            .iter()
            .rev()
            .take_while(|l| l.is_empty())
            .count();

        let content = if style == '|' {
            // Literal: preserve newlines
            lines.join("\n")
        } else {
            // Folded: join with spaces (preserve double
            // newlines)
            let mut result = String::new();
            for (i, line) in lines.iter().enumerate() {
                if i > 0 {
                    if line.is_empty()
                        || lines[i - 1].is_empty()
                    {
                        result.push('\n');
                    } else {
                        result.push(' ');
                    }
                }
                result.push_str(line);
            }
            result
        };

        let content = match chomp {
            Chomp::Strip => {
                content.trim_end_matches('\n').to_owned()
            }
            Chomp::Clip => {
                let trimmed = content
                    .trim_end_matches('\n')
                    .to_owned();
                if trailing_empties > 0 || !content.ends_with('\n') {
                    trimmed + "\n"
                } else {
                    trimmed
                }
            }
            Chomp::Keep => content,
        };

        Ok(Value::String(content))
    }

    fn parse_plain_scalar(&mut self) -> Result<Value> {
        let rest = self.rest();
        let line = match rest.find('\n') {
            Some(i) => &rest[..i],
            None => rest,
        };
        // Strip inline comment
        let effective =
            strip_inline_comment(line).trim_end();
        if effective.is_empty() {
            self.skip_to_eol();
            return Ok(Value::Null);
        }
        let value = interpret_scalar(effective);
        self.advance_by(line.len());
        if self.peek() == Some('\n') {
            self.advance_by(1);
        }
        Ok(value)
    }

    fn parse_tagged_value(
        &mut self,
        indent: usize,
    ) -> Result<Value> {
        // Skip '!'
        self.advance_by(1);
        // Read tag name
        let rest = self.rest();
        let end = rest
            .find(|c: char| {
                c == ' ' || c == '\n' || c == '\r'
            })
            .unwrap_or(rest.len());
        let tag_name = rest[..end].to_owned();
        self.advance_by(end);
        self.skip_inline_spaces();

        let tag = crate::value::tagged::Tag::new(
            format!("!{}", tag_name),
        );

        // Parse the tagged value
        let value = if self.peek() == Some('\n')
            || self.peek() == Some('\r')
            || self.is_eof()
        {
            self.parse_value(indent + 1)?
        } else {
            self.parse_inline_value(indent)?
        };

        Ok(Value::Tagged(Box::new(TaggedValue {
            tag,
            value,
        })))
    }
}

#[derive(Clone, Copy)]
enum Chomp {
    Strip,
    Clip,
    Keep,
}

fn strip_inline_comment(line: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' if !in_double => {
                in_single = !in_single;
            }
            b'"' if !in_single => {
                in_double = !in_double;
            }
            b' ' if !in_single && !in_double => {
                if i + 1 < bytes.len()
                    && bytes[i + 1] == b'#'
                {
                    return &line[..i];
                }
            }
            _ => {}
        }
        i += 1;
    }
    line
}

/// Interpret a plain (unquoted) scalar string as the
/// appropriate YAML type.
fn interpret_scalar(s: &str) -> Value {
    match s {
        "" | "null" | "Null" | "NULL" | "~" => {
            Value::Null
        }
        "true" | "True" | "TRUE" => Value::Bool(true),
        "false" | "False" | "FALSE" => {
            Value::Bool(false)
        }
        ".nan" | ".NaN" | ".NAN" => {
            Value::Number(Number::from(f64::NAN))
        }
        ".inf" | ".Inf" | ".INF" => {
            Value::Number(Number::from(f64::INFINITY))
        }
        "-.inf" | "-.Inf" | "-.INF" => {
            Value::Number(Number::from(f64::NEG_INFINITY))
        }
        _ => {
            // Try integer
            if let Some(n) = parse_integer(s) {
                return n;
            }
            // Try float
            if let Some(n) = parse_float(s) {
                return n;
            }
            Value::String(s.to_owned())
        }
    }
}

fn parse_integer(s: &str) -> Option<Value> {
    if s.starts_with("0x") || s.starts_with("0X") {
        u64::from_str_radix(&s[2..], 16)
            .ok()
            .map(|n| Value::Number(Number::from(n)))
    } else if s.starts_with("0o") || s.starts_with("0O")
    {
        u64::from_str_radix(&s[2..], 8)
            .ok()
            .map(|n| Value::Number(Number::from(n)))
    } else if s.starts_with('-') || s.starts_with('+') {
        s.parse::<i64>()
            .ok()
            .map(|n| Value::Number(Number::from(n)))
    } else {
        // Only parse as integer if it's all digits
        // (or digits with underscores)
        let clean = s.replace('_', "");
        if clean.chars().all(|c| c.is_ascii_digit()) {
            clean
                .parse::<u64>()
                .ok()
                .map(|n| Value::Number(Number::from(n)))
        } else {
            None
        }
    }
}

fn parse_float(s: &str) -> Option<Value> {
    // Must contain a '.' or 'e'/'E' to be a float
    if !s.contains('.')
        && !s.contains('e')
        && !s.contains('E')
    {
        return None;
    }
    let clean = s.replace('_', "");
    clean
        .parse::<f64>()
        .ok()
        .map(|f| Value::Number(Number::from(f)))
}

// ---- SeqDeserializer / MapDeserializer ----
// Used by Value's Deserializer impl.

pub(crate) struct SeqDeserializer {
    iter: std::vec::IntoIter<Value>,
}

impl SeqDeserializer {
    pub(crate) fn new(seq: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: seq.into_iter(),
        }
    }
}

impl<'de> serde::Deserializer<'de> for SeqDeserializer {
    type Error = Error;

    fn deserialize_any<V>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        char str string bytes byte_buf option unit
        unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => {
                seed.deserialize(value).map(Some)
            }
            None => Ok(None),
        }
    }
}

pub(crate) struct MapDeserializer {
    iter: crate::mapping::IntoIter,
    value: Option<Value>,
}

impl MapDeserializer {
    pub(crate) fn new(mapping: Mapping) -> Self {
        MapDeserializer {
            iter: mapping.into_iter(),
            value: None,
        }
    }
}

impl<'de> serde::Deserializer<'de> for MapDeserializer {
    type Error = Error;

    fn deserialize_any<V>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        char str string bytes byte_buf option unit
        unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<K>(
        &mut self,
        seed: K,
    ) -> std::result::Result<Option<K::Value>, Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(key).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(
        &mut self,
        seed: V,
    ) -> std::result::Result<V::Value, Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(Error::msg(
                "value called before key",
            )),
        }
    }
}
