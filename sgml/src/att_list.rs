use crate::dtd::{take_until_whitespace, take_whitespace, MARKUP_DECLARATION_OPEN};
use crate::template_strings::{parse_string, TemplateString};
use nom::bytes::complete::{tag, tag_no_case, take_until};
use nom::multi::many1;
use nom::IResult;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ATTListElement {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct ATTList {
    pub name: String,
    //TODO: can't parse without custom string with expand on demand references or will be vulnerable to billion laughs
    // pub elements: Vec<ATTListElement>
    pub value: TemplateString,
}

pub fn parse_att_list(i: &str) -> IResult<&str, ATTList> {
    let (i, _) = tag(MARKUP_DECLARATION_OPEN)(i)?;
    let (i, _) = tag_no_case("ATTLIST")(i)?;

    let (i, _) = take_whitespace(i)?;
    let (i, name) = take_until_whitespace(i)?;
    let (i, _) = take_whitespace(i)?;
    //Todo; loop

    // let (i, elements) = many1(parse_att_list_element)(i)?;

    let (i, value) = parse_string(i, '>', true)?;

    // let (i, _) = tag(">")(i)?;

    Ok((
        i,
        ATTList {
            name: name.to_string(),
            value: value,
            // elements
        },
    ))
}

pub fn parse_att_list_element(i: &str) -> IResult<&str, ATTListElement> {
    let (i, att_name) = take_until_whitespace(i)?;
    let (i, _) = take_whitespace(i)?;
    let (i, att_value) = take_until_whitespace(i)?;
    let (i, _) = take_whitespace(i)?;

    Ok((
        i,
        ATTListElement {
            name: att_name.to_string(),
            value: att_value.to_string(),
        },
    ))
}

#[cfg(test)]
pub mod test {
    use crate::att_list::{parse_att_list, ATTListElement};

    #[test]
    pub fn test_att_list() {
        let x = parse_att_list(
            "<!ATTLIST BR
        %SDAPREF; \"&#RE;\"
        >",
        );
        println!("{:?}", x);
        let (i, e) = x.unwrap();
        assert_eq!(i, "");
        assert_eq!(e.name, "BR");
        // assert_eq!(e.elements.first(), Some(&ATTListElement {
        //     name: "%SDAPREF;".to_string(),
        //     value: "\"&#RE;\"".to_string(),
        // }));
    }
}
