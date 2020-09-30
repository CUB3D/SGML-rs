use crate::dtd::comment::parse_inline_comment;
use crate::dtd::dtd::{take_until_whitespace, take_whitespace, MARKUP_DECLARATION_OPEN};
use crate::dtd::entity::take_space;
use nom::bytes::complete::{is_a, tag, tag_no_case};
use nom::bytes::streaming::take_until;
use nom::multi::{many0, many1};
use nom::sequence::tuple;
use nom::IResult;

// macro_rules! one_of {
//     ($($expr: expr),*, $i: ident) ==> {
//         || {
//             $(
//             let x = $expr(i);
//             if x.is_some() {
//                 return x;
//             }
//             )*
//             return x;
//         }?;
//     }
// }

pub const LITERAL_START_OR_END: &str = "\"";
pub const LITERAL_START_OR_END_ALTERNATIVE: &str = "'";

//TODO: global
//TODO: this is v.wrong
/// Parse a parameter seperator (PS) ISO(10.1.1)
pub fn parse_parameter_seperator(i: &str) -> IResult<&str, String> {
    match take_space(i) {
        Ok((i, s)) => Ok((i, s.to_string())),
        Err(_) => parse_inline_comment(i),
    }

    // let x = tag(' ');
    // if x.is_som
    // //TODO: others
}

pub fn parse_minimum_literal(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(LITERAL_START_OR_END)(i)?;
    let (i, content) = take_until(LITERAL_START_OR_END)(i)?;
    let (i, _) = tag(LITERAL_START_OR_END)(i)?;

    Ok((i, content))
    //TODO: support alternate form
}

