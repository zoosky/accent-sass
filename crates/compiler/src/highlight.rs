//! Source frames with labelled highlights, drawn the way dart-sass draws them.
//!
//! A deprecation warning points at more than one place: the selector at
//! fault, and the declaration that makes it a problem. dart-sass draws those
//! with the `source_span` package's `Highlighter`, and this module ports the
//! part of it a warning in one file uses: the sidebar with line numbers, a
//! `^` underline for the primary span and `━` for the others, labels
//! (including labels that run over several lines), `...` in place of lines
//! that are skipped, and the corner-and-bar drawing for a span that covers
//! more than one line.
//!
//! What is not ported: highlights in more than one file, and colour.

use codemap::SpanLoc;

/// One span to draw, with the text written after its underline.
pub(crate) struct Highlight {
    /// Where the highlight starts and ends.
    pub loc: SpanLoc,
    /// Text written after the underline. A newline continues it on a further
    /// line, aligned under the first.
    pub label: Option<String>,
    /// Whether this is the span the message is about. It is underlined with
    /// `^` and drawn first on its line.
    pub primary: bool,
}

/// The characters a frame is drawn with.
struct Glyphs {
    down_end: &'static str,
    up_end: &'static str,
    vertical: &'static str,
    horizontal: &'static str,
    horizontal_bold: &'static str,
    top_left: &'static str,
    bottom_left: &'static str,
    cross: &'static str,
    open: &'static str,
    open_again: &'static str,
    close: &'static str,
}

const UNICODE: Glyphs = Glyphs {
    down_end: "╷",
    up_end: "╵",
    vertical: "│",
    horizontal: "─",
    horizontal_bold: "━",
    top_left: "┌",
    bottom_left: "└",
    cross: "┼",
    open: "┌",
    open_again: "┬",
    close: "└",
};

const ASCII: Glyphs = Glyphs {
    down_end: ",",
    up_end: "'",
    vertical: "|",
    horizontal: "-",
    horizontal_bold: "=",
    top_left: ",",
    bottom_left: "'",
    cross: "+",
    open: "/",
    open_again: "/",
    close: "\\",
};

/// A hard tab is drawn as this many spaces, so columns stay aligned.
const SPACES_PER_TAB: usize = 4;

/// A source line in the frame, and the highlights that touch it.
struct Line {
    /// 0-based line number.
    number: usize,
    text: String,
    /// Indices into the sorted highlights.
    highlights: Vec<usize>,
}

/// Draws `highlights` as a frame, from the opening `╷` to the closing `╵`,
/// with no trailing newline.
///
/// All highlights must be in the same file. An empty list draws nothing.
pub(crate) fn highlight(highlights: Vec<Highlight>, unicode: bool) -> String {
    let mut highlights = highlights
        .into_iter()
        .map(normalize_end_of_line)
        .collect::<Vec<_>>();
    if highlights.is_empty() {
        return String::new();
    }
    highlights.sort_by_key(|h| {
        (
            h.loc.begin.line,
            h.loc.begin.column,
            h.loc.end.line,
            h.loc.end.column,
        )
    });

    let lines = collate_lines(&highlights);
    let glyphs = if unicode { &UNICODE } else { &ASCII };

    let contiguous = lines
        .windows(2)
        .all(|pair| pair[0].number + 1 == pair[1].number);
    let last_number = lines.last().map_or(0, |line| line.number);
    let padding = 1
        + (last_number + 1)
            .to_string()
            .len()
            .max(if contiguous { 0 } else { 3 });
    let max_multiline = lines
        .iter()
        .map(|line| {
            line.highlights
                .iter()
                .filter(|&&h| is_multiline(&highlights[h]))
                .count()
        })
        .max()
        .unwrap_or(0);

    let mut frame = Frame {
        buffer: String::new(),
        glyphs,
        padding,
        highlights: &highlights,
        columns: vec![None; max_multiline],
    };

    frame.sidebar(None, Some(glyphs.down_end));
    frame.buffer.push('\n');

    for (i, line) in lines.iter().enumerate() {
        if i > 0 && lines[i - 1].number + 1 != line.number {
            frame.sidebar(Some("..."), None);
            frame.buffer.push('\n');
        }

        // A multi-line highlight that starts at the first non-blank character
        // of its line takes its column before the line is written, so its bar
        // runs down from here.
        for &h in line.highlights.iter().rev() {
            let loc = &highlights[h].loc;
            if is_multiline(&highlights[h])
                && loc.begin.line == line.number
                && is_only_whitespace(&prefix(&line.text, loc.begin.column))
                && !frame.columns.contains(&Some(h))
            {
                replace_first_none(&mut frame.columns, h);
            }
        }

        frame.sidebar(Some(&(line.number + 1).to_string()), None);
        frame.buffer.push(' ');
        frame.multiline_bars(line, None);
        if !frame.columns.is_empty() {
            frame.buffer.push(' ');
        }
        frame.text(&line.text);
        frame.buffer.push('\n');

        let primary = line
            .highlights
            .iter()
            .copied()
            .find(|&h| highlights[h].primary);
        if let Some(primary) = primary {
            frame.indicator(line, primary);
        }
        for &h in &line.highlights {
            if Some(h) != primary {
                frame.indicator(line, h);
            }
        }
    }

    frame.sidebar(None, Some(glyphs.up_end));
    frame.buffer
}

