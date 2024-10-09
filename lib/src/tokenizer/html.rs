use crate::tokenizer::{Standard, TextTokenizer, Tokens};

use html2text;

#[derive(Debug, Default)]
pub struct Html {
    tokenizer: Standard,
}

impl Html {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TextTokenizer for Html {
    fn tokenize<T: AsRef<str>>(&mut self, text: T) -> Tokens {
        let clean_text = html2text::from_read(text.as_ref().as_bytes(), 100);
        self.tokenizer.tokenize(clean_text)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        tokenizer::{html::Html, TextTokenizer},
        tokens,
    };

    #[test]
    fn test_tokenizer_html_basic() {
        let mut html_tokenizer = Html::new();
        let text = "<p>Hello, world!</p>";
        let tokens = html_tokenizer.tokenize(text);
        assert_eq!(tokens, tokens!["Hello", "world"]);
    }

    // #[test]
    // fn test_tokenizer_html_with_tags() {
    //     let mut html_tokenizer = Html::new();
    //     let text = "<div><p>This is <b>bold</b> and <i>italic</i>.</p></div>";
    //     let tokens = html_tokenizer.tokenize(text);
    //     assert_eq!(tokens, tokens!["This", "is", "bold", "and", "italic"]);
    // }

    // #[test]
    // fn test_tokenizer_html_complex() {
    //     let mut html_tokenizer = Html::new();
    //     let text = r#"
    //         <html>
    //             <head>
    //                 <title>Test</title>
    //             </head>
    //             <body>
    //                 <h1>Header</h1>
    //                 <p>Paragraph with <a href="#">link</a> and <img src="image.jpg" alt="image">.</p>
    //                 <ul>
    //                     <li>List item 1</li>
    //                     <li>List item 2</li>
    //                 </ul>
    //             </body>
    //         </html>
    //     "#;
    //     let tokens = html_tokenizer.tokenize(text);
    //     assert_eq!(tokens, vec!["Header", "Paragraph", "with", "link", "and", "image", "List", "item", "1", "List", "item", "2"]);
    // }

    #[test]
    fn test_tokenizer_html_empty() {
        let mut html_tokenizer = Html::new();
        let text = "";
        let tokens = html_tokenizer.tokenize(text);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenizer_html_invalid() {
        let mut html_tokenizer = Html::new();
        let text = "<p>Unclosed tag";
        let tokens = html_tokenizer.tokenize(text);
        assert_eq!(tokens, tokens!["Unclosed", "tag"]);
    }

    #[test]
    fn test_tokenizer_html_special_characters() {
        let mut html_tokenizer = Html::new();
        let text = "<p>Special characters &amp; entities &#x1F600;.</p>";
        let tokens = html_tokenizer.tokenize(text);
        assert_eq!(
            tokens,
            tokens!["Special", "characters", "&", "entities", "😀"]
        );
    }

    #[test]
    fn test_tokenizer_html_script_and_style() {
        let mut html_tokenizer = Html::new();
        let text = r#"
            <html>
                <head>
                    <style>
                        body {font-size: 14px;}
                    </style>
                    <script>
                        console.log("Hello, world!");
                    </script>
                </head>
                <body>
                    <p>Content</p>
                </body>
            </html>
        "#;
        let tokens = html_tokenizer.tokenize(text);
        assert_eq!(tokens, tokens!["Content"]);
    }
}
