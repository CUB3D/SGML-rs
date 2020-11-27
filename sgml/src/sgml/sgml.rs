use crate::dtd::dtd::take_until_whitespace;
use crate::dtd::element::{GROUP_CLOSE, GROUP_OPEN};
use crate::sgml_declaration::sgml_declaration::{
    parse_parameter_seperator, parse_sgml_declaration,
};
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::anychar;
use nom::combinator::opt;
use nom::multi::{many0, many1};
use nom::sequence::tuple;
use nom::IResult;

//TODO: move  all reference delimiters to common place
//TODO: we likely have to support different literals...
/// STAGO
pub const START_TAG_OPEN: &str = "<";
/// TAGC
pub const START_TAG_CLOSE: &str = ">";
/// ETAGO
pub const END_TAG_OPEN: &str = "/>";

/// LIT
pub const LITERAL_START_OR_END: &str = "\"";

pub const LITERAL_START_OR_END_ALTERNATIVE: &str = "'";

/// VI
pub const VALUE_INDICATOR: &str = "=";

/// ISO(6.1)[1]
fn parse_sgml_document(i: &str) -> IResult<&str, &str> {
    let (i, _) = parse_sgml_document_entity(i)?;
    //TODO: others
    let (i, _) = many0(parse_sgml_subdocument_entity)(i)?;
    Ok((i, ""))
}

/// ISO(6.2)[2]
fn parse_sgml_document_entity(i: &str) -> IResult<&str, &str> {
    let (i, _) = many0(parse_separator)(i)?;
    let (i, _) = parse_sgml_declaration(i)?;
    let (i, _) = parse_prolog(i)?;
    let (i, _) = parse_document_instance_set(i)?;
    //TODO: EE
    Ok((i, ""))
}
/// ISO(6.2)[3]
fn parse_sgml_subdocument_entity(i: &str) -> IResult<&str, &str> {
    let (i, _) = parse_prolog(i)?;
    let (i, _) = parse_document_instance_set(i)?;
    //TODO: EE
    Ok((i, ""))
}

/// ISO(7.2)[10]
fn parse_document_instance_set(i: &str) -> IResult<&str, &str> {
    let (i, _) = parse_base_document_element(i)?;
    let (i, _) = many0(parse_other_prolog)(i)?;
    Ok((i, ""))
}

/// ISO(7.2)[11]
fn parse_base_document_element(i: &str) -> IResult<&str, &str> {
    parse_document_element(i)
}

/// ISO(7.2)[12]
fn parse_document_element(i: &str) -> IResult<&str, &str> {
    parse_element(i)
}

/// ISO(7.3)[13]
fn parse_element(i: &str) -> IResult<&str, &str> {
    //TODO: extra
    let (i, _) = opt(parse_start_tag)(i)?;
    let (i, _content) = parse_content(i)?;
    let (i, _) = opt(parse_end_tag)(i)?;

    Ok((i, ""))
}

//TODO
fn parse_prolog(i: &str) -> IResult<&str, &str> {
    Ok((i, ""))
}
fn parse_other_prolog(i: &str) -> IResult<&str, &str> {
    Ok((i, ""))
}
fn parse_content(i: &str) -> IResult<&str, &str> {
    Ok((i, ""))
}
fn parse_end_tag(i: &str) -> IResult<&str, &str> {
    Ok((i, ""))
}

/// Parse a separator (s) ISO(6.2.1)[5]
fn parse_separator(i: &str) -> IResult<&str, &str> {
    //TODO:
    take_until_whitespace(i)
}

/// ISO(7.4)[14]
pub fn parse_start_tag(i: &str) -> IResult<&str, &str> {
    //TODO: minimised
    //TODO: tag length

    let (i, _) = tag(START_TAG_OPEN)(i)?;
    let (i, dts) = parse_document_type_specification(i)?;
    let (i, gis) = parse_generic_identifier_specification(i)?;
    let (i, asl) = parse_attribute_specification_list(i)?;
    let (i, _) = many0(parse_separator)(i)?;
    let (i, _) = tag(START_TAG_CLOSE)(i)?;

    Ok((i, ""))
}

