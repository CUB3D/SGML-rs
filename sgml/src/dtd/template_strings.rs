use crate::dtd::entity::{parse_parameter_reference, Entity};
use nom::bytes::complete::take;
use nom::error::ErrorKind;
use nom::IResult;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ChainElement {
    String(String),
    Reference(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TemplateString {
    pub chain: Vec<ChainElement>,
}

fn take_char(i: &str) -> IResult<&str, char> {
    let (i, x) = take(1usize)(i)?;
    Ok((i, x.chars().nth(0usize).unwrap()))
}

//TODO: could this be impled as a state machine
pub fn parse_string(i: &str, terminal: char, quoting: bool) -> IResult<&str, TemplateString> {
    if quoting {
        assert_ne!(terminal, '\"', "terminal must differ from quotation");
    }

    let mut i = i;
    let mut out = String::new();
    let mut quoted = false;

    let mut chain = Vec::new();

    loop {
        // Try to take a parameter reference first
        if let Ok((j, param)) = parse_parameter_reference(i) {
            if out.len() > 0 {
                chain.push(ChainElement::String(out));
                out = String::new();
            }
            //TODO: keep obj
            chain.push(ChainElement::Reference(param.name));
            i = j;
            continue;
        }

        if let Ok((j, c)) = take_char(i) {
            i = j;

            if c == '"' && quoting {
                quoted = !quoted;
            }

            if c == terminal && !quoted {
                break;
            }

            out.push(c);
        } else {
            break;
        }
    }

    if out.len() > 0 {
        chain.push(ChainElement::String(out));
    }

    Ok((i, TemplateString { chain }))
}

impl TemplateString {
    /// Expand a template string to a real string, this will recursively resolve all entity references until a concrete value is found
    /// NOTE: this should be avoided at all costs and delayed as late as possible when needed, this will trigger entity expansion exploits: e.g. "Billion laughs"
    pub fn expand(&self, entities: &[Entity]) -> String {
        self.chain
            .iter()
            .cloned()
            .map(|c| match c {
                ChainElement::String(s) => s,
                ChainElement::Reference(r) => {
                    //TODO: this is the recursion
                    entities
                        .iter()
                        .find(|e| e.name == r)
                        .expect(&format!("Unable to find {}", r))
                        .content
                        .expand(entities)
                    // &"".to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

impl From<&str> for TemplateString {
    fn from(s: &str) -> Self {
        Self {
            chain: vec![ChainElement::String(s.to_string())],
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::dtd::entity::Entity;
    use crate::dtd::template_strings::{parse_string, ChainElement, TemplateString};

    #[test]
    pub fn test_basic_string() {
        let x = parse_string("this is a test>", '>', true);
        let (i, x) = x.unwrap();
        assert_eq!(i, "");
        assert_eq!(
            x,
            TemplateString {
                chain: vec![ChainElement::String("this is a test".to_string())]
            }
        )
    }

    #[test]
    pub fn test_quoting() {
        let x = parse_string("%SDAPREF; \"<Anchor: #AttList>\">", '>', true);
        let (i, x) = x.unwrap();
        assert_eq!(i, "");
        assert_eq!(
            x,
            TemplateString {
                chain: vec![
                    ChainElement::Reference("SDAPREF".to_string()),
                    ChainElement::String(" \"<Anchor: #AttList>\"".to_string())
                ]
            }
        )
    }

    #[test]
    pub fn test_entity_resolving() {
        let x = parse_string("%SDAPREF; \"<Anchor: #AttList>\">", '>', true);
        let (i, x) = x.unwrap();
        assert_eq!(i, "");
        assert_eq!(
            x,
            TemplateString {
                chain: vec![
                    ChainElement::Reference("SDAPREF".to_string()),
                    ChainElement::String(" \"<Anchor: #AttList>\"".to_string())
                ]
            }
        );

        let e = Entity {
            name: "SDAPREF".to_string(),
            content: TemplateString {
                chain: vec![ChainElement::String("SDAPREF  CDATA  #FIXED".to_string())],
            },
            parameter: true,
            external: false,
            public: false,
        };

        let expanded_string = x.expand(&vec![e]);
        assert_eq!(
            expanded_string,
            "SDAPREF  CDATA  #FIXED \"<Anchor: #AttList>\""
        )
    }

    #[test]
    pub fn test_multiline() {
        let x = parse_string(
            "REL %linkType #IMPLIED
        REV %linkType #IMPLIED
        URN CDATA #IMPLIED
        TITLE CDATA #IMPLIED
        METHODS NAMES #IMPLIED
        \">",
            '"',
            false,
        );
        let (i, x) = x.unwrap();
        assert_eq!(i, ">");
        assert_eq!(x, TemplateString { chain: vec![
            ChainElement::String("REL ".to_string()),
            ChainElement::Reference("linkType".to_string()),
            ChainElement::String(" #IMPLIED\n        REV ".to_string()),
            ChainElement::Reference("linkType".to_string()),
            ChainElement::String(" #IMPLIED\n        URN CDATA #IMPLIED\n        TITLE CDATA #IMPLIED\n        METHODS NAMES #IMPLIED\n        ".to_string())]})
    }

    #[test]
    pub fn test_only_ref() {
        let x = parse_string("%SDAPREF;>", '>', true);
        let (i, x) = x.unwrap();
        assert_eq!(i, "");
        assert_eq!(
            x,
            TemplateString {
                chain: vec![ChainElement::Reference("SDAPREF".to_string())]
            }
        );
    }

    #[test]
    pub fn test_only_ref2() {
        let x = parse_string("%SDAPREF>", '>', true);
        let (i, x) = x.unwrap();
        assert_eq!(i, "");
        assert_eq!(
            x,
            TemplateString {
                chain: vec![ChainElement::Reference("SDAPREF".to_string())]
            }
        );
    }

    #[test]
    pub fn test_ref_in_quotes() {
        let x = parse_string(
            "P | %list | DL
                                 | %preformatted
                                 | %block.forms\">",
            '>',
            true,
        );
        let (i, x) = x.unwrap();
        assert_eq!(i, "");
        assert_eq!(
            x,
            TemplateString {
                chain: vec![
                    ChainElement::String("P | ".to_string()),
                    ChainElement::Reference("list".to_string()),
                    ChainElement::String(" | DL\n                                 | ".to_string()),
                    ChainElement::Reference("preformatted".to_string()),
                    ChainElement::String("\n                                 | ".to_string()),
                    ChainElement::Reference("block.forms".to_string()),
                    ChainElement::String("\">".to_string())
                ]
            }
        );
    }
}
