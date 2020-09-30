use nom::bytes::complete::{tag, take_until};
use nom::IResult;

type Comment = String;

pub fn parse_inline_comment(i: &str) -> IResult<&str, Comment> {
    let (i, _start) = tag("--")(i)?;
    let (i, content) = take_until("--")(i)?;
    let (i, _end) = tag("--")(i)?;

    Ok((i, content.to_string()))
}

pub fn parse_comment_block(i: &str) -> IResult<&str, Comment> {
    let (i, _start) = tag("<!--")(i)?;
    let (i, content) = take_until("-->")(i)?;
    let (i, _end) = tag("-->")(i)?;

    Ok((i, content.to_string()))
}

#[test]
fn test_comment() {
    let x = parse_comment_block(
        "<!--    html.dtd

        Document Type Definition for the HyperText Markup Language
		 (HTML DTD)

	$Id: html.dtd,v 1.30 1995/09/21 23:30:19 connolly Exp $

	Author: Daniel W. Connolly <connolly@w3.org>
	See Also: html.decl, html-1.dtd
	  http://www.w3.org/hypertext/WWW/MarkUp/MarkUp.html
-->",
    );
    println!("{:?}", x);
    let (i, e) = x.unwrap();
    assert_eq!(i, "");
    assert_eq!(
        e,
        "    html.dtd

        Document Type Definition for the HyperText Markup Language
		 (HTML DTD)

	$Id: html.dtd,v 1.30 1995/09/21 23:30:19 connolly Exp $

	Author: Daniel W. Connolly <connolly@w3.org>
	See Also: html.decl, html-1.dtd
	  http://www.w3.org/hypertext/WWW/MarkUp/MarkUp.html
"
    )
}