pub fn parse_capacity_set(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag_no_case("CAPACITY")(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;

    let opt_1 = |i| -> IResult<&str, &str> {
        tuple((
            tag_no_case("PUBLIC"),
            many1(parse_parameter_seperator),
            parse_public_identifier,
        ))(i)?;
        tag("")(i)
    };

    let opt_2 = |i| -> IResult<&str, &str> {
        let (i, _) = tuple((
            tag_no_case("SGMLREF"),
            many1(tuple((
                many1(parse_parameter_seperator),
                parse_name,
                many1(parse_parameter_seperator),
                parse_number,
            ))),
        ))(i)?;
        tag("")(i)
    };

    let (i, _) = match opt_1(i) {
        Ok((i, _)) => Ok((i, "")),
        Err(_) => opt_2(i),
    }?;

    Ok((i, ""))
}

///ISO(10.1.6)
pub fn parse_public_identifier(i: &str) -> IResult<&str, &str> {
    parse_minimum_literal(i)
}

///ISO(13.1.1.2)
pub fn parse_described_character_set_portion(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag_no_case("DESCSET")(i)?;
    let (i, _) = many1(tuple((
        many1(parse_parameter_seperator),
        parse_character_description,
    )))(i)?;
    Ok((i, ""))
}

pub fn parse_unused(i: &str) -> IResult<&str, &str> {
    tag_no_case("UNUSED")(i)
}

///ISO(13.1.1.2)
pub fn parse_character_description(i: &str) -> IResult<&str, &str> {
    let (i, _) = parse_described_set_character_number(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    //TODO: spec is missing a comma here?
    let (i, _) = parse_number_of_characters(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;

    let (i, _) = match parse_base_set_character_number(i) {
        Ok((i, _)) => Ok((i, "")),
        Err(_) => match parse_minimum_literal(i) {
            Ok((i, _)) => Ok((i, "")),
            Err(_) => parse_unused(i),
        },
    }?;

    //TODO: needs one_of

    //TODO:
    Ok((i, ""))
}

/// ISO(13.1.1.2)[177]
pub fn parse_described_set_character_number(i: &str) -> IResult<&str, &str> {
    parse_character_number(i)
}

/// ISO(13.1.1.2)[178]
pub fn parse_base_set_character_number(i: &str) -> IResult<&str, &str> {
    parse_character_number(i)
}

/// ISO(13.1.1.2)[179]
pub fn parse_number_of_characters(i: &str) -> IResult<&str, &str> {
    parse_number(i)
}

/// ISO(9.5)[64]
pub fn parse_character_number(i: &str) -> IResult<&str, &str> {
    parse_number(i)
}

/// ISO(9.3)[56]
pub fn parse_number(i: &str) -> IResult<&str, &str> {
    let (i, _) = many1(is_a("1234567890"))(i)?;
    Ok((i, ""))
}

//TODO: this has weird overly complex definition in the spec
/// ISO(9.3)[55]
pub fn parse_name(i: &str) -> IResult<&str, &str> {
    take_until_whitespace(i)
    // let (i, _) = many1(is_a("1234567890"))(i)?;
    // Ok((i, ""))
}

/// ISO(13.1.1.1)
pub fn parse_base_character_set(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag_no_case("BASESET")(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    parse_public_identifier(i)
}

//TODO: should the meny0(seperator) actually be many1

/// ISO(13.1.1)
pub fn parse_character_set_description(i: &str) -> IResult<&str, &str> {
    let (i, bcs) = parse_base_character_set(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, pdcsp) = parse_described_character_set_portion(i)?;

    let (i, _) = many0(tuple((
        many1(parse_parameter_seperator),
        parse_base_character_set,
        many1(parse_parameter_seperator),
        parse_described_character_set_portion,
    )))(i)?;

    Ok((i, ""))
}

/// ISO(13.1)
pub fn parse_document_character_set(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag_no_case("CHARSET")(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    parse_character_set_description(i)
}

/// ISO(13.3)[181]
pub fn parse_concrete_syntax_scope(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag_no_case("SCOPE")(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;

    let take_document = |i| -> IResult<&str, &str> { tag_no_case("DOCUMENT")(i) };
    let take_instance = |i| -> IResult<&str, &str> { tag_no_case("INSTANCE")(i) };

    match take_document(i) {
        Ok((i, _)) => Ok((i, "")),
        Err(_) => take_instance(i),
    }
}

/// ISO(13.4)[182]
pub fn parse_concrete_syntax(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag_no_case("SYNTAX")(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;

    //TODO: support left hand side

    let (i, _) = parse_shunned_character_number_identification(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_syntax_reference_character_set(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_function_character_identification(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_naming_rules(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_delmiter_set(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_reserved_name_use(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_quantity_set(i)?;

    Ok((i, ""))
}

/// ISO(13.4.4)[186]
pub fn parse_function_character_identification(i: &str) -> IResult<&str, &str> {
    tuple((
        tag_no_case("FUNCTION"),
        many1(parse_parameter_seperator),
        tag_no_case("RE"),
        many1(parse_parameter_seperator),
        parse_character_number,
        many1(parse_parameter_seperator),
        tag_no_case("RS"),
        many1(parse_parameter_seperator),
        parse_character_number,
        many1(parse_parameter_seperator),
        tag_no_case("SPACE"),
        many1(parse_parameter_seperator),
        parse_character_number,
        many0(tuple((
            many1(parse_parameter_seperator),
            parse_added_function,
            many1(parse_parameter_seperator),
            parse_function_class,
            many1(parse_parameter_seperator),
            parse_character_number,
        ))),
    ))
}

/// ISO(13.4.4)[187]
pub fn parse_added_function(i: &str) -> IResult<&str, &str> {
    parse_name(i)
}

/// ISO(13.4.4)[188]
pub fn parse_function_class(i: &str) -> IResult<&str, &str> {
    let take_funchar = |i| -> IResult<&str, &str> { tag_no_case("FUNCHAR")(i) };
}

/// ISO(13.4.3)[194]
pub fn parse_syntax_reference_character_set(i: &str) -> IResult<&str, &str> {
    parse_character_set_description(i)
}

/// ISO(13.4.2)[194]
pub fn parse_shunned_character_number_identification(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag_no_case("SHUNCHAR")(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;

    let take_none = |i| -> IResult<&str, &str> { tag_no_case("NONE")(i) };

    let take_right_side = |i| -> IResult<&str, &str> {
        let controls_or_character_number = |i| -> IResult<&str, &str> {
            let take_controls = |i| -> IResult<&str, &str> { tag_no_case("CONTROLS")(i) };
            match take_controls(i) {
                Ok((i, _)) => Ok((i, "")),
                Err(_) => parse_character_number(i),
            }
        };

        let (i, _) = tuple((
            controls_or_character_number,
            many0(tuple((
                many1(parse_parameter_seperator),
                parse_character_number,
            ))),
        ))(i)?;
        Ok((i, ""))
    };

    match take_none(i) {
        Ok((i, _)) => Ok((i, "")),
        Err(_) => take_right_side(i),
    }
}

/// ISO(13.4.8)[194]
pub fn parse_quantity_set(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag_no_case("QUANTITY")(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = tag_no_case("SGMLREF")(i)?;
    let (i, _) = many0(tuple((
        many1(parse_parameter_seperator),
        parse_name,
        many1(parse_parameter_seperator),
        parse_number,
    )))(i)?;

    Ok((i, ""))
}

/// ISO(13.4.7)[193]
pub fn parse_reserved_name_use(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag_no_case("NAMES")(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = tag_no_case("SGMLREF")(i)?;
    let (i, _) = many0(tuple((
        many1(parse_parameter_seperator),
        parse_name,
        many1(parse_parameter_seperator),
        parse_name,
    )))(i)?;

    Ok((i, ""))
}

/// ISO(13.4.6)[190]
pub fn parse_delimiter_set(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag_no_case("DELIM")(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_general_delimiters(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_short_reference_delimiters(i)?;

    Ok((i, ""))
}

/// ISO(13)[171]
pub fn parse_sgml_declaration(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(MARKUP_DECLARATION_OPEN)(i)?;
    let (i, _) = tag_no_case("SGML")(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    //TOdo; name
    let (i, title) = parse_minimum_literal(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_document_character_set(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_capacity_set(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_concrete_syntax_scope(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;
    let (i, _) = parse_concrete_syntax(i)?;
    let (i, _) = many1(parse_parameter_seperator)(i)?;

    Ok((i, ""))

    //concrete syntax, ps +, feature use,
    // ps +, application-specific information, ps*,
    // mdc
}

#[cfg(test)]
pub mod test {
    use crate::sgml_declaration::sgml_declaration::parse_sgml_declaration;

    #[test]
    pub fn test_html4_declaration() {
        let i = "<!SGML  \"ISO 8879:1986 (WWW)\"
    --
         SGML Declaration for HyperText Markup Language version HTML 4

         With support for the first 17 planes of ISO 10646 and
         increased limits for tag and literal lengths etc.
    --

    CHARSET
          BASESET  \"ISO Registration Number 177//CHARSET
        ISO/IEC 10646-1:1993 UCS-4 with
        implementation level 3//ESC 2/5 2/15 4/6\"
        DESCSET 0       9       UNUSED
        9       2       9
        11      2       UNUSED
        13      1       13
        14      18      UNUSED
        32      95      32
        127     1       UNUSED
        128     32      UNUSED
        160     55136   160
        55296   2048    UNUSED  -- SURROGATES --
            57344   1056768 57344

        CAPACITY        SGMLREF
        TOTALCAP        150000
        GRPCAP          150000
        ENTCAP          150000

        SCOPE    DOCUMENT
        SYNTAX
        SHUNCHAR CONTROLS 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
        17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 127
        BASESET  \"ISO 646IRV:1991//CHARSET
                   International Reference Version
                   (IRV)//ESC 2/8 4/2\"
        DESCSET  0 128 0

        FUNCTION
        RE            13
        RS            10
        SPACE         32
        TAB SEPCHAR    9

        NAMING   LCNMSTRT \"\"
        UCNMSTRT \"\"
        LCNMCHAR \".-_:\"
        UCNMCHAR \".-_:\"
        NAMECASE GENERAL YES
        ENTITY  NO
        DELIM    GENERAL  SGMLREF
        HCRO \"&#38;#x\" -- 38 is the number for ampersand --
            SHORTREF SGMLREF
        NAMES    SGMLREF
        QUANTITY SGMLREF
        ATTCNT   60      -- increased --
            ATTSPLEN 65536   -- These are the largest values --
            LITLEN   65536   -- permitted in the declaration --
            NAMELEN  65536   -- Avoid fixed limits in actual --
            PILEN    65536   -- implementations of HTML UA's --
            TAGLVL   100
        TAGLEN   65536
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
        APPINFO NONE
            >";

        let x = parse_sgml_declaration(i);
        println!("{:?}", x);
        let (i, e) = x.unwrap();
        assert_eq!(i, "")
    }
}