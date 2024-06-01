use crate::{lexer::Lexer, token::Token, Location};

// this is almost equivalent to std::iter::Peekable<Lexer<'a>>,
// but sadly we can't use a type alias because we need a few
// custom methods...

type LItem<'a> = <Lexer<'a> as Iterator>::Item;

#[derive(Clone)]
pub struct TokenStream<'a> {
    lexer: Lexer<'a>,
    // remember a peeked value, even if it was none
    peeked: Option<Option<LItem<'a>>>,

    last_location: Option<Location>,
}

impl<'a> TokenStream<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            peeked: None,

            last_location: None,
        }
    }

    pub fn peek(&mut self) -> Option<&LItem<'a>> {
        self.peeked
            .get_or_insert_with(|| self.lexer.next())
            .as_ref()
    }

    pub fn location_since(&self, start: &Token<'a>) -> Location {
        let from = start.span.location().start;
        let to = self
            .last_location
            .clone()
            // this should be unreachable if the invoker has a Token<'a>
            .expect("token stream has not yet found any token")
            .end;

        from..to
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = LItem<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.peeked.take() {
            Some(v) => v,
            None => self.lexer.next(),
        };

        if let Some(Ok(ref token)) = item {
            self.last_location = Some(token.span.location());
        }

        item
    }
}
