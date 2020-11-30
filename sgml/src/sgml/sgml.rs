use crate::dtd::dtd::{
    take_until_whitespace, take_whitespace, DECLARATION_SUBSET_CLOSE, DECLARATION_SUBSET_OPEN,
    MARKED_SECTION_CLOSE, MARKUP_DECLARATION_CLOSE, MARKUP_DECLARATION_OPEN,
    PROCESSING_INSTRUCTION_CLOSE, PROCESSING_INSTRUCTION_OPEN,
};
use crate::dtd::element::{GROUP_CLOSE, GROUP_OPEN};
use crate::dtd::marked_section::parse_marked_section;
use crate::sgml_declaration::sgml_declaration::{
    parse_number, parse_parameter_seperator, parse_public_identifier, parse_sgml_declaration,
};
use nom::bits::complete::take;
use nom::bytes::complete::{tag, tag_no_case, take_while, take_while_m_n};
use nom::combinator::opt;
use nom::error::ErrorKind::{Many0, TakeUntil};
use nom::multi::{many0, many1};
use nom::sequence::tuple;
use nom::IResult;
use std::io::ErrorKind;

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

/// COM
pub const COMMENT_START_OR_END: &str = "--";

/// RNI
pub const RESERVED_NAME_INDICATOR: &str = "#";

/// CRO
pub const CHARACTER_REFERENCE_OPEN: &str = "#";

/// REFC
pub const REFERENCE_CLOSE: &str = ";";

/// ERO
pub const ENTITY_REFERENCE_OPEN: &str = "&";

macro_rules! either {
    ($i: expr, $first: expr, $second: expr, $($next: expr),*) => {
        match $first($i) {
            Ok((i, x)) => Ok((i, "")),
            Err(_) => either!($i, $second, $($next),*)
        }
    };

    ($i: expr, $first:expr, $second:expr) => {
        match $first($i) {
            Ok((i, x)) => Ok((i, "")),
            Err(_) => $second($i)
        }
    };
}

/// ISO(6.1)[1]
fn parse_sgml_document(i: &str) -> IResult<&str, &str> {
    fn inner(i: &str) -> IResult<&str, &str> {
        either!(
            i,
            parse_sgml_subdocument_entity,
            parse_sgml_text_entity,
            parse_character_data_entity,
            parse_specific_character_data_entity,
            parse_non_sgml_data_entity
        )
    }

    let (i, _) = parse_sgml_document_entity(i)?;
    //TODO: should be many0, need to parse more of the above
    let (i, _) = opt(inner)(i)?;
    Ok((i, ""))
}

/// ISO(6.2)[2]
fn parse_sgml_document_entity(i: &str) -> IResult<&str, &str> {
    let (i, _) = many0(parse_separator)(i)?;
    let (i, _) = parse_sgml_declaration(i)?;
    let (i, _) = parse_prolog(i)?;
    let (i, _) = parse_document_instance_set(i)?;
    Ok((i, ""))
}

/// ISO(6.2)[3]
fn parse_sgml_subdocument_entity(i: &str) -> IResult<&str, &str> {
    let (i, _) = parse_prolog(i)?;
    let (i, _) = parse_document_instance_set(i)?;
    Ok((i, ""))
}

/// ISO(6.2)[4]
fn parse_sgml_text_entity(i: &str) -> IResult<&str, &str> {
    let (i, _) = many0(parse_sgml_character)(i)?;
    Ok((i, ""))
}

/// Parse a separator (s) ISO(6.2.1)[5]
fn parse_separator(i: &str) -> IResult<&str, &str> {
    take_whitespace(i)
    // TODO:
}

/// ISO(6.3)[5.1]
fn parse_character_data_entity(i: &str) -> IResult<&str, &str> {
    let (i, _) = many0(parse_sgml_character)(i)?;
    Ok((i, ""))
}

/// ISO(6.3)[5.2]
fn parse_specific_character_data_entity(i: &str) -> IResult<&str, &str> {
    let (i, _) = many0(parse_sgml_character)(i)?;
    Ok((i, ""))
}

/// ISO(6.3)[6]
fn parse_non_sgml_data_entity(i: &str) -> IResult<&str, &str> {
    let (i, _) = many0(parse_character)(i)?;
    Ok((i, ""))
}