/// The frame being drawn, and the multi-line highlights whose bars are open.
struct Frame<'a> {
    buffer: String,
    glyphs: &'static Glyphs,
    padding: usize,
    highlights: &'a [Highlight],
    /// One slot per multi-line highlight that can be open at once.
    columns: Vec<Option<usize>>,
}

impl Frame<'_> {
    /// Writes the line-number gutter and its bar, or `end` in place of the bar.
    fn sidebar(&mut self, text: Option<&str>, end: Option<&str>) {
        let text = text.unwrap_or("");
        self.buffer.push_str(text);
        for _ in text.chars().count()..self.padding {
            self.buffer.push(' ');
        }
        self.buffer.push_str(end.unwrap_or(self.glyphs.vertical));
    }

    /// Writes source text with hard tabs expanded.
    fn text(&mut self, text: &str) {
        for c in text.chars() {
            if c == '\t' {
                self.buffer.push_str(&" ".repeat(SPACES_PER_TAB));
            } else {
                self.buffer.push(c);
            }
        }
    }

    /// Writes the bars of open multi-line highlights for `line`. With
    /// `current`, draws the corner that starts or ends that highlight and a
    /// rule from it to the rightmost column.
    fn multiline_bars(&mut self, line: &Line, current: Option<usize>) {
        let glyphs = self.glyphs;
        let mut opened_on_this_line = false;
        let mut found_current = false;

        for slot in self.columns.clone() {
            let Some(h) = slot else {
                if found_current || opened_on_this_line {
                    self.buffer.push_str(glyphs.horizontal);
                } else {
                    self.buffer.push(' ');
                }
                continue;
            };
            let loc = &self.highlights[h].loc;

            if current == Some(h) {
                found_current = true;
                self.buffer.push_str(if loc.begin.line == line.number {
                    glyphs.top_left
                } else {
                    glyphs.bottom_left
                });
            } else if found_current {
                self.buffer.push_str(glyphs.cross);
            } else {
                let vertical = if opened_on_this_line {
                    glyphs.cross
                } else {
                    glyphs.vertical
                };
                if current.is_some() {
                    self.buffer.push_str(vertical);
                } else if loc.begin.line == line.number {
                    self.buffer.push_str(if opened_on_this_line {
                        glyphs.open_again
                    } else {
                        glyphs.open
                    });
                    opened_on_this_line = true;
                } else if loc.end.line == line.number && loc.end.column == line.text.chars().count()
                {
                    self.buffer.push_str(if self.highlights[h].label.is_none() {
                        glyphs.close
                    } else {
                        vertical
                    });
                } else {
                    self.buffer.push_str(vertical);
                }
            }
        }
    }

    /// Writes the line under `line` that marks where highlight `h` is.
    fn indicator(&mut self, line: &Line, h: usize) {
        let highlight = &self.highlights[h];
        let loc = highlight.loc.clone();

        if !is_multiline(highlight) {
            self.sidebar(None, None);
            self.buffer.push(' ');
            self.multiline_bars(line, Some(h));
            if !self.columns.is_empty() {
                self.buffer.push(' ');
            }
            let character = if highlight.primary {
                "^"
            } else {
                self.glyphs.horizontal_bold
            };
            let length = self.underline(line, loc.begin.column, loc.end.column, character);
            self.label(h, length);
        } else if loc.begin.line == line.number {
            if self.columns.contains(&Some(h)) {
                return;
            }
            replace_first_none(&mut self.columns, h);

            self.sidebar(None, None);
            self.buffer.push(' ');
            self.multiline_bars(line, Some(h));
            self.arrow(line, loc.begin.column, true);
            self.buffer.push('\n');
        } else if loc.end.line == line.number {
            let covers_whole_line = loc.end.column == line.text.chars().count();
            if covers_whole_line && highlight.label.is_none() {
                replace_with_none(&mut self.columns, h);
                return;
            }

            self.sidebar(None, None);
            self.buffer.push(' ');
            self.multiline_bars(line, Some(h));
            let length = if covers_whole_line {
                self.buffer.push_str(&self.glyphs.horizontal.repeat(3));
                3
            } else {
                self.arrow(line, loc.end.column.saturating_sub(1), false)
            };
            self.label(h, length);
            replace_with_none(&mut self.columns, h);
        }
    }

    /// Underlines columns `start..end` of `line`, at least one character
    /// wide, and returns how many characters it wrote including the indent.
    fn underline(&mut self, line: &Line, start: usize, end: usize, character: &str) -> usize {
        let tabs_before = count_tabs(&prefix(&line.text, start));
        let tabs_inside = count_tabs(&slice(&line.text, start, end));
        let start = start + tabs_before * (SPACES_PER_TAB - 1);
        let end = end + (tabs_before + tabs_inside) * (SPACES_PER_TAB - 1);
        let width = end.saturating_sub(start).max(1);

        self.buffer.push_str(&" ".repeat(start));
        self.buffer.push_str(&character.repeat(width));
        start + width
    }

    /// Writes a rule ending in `^` under `column`, and returns its length.
    fn arrow(&mut self, line: &Line, column: usize, beginning: bool) -> usize {
        let tabs = count_tabs(&prefix(&line.text, column + usize::from(!beginning)));
        let length = 1 + column + tabs * (SPACES_PER_TAB - 1);
        self.buffer.push_str(&self.glyphs.horizontal.repeat(length));
        self.buffer.push('^');
        length + 1
    }

    /// Writes the label of highlight `h` after an underline `length`
    /// characters long, ending the line.
    fn label(&mut self, h: usize, length: usize) {
        let Some(label) = self.highlights[h].label.clone() else {
            self.buffer.push('\n');
            return;
        };

        let mut lines = label.split('\n');
        self.buffer.push(' ');
        self.buffer.push_str(lines.next().unwrap_or(""));
        self.buffer.push('\n');

        for text in lines {
            self.sidebar(None, None);
            self.buffer.push(' ');
            for slot in self.columns.clone() {
                if slot.is_none() || slot == Some(h) {
                    self.buffer.push(' ');
                } else {
                    self.buffer.push_str(self.glyphs.vertical);
                }
            }
            self.buffer.push_str(&" ".repeat(length));
            self.buffer.push(' ');
            self.buffer.push_str(text);
            self.buffer.push('\n');
        }
    }
}

