use crate::dtd::dtd::{take_whitespace, take_whitespace_opt};
use crate::dtd::entity::Entity;
use crate::dtd::template_strings::{parse_string, ChainElement, TemplateString};
use nom::bytes::complete::{tag, tag_no_case, take_until, take_while};
use nom::combinator::opt;
use nom::error::ErrorKind;
use nom::multi::{separated_list0, separated_list1};
use nom::IResult;

#[derive(Debug, Clone)]
pub enum ElementName {
    Single(String),
    Group(Vec<String>),
}

impl ElementName {
    pub fn applies_to(&self, val: &str) -> bool {
        match self {
            ElementName::Single(s) => s == val,
            ElementName::Group(g) => g
                .iter()
                .map(|f| f.as_str())
                .collect::<Vec<_>>()
                .contains(&val),
        }
    }

    pub fn as_single(&self) -> Option<String> {
        match self {
            ElementName::Single(s) => Some(s.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    pub name: TemplateString,
    pub start_optional: bool,
    pub end_optional: bool,
    pub content_model: TemplateString,
}

impl Element {
    pub fn get_name(&self, entities: &[Entity]) -> ElementName {
        let expanded_name = self.name.expand(entities);

        if self
            .name
            .chain
            .contains(&ChainElement::Reference("heading".to_string()))
        {
            println!()
        }

        if let Ok((i, elements)) = parse_element_name_group(expanded_name.as_str()) {
            ElementName::Group(elements)
        } else {
            ElementName::Single(expanded_name)
        }
    }

    /// Converts groups elements into individual elements
    pub fn decompose(&self, entities: &[Entity]) -> Vec<Element> {
        match self.get_name(entities) {
            ElementName::Single(_) => vec![self.clone()],
            ElementName::Group(g) => g
                .iter()
                .map(|f| Element {
                    name: f.as_str().into(),
                    start_optional: self.start_optional,
                    end_optional: self.end_optional,
                    content_model: self.content_model.clone(),
                })
                .collect::<Vec<_>>(),
        }
    }
}

pub fn parse_name_group_string(i: &str) -> IResult<&str, TemplateString> {
    let (i, _grpo) = tag(GROUP_OPEN)(i)?;
    parse_string(i, ')', false)
}

pub fn parse_element(i: &str) -> IResult<&str, Element> {
    let (i, _start) = tag_no_case("<!element ")(i)?;

    // Try parsing names as a group first
    let mut i = i;
    let mut name;
    if let Ok((j, name_group)) = parse_name_group_string(i) {
        name = name_group;
        i = j;
    } else {
        let (j, name_str) = take_until(" ")(i)?;
        name = name_str.into();
        i = j;
    }

    let (i, _) = take_whitespace(i)?;
    let (i, start_tag) = take_until(" ")(i)?;
    let (i, _) = take_whitespace(i)?;
    let (i, end_tag) = take_until(" ")(i)?;
    let (i, _) = take_whitespace(i)?;

    let (i, content) = parse_string(i, '>', true)?;
    // let (i, _) = tag(">")(i)?;

    Ok((
        i,
        Element {
            name,
            start_optional: start_tag == "O",
            end_optional: end_tag == "O",
            content_model: content,
        },
    ))
}

#[test]
pub fn test_memo() {
    let x = parse_element("<!element memo                 - O (sender, receivers, contents)>");
    println!("{:?}", x);
    let (i, e) = x.unwrap();
    assert_eq!(i, "");
    assert_eq!(e.name, "memo".into());
    assert_eq!(e.start_optional, false);
    assert_eq!(e.end_optional, true);
    assert_eq!(e.content_model.expand(&[]), "(sender, receivers, contents)");
}

#[test]
pub fn test_group() {
    let x = parse_element("<!ELEMENT (%font;|%phrase) - - (text)*>");
    println!("{:?}", x);
    let (i, e) = x.unwrap();
    assert_eq!(i, "");
    assert_eq!(
        e.name,
        TemplateString {
            chain: vec![
                ChainElement::Reference("font".to_string()),
                ChainElement::String("|".to_string()),
                ChainElement::Reference("phrase".to_string())
            ]
        }
    );
    assert_eq!(e.start_optional, false);
    assert_eq!(e.end_optional, false);
    assert_eq!(e.content_model.expand(&[]), "(text)*");
}

#[test]
pub fn test_spaced_group() {
    let x = parse_element("<!ELEMENT ( %heading )  - -  (%text;)*>");
    println!("{:?}", x);
    let (i, e) = x.unwrap();
    assert_eq!(i, "");
    assert_eq!(
        e.name,
        TemplateString {
            chain: vec![
                ChainElement::String(" ".to_string()),
                ChainElement::Reference("heading".to_string()),
                ChainElement::String(" ".to_string())
            ]
        }
    );
    assert_eq!(e.start_optional, false);
    assert_eq!(e.end_optional, false);
    // assert_eq!(e.content_model.expand(&[]), "(text)*");
}

#[derive(Debug, Clone)]
pub enum ContentModelTokenValue {
    Empty,
    PcData,
    Reference(String),
}

#[derive(Debug, Clone)]
pub struct ContentModelToken {
    pub value: ContentModelTokenValue,
    pub required: bool,
    pub repeat: bool,
}

pub const GROUP_OPEN: &str = "(";
pub const GROUP_CLOSE: &str = ")";

pub const OPTIONAL_OCCURRENCE_INDICATOR: &str = "?";
pub const REQUIRED_AND_REPEATABLE: &str = "+";
pub const OPTIONAL_AND_REPEATABLE: &str = "*";

pub const CONNECTOR_SEQUENCE: &str = ",";
pub const CONNECTOR_OR: &str = "|";
pub const CONNECTOR_AND: &str = "&";

pub const RESERVED_NAME_INDICATOR: char = '#';

pub fn parse_content_model_token(i: &str) -> IResult<&str, ContentModelToken> {
    //TODO: treat pcdata specially
    let (i, name) = take_while(|f: char| f.is_alphanumeric() || f == RESERVED_NAME_INDICATOR)(i)?;
    //TODO: can these be combined or only one per token
    let (i, occurrence_opt) = opt(tag(OPTIONAL_OCCURRENCE_INDICATOR))(i)?;
    let (i, required_and_repeat) = opt(tag(REQUIRED_AND_REPEATABLE))(i)?;
    let (i, opt_and_repeat) = opt(tag(OPTIONAL_AND_REPEATABLE))(i)?;

    Ok((
        i,
        ContentModelToken {
            value: ContentModelTokenValue::Reference(name.to_string()),
            required: required_and_repeat.is_some()
                || occurrence_opt.is_none()
                || opt_and_repeat.is_none(),
            repeat: required_and_repeat.is_some() || opt_and_repeat.is_some(),
        },
    ))
}

fn take_empty(i: &str) -> IResult<&str, &str> {
    tag("EMPTY")(i)
}

fn take_group_close(i: &str) -> IResult<&str, &str> {
    tag(GROUP_CLOSE)(i)
}

pub fn parse_element_name_group(i: &str) -> IResult<&str, Vec<String>> {
    if !i.contains('|') {
        return Err(nom::Err::Error(nom::error::Error::new(i, ErrorKind::Tag)));
    }
    let (i, _) = take_whitespace_opt(i)?;
    let (i, x) = separated_list1(tag("|"), take_while(|c: char| c.is_alphanumeric()))(i)?;
    let (i, _) = take_whitespace_opt(i)?;
    Ok((i, x.iter().map(|f| f.to_string()).collect()))

    // let (i, _grpo) = tag(GROUP_OPEN)(i)?;
    /*let (i, _) = take_whitespace_opt(i)?;
    let mut i = i;
    let mut out = Vec::new();
    loop {
        let (j, name) = take_while(|c: char| c != '|' && c != ')' && c != ' ')(i)?;
        let (j, _) = take_whitespace_opt(j)?;

        // If we are stuck in a loop and can't go further, error out
        if i == j {
            return Err(nom::Err::Error((i, ErrorKind::Tag)));
        }

        i = j;

        out.push(name.to_string());

        if i.len() == 0 {
            break;
        }

        // if let Ok((j, _)) = take_group_close(i) {
        //     i = j;
        //     break;
        // }
    }

    Ok((i, out))*/
}

pub fn parse_content_model_group(i: &str) -> IResult<&str, Vec<ContentModelToken>> {
    let (i, _grpo) = tag(GROUP_OPEN)(i)?;
    let (i, _) = take_whitespace_opt(i)?;
    let mut i = i;
    let mut out = Vec::new();
    loop {
        let (j, cvt) = parse_content_model_token(i)?;
        //TODO;
        let (j, seq_sep) = opt(tag(CONNECTOR_SEQUENCE))(j)?;
        let (j, sep_or) = opt(tag(CONNECTOR_OR))(j)?;
        let (j, sep_and) = opt(tag(CONNECTOR_AND))(j)?;
        let (j, _) = take_whitespace_opt(j)?;

        // If we are stuck in a loop and can't go further, error out
        if i == j {
            return Err(nom::Err::Error(nom::error::Error::new(i, ErrorKind::Tag)));
        }

        i = j;

        out.push(cvt);

        if let Ok((j, _)) = take_group_close(i) {
            i = j;
            break;
        }
    }

    let (i, occurrence_opt) = opt(tag(OPTIONAL_OCCURRENCE_INDICATOR))(i)?;
    let (i, required_and_repeat) = opt(tag(REQUIRED_AND_REPEATABLE))(i)?;
    let (i, opt_and_repeat) = opt(tag(OPTIONAL_AND_REPEATABLE))(i)?;

    Ok((i, out))
}

// Expects result of content.expand(entities)
pub fn parse_content_model(i: &str) -> IResult<&str, Vec<ContentModelToken>> {
    // Special case: empty content (B.4.2.6)
    if let Ok((i, _)) = take_empty(i) {
        return Ok((
            i,
            vec![ContentModelToken {
                value: ContentModelTokenValue::Empty,
                repeat: false,
                required: true,
            }],
        ));
    }

    let (i, tokens) = parse_content_model_group(i)?;
    let (i, _) = take_whitespace_opt(i)?;
    let (i, included) = opt(tag("+"))(i)?;
    let (i, excluded) = opt(tag("-"))(i)?;
    let (i, _) = take_whitespace_opt(i)?;

    if included.is_some() || excluded.is_some() {
        let (i, extra) = parse_content_model_group(i)?;
        Ok((i, tokens))
    } else {
        Ok((i, tokens))
    }
}

#[cfg(test)]
pub mod test {
    use crate::dtd::element::{parse_content_model, parse_content_model_group};
    use crate::dtd::template_strings::TemplateString;

    #[test]
    pub fn test_parse_content_model_basic() {
        let x = parse_content_model("(front, body, rear)");
        println!("{:?}", x);
        let (i, e) = x.unwrap();
        assert_eq!(i, "");
    }

    #[test]
    pub fn test_parse_content_model_opt() {
        let x = parse_content_model("(front?, body, rear?)");
        println!("{:?}", x);
        let (i, e) = x.unwrap();
        assert_eq!(i, "");
    }

    #[test]
    pub fn test_parse_content_model_req_rep() {
        let x = parse_content_model("(p+)");
        println!("{:?}", x);
        let (i, e) = x.unwrap();
        assert_eq!(i, "");
    }

    #[test]
    pub fn test_parse_content_model_opt_rep() {
        let x = parse_content_model("(p*)");
        println!("{:?}", x);
        let (i, e) = x.unwrap();
        assert_eq!(i, "");
    }

    #[test]
    pub fn test_parse_content_model_or() {
        let x = parse_content_model("(p|xmp)+");
        println!("{:?}", x);
        let (i, e) = x.unwrap();
        assert_eq!(i, "");
    }

    #[test]
    pub fn test_multi_group() {
        let x = parse_content_model("(H1|H2|H3|H4|H5|H6|#PCDATA | A | IMG | BR)* -(A)");
        println!("{:?}", x);
        let (i, e) = x.unwrap();
        assert_eq!(i, "");
    }
}
