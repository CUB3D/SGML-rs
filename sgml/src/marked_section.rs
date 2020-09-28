use crate::dtd::{
    take_whitespace, take_whitespace_opt, DECLARATION_SUBSET_OPEN, MARKED_SECTION_CLOSE,
    MARKUP_DECLARATION_OPEN, MDC,
};
use nom::bytes::complete::{tag, take_until};
use nom::IResult;

#[derive(Debug, Clone)]
pub struct MarkedSection {
    pub status: String,
    pub content: String,
}

pub fn parse_marked_section(i: &str) -> IResult<&str, MarkedSection> {
    let (i, _) = tag(MARKUP_DECLARATION_OPEN)(i)?;
    let (i, _) = tag(DECLARATION_SUBSET_OPEN)(i)?;
    let (i, _) = take_whitespace_opt(i)?;
    let (i, status) = take_until(" ")(i)?;
    let (i, _) = take_whitespace(i)?;
    let (i, _) = tag(DECLARATION_SUBSET_OPEN)(i)?;
    let (i, content) = take_until(MARKED_SECTION_CLOSE)(i)?;
    let (i, _) = tag(MARKED_SECTION_CLOSE)(i)?;
    let (i, _) = tag(MDC)(i)?;

    Ok((
        i,
        MarkedSection {
            status: status.to_string(),
            content: content.to_string(),
        },
    ))
}

#[cfg(test)]
pub mod test {
    use crate::marked_section::parse_marked_section;

    #[test]
    pub fn test_marked_section() {
        let x = parse_marked_section(
            "<![ %HTML.Recommended [
    <!ENTITY % HTML.Deprecated \"IGNORE
        \">
]]>",
        );

        let (i, e) = x.unwrap();

        assert_eq!(i, "");
    }
}