/// ISO(11.1)[110]
fn parse_document_type_declaration(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(MARKUP_DECLARATION_OPEN)(i)?;
    let (i, _) = tag_no_case("DOCTYPE")(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_document_type_name(i)?;
    let (i, _) = opt(tuple((
        many1(parse_parameter_seperator),
        parse_external_identifier,
    )))(i)?;
    let (i, _) = opt(tuple((
        many1(parse_parameter_seperator),
        tag(DECLARATION_SUBSET_OPEN),
        parse_document_type_declaration_subset,
        tag(DECLARATION_SUBSET_CLOSE),
    )))(i)?;
    let (i, _) = many0(parse_parameter_seperator)(i)?;
    let (i, _) = tag(MARKUP_DECLARATION_CLOSE)(i)?;

    Ok((i, ""))
}

/// ISO(11.1)[111]
fn parse_document_type_name(i: &str) -> IResult<&str, &str> {
    parse_generic_identifier(i)
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
    //TODO: extra also start tag can be opt
    let (i, _) = parse_start_tag(i)?;
    let (i, _content) = parse_content(i)?;
    let (i, _) = opt(parse_end_tag)(i)?;

    Ok((i, ""))
}

/// ISO(7.1)[9]
fn parse_base_document_type_declaration(i: &str) -> IResult<&str, &str> {
    parse_document_type_declaration(i)
}

//TODO
/// ISO(7.1)[7]
fn parse_prolog(i: &str) -> IResult<&str, &str> {
    let (i, _) = many0(parse_other_prolog)(i)?;
    let (i, _) = parse_base_document_type_declaration(i)?;

    fn inner1(i: &str) -> IResult<&str, &str> {
        either!(i, parse_document_type_declaration, parse_other_prolog)
    }
    let (i, _) = many0(inner1)(i)?;

    fn inner2(i: &str) -> IResult<&str, &str> {
        either!(i, parse_link_type_declaration, parse_other_prolog)
    }
    let (i, _) = many0(inner2)(i)?;

    Ok((i, ""))
}

/// ISO(7.1)[8]
fn parse_other_prolog(i: &str) -> IResult<&str, &str> {
    let (i, _) = either!(
        i,
        parse_comment_declaration,
        parse_processing_instruction,
        parse_separator
    )?;
    Ok((i, ""))
}

///ISO(7.6)[24]
fn parse_content(i: &str) -> IResult<&str, &str> {
    let (i, _) = either!(
        i,
        parse_mixed_content,
        parse_element_content,
        parse_replaceable_character_data,
        parse_character_data
    )?;
    Ok((i, ""))
}
/// ISO(7.6)[25]
fn parse_mixed_content(i: &str) -> IResult<&str, &str> {
    either!(
        i,
        /*parse_data_character,*/ parse_element,
        parse_other_content
    )
}
/// ISO(7.6)[26]
fn parse_element_content(i: &str) -> IResult<&str, &str> {
    fn parse_content_inner(i: &str) -> IResult<&str, &str> {
        either!(i, parse_element, parse_other_content, parse_separator)
    }

    //TODO: many1 -> many0
    let (i, _) = many1(parse_content_inner)(i)?;
    Ok((i, ""))
}
/// ISO(7.6)[27]
fn parse_other_content(i: &str) -> IResult<&str, &str> {
    //TODO: shortref support
    either!(
        i,
        parse_comment_declaration,
        parse_short_reference_use_declaration,
        parse_link_set_use_declaration,
        parse_processing_instruction,
        parse_character_reference,
        parse_general_entity_reference,
        parse_marked_section_declaration
    )
}
/// ISO(9.1)[46]
fn parse_replaceable_character_data(i: &str) -> IResult<&str, &str> {
    fn inner(i: &str) -> IResult<&str, &str> {
        either!(
            i,
            parse_data_character,
            parse_character_reference,
            parse_general_entity_reference
        )
    }
    let (i, _) = many0(inner)(i)?;
    Ok((i, ""))
}

/// ISO(9.2)[47]
fn parse_character_data(i: &str) -> IResult<&str, &str> {
    //TODO: this should be a many0 however causes parsing issues rn
    let (i, _s) = many1(parse_data_character)(i)?;
    Ok((i, ""))
}

/// ISO(7.5)[19]
fn parse_end_tag(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(END_TAG_OPEN)(i)?;
    let (i, _) = parse_document_type_specification(i)?;
    let (i, _) = parse_generic_identifier_specification(i)?;
    let (i, _) = many0(parse_separator)(i)?;
    let (i, _) = tag(START_TAG_CLOSE)(i)?;
    //TODO: min end tag
    Ok((i, ""))
}

/// ISO(12.1)[154]
fn parse_link_type_declaration(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(MARKUP_DECLARATION_OPEN)(i)?;
    let (i, _) = tag_no_case("LINKTYPE")(i)?;
    let (i, _) = many0(parse_parameter_seperator)(i)?;
    let (i, _) = parse_link_type_name(i)?;
    let (i, _) = many0(parse_parameter_seperator)(i)?;
    //TODO: why is this braced
    let (i, _) = either!(
        i,
        parse_simple_link_specification,
        parse_implicit_link_specification,
        parse_explicit_link_specification
    )?;
    let (i, _) = opt(tuple((
        many0(parse_parameter_seperator),
        parse_external_identifier,
    )))(i)?;
    let (i, _) = opt(tuple((
        many0(parse_parameter_seperator),
        tag(DECLARATION_SUBSET_OPEN),
        parse_link_type_declaration_subset,
        tag(DECLARATION_SUBSET_CLOSE),
    )))(i)?;
    let (i, _) = many0(parse_parameter_seperator)(i)?;
    let (i, _) = tag(MARKUP_DECLARATION_CLOSE)(i)?;

    Ok((i, ""))
}

/// ISO(12.1)[155]
fn parse_link_type_name(i: &str) -> IResult<&str, &str> {
    parse_name(i)
}

/// ISO(12.1.1)[156]
fn parse_simple_link_specification(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(RESERVED_NAME_INDICATOR)(i)?;
    let (i, _) = tag_no_case("SIMPLE")(i)?;
    let (i, _) = tag(RESERVED_NAME_INDICATOR)(i)?;
    let (i, _) = many0(parse_parameter_seperator)(i)?;
    let (i, _) = tag_no_case("IMPLIED")(i)?;
    Ok((i, ""))
}

/// ISO(12.1.2)[157]
fn parse_implicit_link_specification(i: &str) -> IResult<&str, &str> {
    let (i, _) = parse_source_document_type_name(i)?;
    let (i, _) = many0(parse_parameter_seperator)(i)?;
    let (i, _) = tag(RESERVED_NAME_INDICATOR)(i)?;
    let (i, _) = tag_no_case("IMPLIED")(i)?;
    Ok((i, ""))
}

/// ISO(12.1.3)[158]
fn parse_explicit_link_specification(i: &str) -> IResult<&str, &str> {
    let (i, _) = parse_source_document_type_name(i)?;
    let (i, _) = many0(parse_parameter_seperator)(i)?;
    let (i, _) = parse_result_document_type_name(i)?;
    Ok((i, ""))
}

/// ISO(12.1.3)[159]
fn parse_source_document_type_name(i: &str) -> IResult<&str, &str> {
    parse_document_type_name(i)
}

/// ISO(12.1.3)[160]
fn parse_result_document_type_name(i: &str) -> IResult<&str, &str> {
    parse_document_type_name(i)
}

/// ISO(12.1.4)[161] TODO
fn parse_link_type_declaration_subset(i: &str) -> IResult<&str, &str> {
    Ok((i, ""))
}

/// ISO(10.3)[91]
fn parse_comment_declaration(i: &str) -> IResult<&str, &str> {
    fn inner(i: &str) -> IResult<&str, &str> {
        let (i, _) = opt(tuple((
            parse_comment,
            many0(tuple((parse_separator, parse_comment))),
        )))(i)?;
        Ok((i, ""))
    }

    let (i, _) = tag(MARKUP_DECLARATION_OPEN)(i)?;
    let (i, _) = either!(i, parse_comment, inner)?;
    let (i, _) = tag(MARKUP_DECLARATION_CLOSE)(i)?;
    Ok((i, ""))
}
/// ISO(10.3)[92]
fn parse_comment(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(COMMENT_START_OR_END)(i)?;
    let (i, _) = many0(parse_sgml_character)(i)?;
    let (i, _) = tag(COMMENT_START_OR_END)(i)?;
    Ok((i, ""))
}
#[test]
pub fn test_parse_comment_decl() {
    let x = parse_comment_declaration("<!-- Here\'s a good place to put a comment. -->");
    let (i, d) = x.expect("Unable to parse comment");
    assert_eq!("", i);
}

/// ISO(11.6)[151]
fn parse_map_name(i: &str) -> IResult<&str, &str> {
    parse_name(i)
}

/// ISO(11.6)[152]
fn parse_short_reference_use_declaration(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(MARKUP_DECLARATION_OPEN)(i)?;
    let (i, _) = tag_no_case("USEMAP")(i)?;
    let (i, _) = many1(parse_separator)(i)?;
    let (i, _) = parse_map_specification(i)?;
    let (i, _) = opt(tuple((
        many1(parse_separator),
        parse_associated_element_type,
    )))(i)?;
    let (i, _) = many1(parse_separator)(i)?;
    let (i, _) = tag(MARKUP_DECLARATION_CLOSE)(i)?;

    Ok((i, ""))
}

/// ISO(10.1.5)[72]
fn parse_associated_element_type(i: &str) -> IResult<&str, &str> {
    either!(i, parse_generic_identifier, parse_name_group)
}

/// ISO(11.6)[153]
fn parse_map_specification(i: &str) -> IResult<&str, &str> {
    //TODO: is this actually a *literal* empty?
    fn inner(i: &str) -> IResult<&str, &str> {
        let (i, _) = tuple((tag(RESERVED_NAME_INDICATOR), tag_no_case("EMPTY")))(i)?;
        Ok((i, ""))
    }

    let (i, _) = either!(i, parse_map_name, inner)?;
    Ok((i, ""))
}

/// ISO(12.3)[169]
fn parse_link_set_use_declaration(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(MARKUP_DECLARATION_OPEN)(i)?;
    let (i, _) = tag_no_case("USELINK")(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_link_set_specification(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_link_type_name(i)?;
    let (i, _) = many0(parse_parameter_seperator)(i)?;
    let (i, _) = tag(MARKUP_DECLARATION_CLOSE)(i)?;
    Ok((i, ""))
}

/// ISO(12.3)[170]
fn parse_link_set_specification(i: &str) -> IResult<&str, &str> {
    fn inner1(i: &str) -> IResult<&str, &str> {
        //TODO: again is this *literal* empty / restore
        let (i, _) = tuple((tag(RESERVED_NAME_INDICATOR), tag_no_case("EMPTY")))(i)?;
        Ok((i, ""))
    }
    fn inner2(i: &str) -> IResult<&str, &str> {
        //TODO: again is this *literal* empty / restore
        let (i, _) = tuple((tag(RESERVED_NAME_INDICATOR), tag_no_case("RESTORE")))(i)?;
        Ok((i, ""))
    }

    let (i, _) = either!(i, parse_link_set_name, inner1, inner2)?;

    Ok((i, ""))
}

/// ISO(12.2)[164]
fn parse_link_set_name(i: &str) -> IResult<&str, &str> {
    fn inner(i: &str) -> IResult<&str, &str> {
        //TODO: again is this *literal* initial
        let (i, _) = tuple((tag(RESERVED_NAME_INDICATOR), tag_no_case("INITIAL")))(i)?;
        Ok((i, ""))
    }
    let (i, _) = either!(i, parse_name, inner)?;

    Ok((i, ""))
}

/// ISO(8)[44]
fn parse_processing_instruction(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(PROCESSING_INSTRUCTION_OPEN)(i)?;
    let (i, _) = parse_system_data(i)?;
    let (i, _) = tag(PROCESSING_INSTRUCTION_CLOSE)(i)?;
    Ok((i, ""))
}

/// ISO(8)[45]
fn parse_system_data(i: &str) -> IResult<&str, &str> {
    parse_character_data(i)
}

/// ISO(9.4.5)[61]
fn parse_reference_end(i: &str) -> IResult<&str, &str> {
    //TODO: the rest
    let (i, _) = opt(tag(REFERENCE_CLOSE))(i)?;

    Ok((i, ""))
}

/// ISO(9.5)[62]
fn parse_character_reference(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(CHARACTER_REFERENCE_OPEN)(i)?;
    let (i, _) = either!(i, parse_function_name, parse_character_number)?;
    let (i, _) = parse_reference_end(i)?;

    Ok((i, ""))
}

/// ISO(9.5)[63]
fn parse_function_name(i: &str) -> IResult<&str, &str> {
    fn first(i: &str) -> IResult<&str, &str> {
        tag_no_case("RE")(i)
    }
    fn second(i: &str) -> IResult<&str, &str> {
        tag_no_case("RS")(i)
    }
    fn third(i: &str) -> IResult<&str, &str> {
        tag_no_case("SPACE")(i)
    }
    let (i, _) = either!(i, first, second, third, parse_name)?;

    Ok((i, ""))
}

/// ISO(9.5)[64]
fn parse_character_number(i: &str) -> IResult<&str, &str> {
    let (i, _) = parse_number(i)?;
    Ok((i, ""))
}

/// ISO(9.4.4)[59]
fn parse_general_entity_reference(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(ENTITY_REFERENCE_OPEN)(i)?;
    let (i, _) = opt(parse_name_group)(i)?;
    let (i, _) = parse_name(i)?;
    let (i, _) = parse_reference_end(i)?;

    Ok((i, ""))
}

/// ISO(10.4)[93]
fn parse_marked_section_declaration(i: &str) -> IResult<&str, &str> {
    let (i, _) = parse_marked_section_start(i)?;
    let (i, _) = parse_status_keyword_specification(i)?;
    let (i, _) = tag(DECLARATION_SUBSET_OPEN)(i)?;
    let (i, _) = parse_marked_section(i)?;
    let (i, _) = parse_marked_section_end(i)?;

    Ok((i, ""))
}
/// ISO(10.4)[94]
fn parse_marked_section_start(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(MARKUP_DECLARATION_OPEN)(i)?;
    let (i, _) = tag(DECLARATION_SUBSET_OPEN)(i)?;
    Ok((i, ""))
}
/// ISO(10.4)[95]
fn parse_marked_section_end(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(MARKED_SECTION_CLOSE)(i)?;
    let (i, _) = tag(MARKUP_DECLARATION_CLOSE)(i)?;
    Ok((i, ""))
}
/// ISO(10.4)[93]
fn parse_status_keyword_specification(i: &str) -> IResult<&str, &str> {
    fn inner(i: &str) -> IResult<&str, &str> {
        either!(i, parse_status_keyword, tag_no_case("TEMP"))
    }
    let (i, _) = many0(tuple((many1(parse_parameter_seperator), inner)))(i)?;
    let (i, _) = many0(parse_parameter_seperator)(i)?;

    Ok((i, ""))
}
/// ISO(10.4)[93]
fn parse_status_keyword(i: &str) -> IResult<&str, &str> {
    fn first(i: &str) -> IResult<&str, &str> {
        tag_no_case("CDATA")(i)
    }
    fn second(i: &str) -> IResult<&str, &str> {
        tag_no_case("IGNORE")(i)
    }
    fn third(i: &str) -> IResult<&str, &str> {
        tag_no_case("INCLUDE")(i)
    }
    fn fourth(i: &str) -> IResult<&str, &str> {
        tag_no_case("RCDATA")(i)
    }
    let (i, _) = either!(i, first, second, third, fourth)?;
    Ok((i, ""))
}

fn parse_sgml_character(i: &str) -> IResult<&str, &str> {
    take_while_m_n(1, 1, |c: char| c != '-' && c != '<' && c != '>')(i)
}

/// ISO(9.2)[48]
fn parse_data_character(i: &str) -> IResult<&str, &str> {
    parse_sgml_character(i)
}

/// ISO(9.2)[49]
fn parse_character(i: &str) -> IResult<&str, &str> {
    //TODO: needs capacity set parsing to add NONSGML support
    // let (i, _) = either!(i, parse_sgml_character, tag(NONSGML))?;
    let (i, _) = parse_sgml_character(i)?;
    Ok((i, ""))
}

/// ISO(10.1.6)[73]
fn parse_external_identifier(i: &str) -> IResult<&str, &str> {
    fn take_system(i: &str) -> IResult<&str, &str> {
        tag_no_case("SYSTEM")(i)
    }
    fn inner(i: &str) -> IResult<&str, &str> {
        let (i, _) = tuple((
            tag_no_case("PUBLIC"),
            many1(parse_parameter_seperator),
            parse_public_identifier,
        ))(i)?;
        Ok((i, ""))
    }

    let (i, _tag) = either!(i, take_system, inner)?;

    let (i, _) = opt(tuple((
        many1(parse_parameter_seperator),
        parse_system_identifier,
    )))(i)?;

    Ok((i, ""))
}

/// ISO(10.1.6)[75]
fn parse_system_identifier(i: &str) -> IResult<&str, &str> {
    fn first(i: &str) -> IResult<&str, &str> {
        let (i, _) = tuple((
            tag(LITERAL_START_OR_END),
            parse_system_data,
            tag(LITERAL_START_OR_END),
        ))(i)?;
        Ok((i, ""))
    }
    fn second(i: &str) -> IResult<&str, &str> {
        let (i, _) = tuple((
            tag(LITERAL_START_OR_END_ALTERNATIVE),
            parse_system_data,
            tag(LITERAL_START_OR_END_ALTERNATIVE),
        ))(i)?;
        Ok((i, ""))
    }
    either!(i, first, second)
}

fn parse_document_type_declaration_subset(i: &str) -> IResult<&str, &str> {
    Ok((i, ""))
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
    either!(i, parse_attribute_value, parse_attribute_value_literal)
}

/// ISO(7.9.4)[35]
pub fn parse_attribute_value(i: &str) -> IResult<&str, &str> {
    //TODO:
    parse_character_data(i)
    // either!(i, parse_character_data, parse_general_entity_name, parse_general_entity_name_list, parse_id_value, parse_id_reference_value, parse_id_reference_list, parse_name, parse_name_list, parse_name_token, parse_name_token_list, parse_notation_name, parse_number, parse_number_list, parse_number_token, parse_number_token_list);
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
    let (i, _) = opt(parse_name_group)(i)?;
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
    take_while(|t| t != ' ' && t != '>')(i)
}

#[cfg(test)]
pub mod test {
    use crate::sgml::sgml::parse_sgml_document;

    #[test]
    pub fn sgml_parse_html2_example() {
        let declaration = "<!SGML  \"ISO 8879:1986\"
--
	SGML Declaration for HyperText Markup Language (HTML).

--

CHARSET
         BASESET  \"ISO 646:1983//CHARSET
        International Reference Version
            (IRV)//ESC 2/5 4/0\"
        DESCSET  0   9   UNUSED
        9   2   9
        11  2   UNUSED
        13  1   13
        14  18  UNUSED
        32  95  32
        127 1   UNUSED
        BASESET   \"ISO Registration Number 100//CHARSET
                ECMA-94 Right Part of
                Latin Alphabet Nr. 1//ESC 2/13 4/1\"

        DESCSET  128  32   UNUSED
        160  96    32

        CAPACITY        SGMLREF
        TOTALCAP        150000
        GRPCAP          150000
        ENTCAP		150000

        SCOPE    DOCUMENT
        SYNTAX
        SHUNCHAR CONTROLS 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
        17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 127
        BASESET  \"ISO 646:1983//CHARSET
                   International Reference Version
                   (IRV)//ESC 2/5 4/0\"
        DESCSET  0 128 0
        FUNCTION
        RE          13
        RS          10
        SPACE       32
        TAB SEPCHAR  9


        NAMING   LCNMSTRT \"\"
        UCNMSTRT \"\"
        LCNMCHAR \".-\"
        UCNMCHAR \".-\"
        NAMECASE GENERAL YES
        ENTITY  NO
        DELIM    GENERAL  SGMLREF
        SHORTREF SGMLREF
        NAMES    SGMLREF
        QUANTITY SGMLREF
        ATTSPLEN 2100
        LITLEN   1024
        NAMELEN  72    -- somewhat arbitrary; taken from
        internet line length conventions --
            PILEN    1024
        TAGLVL   100
        TAGLEN   2100
        GRPGTCNT 150
        GRPCNT   64

        FEATURES
        MINIMIZE
        DATATAG  NO
        OMITTAG  YES
        RANK     NO
        SHORTTAG YES
        LINK
        SIMPLE   NO
        IMPLICIT NO
        EXPLICIT NO
        OTHER
        CONCUR   NO
        SUBDOC   NO
        FORMAL   YES
        APPINFO    \"SDA\"  -- conforming SGML Document Access application
            --
            >";

        let content = "<!DOCTYPE HTML PUBLIC \"-//IETF//DTD HTML 2.0//EN\">
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

        let combined = format!("{}{}", declaration, content);

        let x = parse_sgml_document(&combined);
        println!("{:?}", x);
        assert_eq!(x.is_ok(), true);
        let (i, x) = x.unwrap();
        assert_eq!("", i);
    }
}
