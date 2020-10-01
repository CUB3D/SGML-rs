use crate::dtd::att_list::{parse_att_list, ATTList, ATTListElement};
use crate::dtd::comment::parse_comment_block;
use crate::dtd::element::{
    parse_content_model, parse_content_model_group, parse_element, ContentModelToken,
    ContentModelTokenValue, Element, ElementName,
};
use crate::dtd::entity::{parse_entity, parse_parameter_reference, Entity, ParameterReference};
use crate::dtd::marked_section::{parse_marked_section, MarkedSection};
use nom::branch::alt;
use nom::bytes::complete::{take_while, take_while1};
use nom::error::ErrorKind;
use nom::multi::many0;
use nom::sequence::tuple;
use nom::IResult;

/// See ISO(B.8.1)
pub const MARKUP_DECLARATION_OPEN: &str = "<!";
pub const DECLARATION_SUBSET_OPEN: &str = "[";
pub const MARKED_SECTION_CLOSE: &str = "]]";
pub const MARKUP_DECLARATION_CLOSE: &str = ">";

pub const PROCESSING_INSTRUCTION_OPEN: &str = "<?";
pub const PROCESSING_INSTRUCTION_CLOSE: &str = ">";

pub struct DocumentTypeDefinitionNode {
    pub value: Element,
    pub children: Vec<DocumentTypeDefinitionNode>,
}

#[derive(Debug, Clone)]
pub struct DocumentTypeDefinitionElement<'a> {
    element: Element,
    tree: &'a DocumentTypeDefinition,
}

impl<'a> DocumentTypeDefinitionElement<'a> {
    pub fn get_content_model(&self) -> Vec<ContentModelToken> {
        let (_, x) = parse_content_model(
            self.element
                .content_model
                .expand(&self.tree.entities)
                .as_str(),
        )
        .unwrap();
        x
    }

    ///TODO: not really needed, end user won't care if this was a (a|B) element or not (should this change be propegated to the parser as well?
    pub fn decompose(&self) -> Vec<Self> {
        self.element
            .decompose(&self.tree.entities)
            .into_iter()
            .map(|e| DocumentTypeDefinitionElement {
                element: e,
                tree: self.tree,
            })
            .collect::<Vec<_>>()
    }

    pub fn get_name(&self) -> String {
        self.element
            .get_name(&self.tree.entities)
            .as_single()
            .expect("No children")
    }

    //TODO: return result (err if child is not found?)
    pub fn get_children(&self) -> Vec<Self> {
        let cm = self.get_content_model();
        println!("Children of {}", self.get_name());

        cm.iter()
            .filter(|c| matches!(c.value, ContentModelTokenValue::Reference(_)))
            .map(|c| {
                match &c.value {
                    ContentModelTokenValue::Reference(s) => s.clone(),
                    _ => unimplemented!(),
                }
                //TODO: this filter is a big hack: figure out why the parse can't parse HEAD correctly
                //TODO: different bug: add support for tokenValue::pcdata in the parser
            })
            .filter(|c| c.len() > 0 && c != "#PCDATA")
            .map(|name| {
                self.tree
                    .get_element_by_name(&name)
                    .expect(&format!("Can't find element by name '{}'", name))
            })
            .collect::<Vec<_>>()
    }
}

/// See ISO(B.3.2)
#[derive(Debug, Clone)]
pub struct DocumentTypeDefinition {
    pub entities: Vec<Entity>,
    pub elements: Vec<Element>,
}

