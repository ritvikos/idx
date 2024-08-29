use crate::tokenizer::TextTokenizer;

const DELIMITERS: [char; 7] = [' ', ',', ';', '!', '.', '\t', '\r'];

pub struct Standard {}

impl TextTokenizer for Standard {
    fn tokenize(&mut self, text: &str) -> Vec<String> {
        todo!()
    }
}