/// Collects the lines the highlights cover, in order, each with the
/// highlights that touch it.
fn collate_lines(highlights: &[Highlight]) -> Vec<Line> {
    let mut lines: Vec<Line> = Vec::new();
    for highlight in highlights {
        let loc = &highlight.loc;
        for number in loc.begin.line..=loc.end.line {
            if lines.last().is_none_or(|last| number > last.number) {
                lines.push(Line {
                    number,
                    text: loc
                        .file
                        .source_line(number)
                        .trim_end_matches('\r')
                        .to_owned(),
                    highlights: Vec::new(),
                });
            }
        }
    }

    for line in &mut lines {
        line.highlights = highlights
            .iter()
            .enumerate()
            .filter(|(_, h)| h.loc.begin.line <= line.number && line.number <= h.loc.end.line)
            .map(|(i, _)| i)
            .collect();
    }

    lines
}

/// A span that ends at the very start of a line ends, for drawing, at the end
/// of the line before, as `source_span` normalizes it.
fn normalize_end_of_line(mut highlight: Highlight) -> Highlight {
    let loc = &mut highlight.loc;
    if loc.end.column == 0 && loc.end.line > loc.begin.line {
        loc.end.line -= 1;
        loc.end.column = loc.file.source_line(loc.end.line).chars().count();
    }
    highlight
}

fn is_multiline(highlight: &Highlight) -> bool {
    highlight.loc.begin.line != highlight.loc.end.line
}

fn replace_first_none(columns: &mut [Option<usize>], h: usize) {
    if let Some(slot) = columns.iter_mut().find(|slot| slot.is_none()) {
        *slot = Some(h);
    }
}

fn replace_with_none(columns: &mut [Option<usize>], h: usize) {
    if let Some(slot) = columns.iter_mut().find(|slot| **slot == Some(h)) {
        *slot = None;
    }
}

/// The first `columns` characters of `text`.
fn prefix(text: &str, columns: usize) -> String {
    text.chars().take(columns).collect()
}

/// Characters `start..end` of `text`.
fn slice(text: &str, start: usize, end: usize) -> String {
    text.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn count_tabs(text: &str) -> usize {
    text.chars().filter(|&c| c == '\t').count()
}

fn is_only_whitespace(text: &str) -> bool {
    text.chars().all(|c| c == ' ' || c == '\t')
}