impl From<Vec<DTDElement>> for DocumentTypeDefinition {
    fn from(elements: Vec<DTDElement>) -> Self {
        let entities = elements
            .clone()
            .into_iter()
            .filter(|e| matches!(e, DTDElement::Entity(_)))
            .map(|e| match e {
                DTDElement::Entity(e) => e,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();

        let elements = elements
            .into_iter()
            .filter(|e| matches!(e, DTDElement::Element(_)))
            .map(|e| match e {
                DTDElement::Element(e) => e,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();

        Self { entities, elements }
    }
}

impl DocumentTypeDefinition {
    pub fn get_element_by_name(&self, name: &str) -> Option<DocumentTypeDefinitionElement> {
        self.get_elements()
            .into_iter()
            .find(|e| e.get_name() == name)
    }

    pub fn get_elements(&self) -> Vec<DocumentTypeDefinitionElement> {
        self.elements
            .iter()
            .cloned()
            .flat_map(|e| {
                DocumentTypeDefinitionElement {
                    element: e,
                    tree: self,
                }
                .decompose()
            })
            .collect::<Vec<_>>()
    }

    pub fn get_children(&self, element: &Element) -> Vec<ContentModelToken> {
        let elements = &self.elements;
        let s = element.content_model.expand(&self.entities);
        let (_, cm) = parse_content_model(&s).expect("Failed to parse content model");
        cm
    }

    pub fn get_roots(&self) -> Vec<DocumentTypeDefinitionElement> {
        let elements = self.get_elements();

        let mut found = Vec::new();

        for z in &elements {
            let cm = z.get_content_model();

            for c in &cm {
                match c.value {
                    ContentModelTokenValue::Reference(ref s) => {
                        found.push(s.clone());
                    }
                    _ => {}
                }
            }
        }

        elements
            .into_iter()
            .flat_map(|e| e.decompose())
            .filter(|e| !found.contains(&e.get_name()))
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone)]
pub enum DTDElement {
    WhiteSpace(String),
    Comment(String),
    Entity(Entity),
    MarkedSection(MarkedSection),
    ParameterReference(ParameterReference),
    Element(Element),
    ATTList(ATTList),
}

pub fn is_whitespace(i: char) -> bool {
    i == ' ' || i == '\n' || i == '\t'
}

pub fn take_whitespace(i: &str) -> IResult<&str, &str> {
    take_while1(is_whitespace)(i)
}
pub fn take_whitespace_opt(i: &str) -> IResult<&str, &str> {
    take_while(is_whitespace)(i)
}

pub fn take_until_whitespace(i: &str) -> IResult<&str, &str> {
    take_while1(|c| !is_whitespace(c))(i)
}

pub fn parse_dtd_element(i: &str) -> IResult<&str, DTDElement> {
    if let Ok((i, w)) = take_whitespace(i) {
        return Ok((i, DTDElement::WhiteSpace(w.to_string())));
    }
    if let Ok((i, s)) = parse_comment_block(i) {
        return Ok((i, DTDElement::Comment(s)));
    }
    if let Ok((i, e)) = parse_entity(i) {
        return Ok((i, DTDElement::Entity(e)));
    }
    if let Ok((i, e)) = parse_element(i) {
        return Ok((i, DTDElement::Element(e)));
    }
    if let Ok((i, ms)) = parse_marked_section(i) {
        return Ok((i, DTDElement::MarkedSection(ms)));
    }
    if let Ok((i, pr)) = parse_parameter_reference(i) {
        return Ok((i, DTDElement::ParameterReference(pr)));
    }
    if let Ok((i, at)) = parse_att_list(i) {
        return Ok((i, DTDElement::ATTList(at)));
    }

    return Err(nom::Err::Error(nom::error::make_error(i, ErrorKind::Eof)));

    // alt((take_whitespace, parse_comment, parse_entity))(i)
    // many0(alt((
    //     take_whitespace,
    //     parse_comment,
    //     parse_entity,
    //     )))(i)
}

pub fn read_dtd(i: &str) -> IResult<&str, DocumentTypeDefinition> {
    let (i, elements) = many0(parse_dtd_element)(i)?;

    Ok((i, elements.into()))
}

#[cfg(test)]
pub mod test {
    use crate::dtd::dtd::read_dtd;
    use std::fs::File;
    use std::io::Read;

    #[test]
    pub fn test_read_html_lat1() {
        let mut f = File::open("./dtd/xhtml-lat1.ent").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();

        let x = read_dtd(&s);
        println!("{:?}", x);
        let (i, e) = x.unwrap();
        assert_eq!(i, "");
    }

    #[test]
    pub fn test_read_html_special() {
        let mut f = File::open("./dtd/xhtml-special.ent").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();

        let x = read_dtd(&s);
        println!("{:?}", x);
        let (i, e) = x.unwrap();
        assert_eq!(i, "");
    }

    #[test]
    pub fn test_read_html_symbol() {
        let mut f = File::open("./dtd/xhtml-symbol.ent").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();

        let x = read_dtd(&s);
        println!("{:?}", x);
        let (i, e) = x.unwrap();
        assert_eq!(i, "");
    }

    #[test]
    pub fn test_read_html_dtd() {
        let mut f = File::open("./dtd/html.dtd").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();

        let x = read_dtd(&s);
        println!("{:?}", x);
        let (i, e) = x.unwrap();
        assert_eq!(i, "");
    }
}
