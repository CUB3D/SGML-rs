use crate::dtd::comment::parse_inline_comment;
use crate::dtd::dtd::{take_until_whitespace, take_whitespace, take_whitespace_opt};
use crate::dtd::template_strings::{parse_string, TemplateString};
use nom::bytes::complete::{tag, tag_no_case, take_until, take_while1, take_while_m_n};
use nom::combinator::opt;
use nom::error::ErrorKind;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Clone, Debug)]
pub struct Entity {
    pub name: String,
    pub external: bool,
    pub parameter: bool,
    pub public: bool,
    pub content: TemplateString,
}

pub fn parse_entity(i: &str) -> IResult<&str, Entity> {
    let (i, _) = tag_no_case("<!ENTITY ")(i)?;
    let (i, param_marker) = opt(tuple((tag("%"), take_whitespace)))(i)?;
    let (i, name) = take_until_whitespace(i)?;
    let (i, _) = take_whitespace(i)?;
    let (i, external_marker) = opt(tuple((tag("SYSTEM"), take_whitespace)))(i)?;
    //TODO: spec seems to state that system and public are mutually exclusive ISO(B6.2.3)
    // if this is a public entitiy, then the content is called the "public identifier" (TODO: seems that there can be two "fields"  one for public identifier and one for content
    //TODO: public identifiers can only contain "minimum data characters"
    let (i, public_marker) = opt(tuple((tag("PUBLIC"), take_whitespace)))(i)?;

    //TODO: force parsing of the content to ignore anything that might be a delimiter, (used to prevent / when SHORTTAG is enabled from terminating in the following <!ENTITY sol "/"> -> <!ENTITY sol CDATA "/">)
    let (i, _cdata) = opt(tuple((tag("CDATA"), take_whitespace)))(i)?;

    let (i, _) = tag("\"")(i)?;
    //TODO: escaping and %asdf; substitution (note billion laughs)
    // let (i, content) = take_until("\"")(i)?;

    let (i, content) = parse_string(i, '"', false)?;

    // let (i, _content_term) = tag("\"")(i)?;

    let (i, _) = take_whitespace_opt(i)?;
    let (i, _inline_comment) = opt(parse_inline_comment)(i)?;
    let (i, _) = take_whitespace_opt(i)?;

    let (i, _term) = tag(">")(i)?;

    Ok((
        i,
        Entity {
            name: name.to_string(),
            external: external_marker.is_some(),
            parameter: param_marker.is_some(),
            public: public_marker.is_some(),
            content: content,
        },
    ))
}

#[test]
fn test_internal_general() {
    let x = parse_entity("<!ENTITY greeting1 \"Hello world\">");
    println!("{:?}", x);
    let (i, e) = x.unwrap();
    assert_eq!(i, "");
    assert_eq!(e.name, "greeting1");
    assert_eq!(e.content, "Hello world".into());
    assert_eq!(e.external, false);
    assert_eq!(e.parameter, false);
}

#[test]
fn test_external_general() {
    let x = parse_entity("<!ENTITY greeting2 SYSTEM \"file:///hello.txt\">");
    println!("{:?}", x);
    let (i, e) = x.unwrap();
    assert_eq!(i, "");
    assert_eq!(e.name, "greeting2");
    assert_eq!(e.content, "file:///hello.txt".into());
    assert_eq!(e.external, true);
    assert_eq!(e.parameter, false);
}

#[test]
fn test_internal_parameter() {
    let x = parse_entity("<!ENTITY % greeting3 \"¡Hola!\">");
    println!("{:?}", x);
    let (i, e) = x.unwrap();
    assert_eq!(i, "");
    assert_eq!(e.name, "greeting3");
    assert_eq!(e.content, "¡Hola!".into());
    assert_eq!(e.external, false);
    assert_eq!(e.parameter, true);
}

const ENTITY_REFERENCE_OPEN: &str = "&";
const PARAMETER_ENTITY_REFERENCE_OPEN: &str = "%";
const REFERENCE_CLOSE: &str = ";";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParameterReference {
    pub name: String,
}

fn take_reference_close(i: &str) -> IResult<&str, &str> {
    tag(REFERENCE_CLOSE)(i)
}

pub fn take_space(i: &str) -> IResult<&str, &str> {
    take_while_m_n(1, 1, |c: char| c.is_whitespace())(i)
    // tag(" ")(i)
}

fn take_record_end(i: &str) -> IResult<&str, &str> {
    let (i, a) = opt(tag(")"))(i)?;
    let (i, b) = opt(tag(">"))(i)?;
    let (i, c) = opt(tag("|"))(i)?;
    let (i, d) = opt(tag("\""))(i)?;

    return if a.is_some() || b.is_some() || c.is_some() || d.is_some() {
        Ok((i, ""))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(i, ErrorKind::Tag)))
    };
}

pub fn parse_parameter_reference(i: &str) -> IResult<&str, ParameterReference> {
    let (i, _) = tag(PARAMETER_ENTITY_REFERENCE_OPEN)(i)?;
    let (i, name) = take_while1(|c: char| c.is_alphanumeric() || c == '.')(i)?;

    let refc = take_reference_close(i);

    match refc {
        Ok((i, _)) => Ok((
            i,
            ParameterReference {
                name: name.to_string(),
            },
        )),
        // Special case(B.6.1) entity references don't need REFC if followed by either a space or a record end
        Err(e) => {
            // Check for space (note: dont consume just peek)
            if let Ok((_, _)) = take_space(i) {
                return Ok((
                    i,
                    ParameterReference {
                        name: name.to_string(),
                    },
                ));
            }

            // Check for record end
            if let Ok((_, _)) = take_record_end(i) {
                return Ok((
                    i,
                    ParameterReference {
                        name: name.to_string(),
                    },
                ));
            }

            // Check for end of data
            if i.len() == 0 {
                return Ok((
                    i,
                    ParameterReference {
                        name: name.to_string(),
                    },
                ));
            }

            return Err(e);
        }
    }
}

#[test]
pub fn entitiy_reference_only() {
    let x = parse_parameter_reference("%flow");
    println!("{:?}", x);
    let (i, e) = x.unwrap();
    assert_eq!(i, "");
    assert_eq!(
        e,
        ParameterReference {
            name: "flow".to_string()
        }
    )
}

#[test]
pub fn entitiy_reference_newline() {
    let x = parse_parameter_reference("%flow\n");
    println!("{:?}", x);
    let (i, e) = x.unwrap();
    assert_eq!(i, "\n");
    assert_eq!(
        e,
        ParameterReference {
            name: "flow".to_string()
        }
    )
}