/// ISO(7.8)[29]
pub fn parse_generic_identifier_specification(i: &str) -> IResult<&str, &str> {
    //TODO: support for rank
    parse_generic_identifier(i)

    // match  {
    //     Ok(x) => Ok(x),
    //     Err(_) => parse_rank_stem(i)?
    // }
}

/// ISO(7.8)[30]
pub fn parse_generic_identifier(i: &str) -> IResult<&str, &str> {
    parse_name(i)
}

/// ISO(7.9)[31]
pub fn parse_attribute_specification_list(i: &str) -> IResult<&str, &str> {
    let (i, _) = many0(parse_attribute_specification)(i)?;
    Ok((i, ""))
}

/// ISO(7.9)[32]
pub fn parse_attribute_specification(i: &str) -> IResult<&str, &str> {
    let (i, _) = many0(parse_separator)(i)?;
    let (i, _) = opt(tuple((
        parse_name,
        many0(parse_separator),
        tag(VALUE_INDICATOR),
        many0(parse_separator),
    )))(i)?;
    parse_attribute_value_specification(i)
}

/// ISO(7.9.3)[33]
pub fn parse_attribute_value_specification(i: &str) -> IResult<&str, &str> {
    //TODO: either
    // parse_attribute_value();
    parse_attribute_value_literal(i)
}

/// ISO(7.9.3)[34]
pub fn parse_attribute_value_literal(i: &str) -> IResult<&str, &str> {
    //TODO: other form
    let (i, _) = tuple((
        tag(LITERAL_START_OR_END),
        take_until_whitespace,
        tag(LITERAL_START_OR_END),
    ))(i)?;
    Ok((i, ""))
}

/// ISO(7.7)[28]
pub fn parse_document_type_specification(i: &str) -> IResult<&str, &str> {
    let (i, _) = many1(parse_name_group)(i)?;
    Ok((i, ""))
}

/// ISO(10.1.3)[69]
pub fn parse_name_group(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(GROUP_OPEN)(i)?;
    let (i, _) = many0(parse_ts)(i)?;
    let (i, _) = parse_name(i)?;
    let (i, _) = many0(tuple((
        many0(parse_ts),
        parse_connector,
        many0(parse_ts),
        parse_name,
    )))(i)?;
    let (i, _) = many0(parse_ts)(i)?;
    let (i, _) = tag(GROUP_CLOSE)(i)?;

    Ok((i, ""))
}

/// ISO(11.2.4.1)[131]
pub fn parse_connector(i: &str) -> IResult<&str, &str> {
    //TODO:
    Ok((i, ""))
}

/// ISO(10.1.3)[70]
pub fn parse_ts(i: &str) -> IResult<&str, &str> {
    parse_separator(i)
}

//TODO: this depends on the concrete syntax
///(4.198)
//TODO: also 9.3 (also 9.1 for other areas)
pub fn parse_name(i: &str) -> IResult<&str, &str> {
    take_until(" ")(i)
}

#[cfg(test)]
pub mod test {
    use crate::sgml::sgml::parse_sgml_document;

    #[test]
    pub fn sgml_parse_html2_example() {
        let i = "<!DOCTYPE HTML PUBLIC \"-//IETF//DTD HTML 2.0//EN\">
            <HTML>
            <!-- Here's a good place to put a comment. -->
        <HEAD>
            <TITLE>Structural Example</TITLE>
            </HEAD><BODY>
            <H1>First Header</H1>
            <P>This is a paragraph in the example HTML file. Keep in mind
        that the title does not appear in the document text, but that
        the header (defined by H1) does.</P>
            <OL>
            <LI>First item in an ordered list.
            <LI>Second item in an ordered list.
            <UL COMPACT>
            <LI> Note that lists can be nested;
        <LI> Whitespace may be used to assist in reading the
        HTML source.
            </UL>
            <LI>Third item in an ordered list.
            </OL>
            <P>This is an additional paragraph. Technically, end tags are
        not required for paragraphs, although they are allowed. You can
        include character highlighting in a paragraph. <EM>This sentence
        of the paragraph is emphasized.</EM> Note that the &lt;/P&gt;
        end tag has been omitted.
            <P>
            <IMG SRC =\"triangle.xbm\" alt=\"Warning: \">
            Be sure to read these <b>bold instructions</b>.
            </BODY></HTML>";

        let x = parse_sgml_document(i);
        assert_eq!(x.is_ok(), true);
    }
}
