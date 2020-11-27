/// STAGO
pub const START_TAG_OPEN: &str = "<";
/// TAGC
pub const START_TAG_CLOSE: &str = ">";
// ETAGO
pub const END_TAG_OPEN: &str = "/>";

/// ISO(7.4)[14]
pub fn parse_start_tag(i: &str) -> IResult<&str, &str> {
    Ok((i, ""))
}

#[cfg(test)]
pub mod test {

}

