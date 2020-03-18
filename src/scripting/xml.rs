use crate::error::{user_error, Result};

use sxd_document::parser;
use sxd_xpath::{Context, Factory};

pub struct XmlNamespace {
    prefix: String,
    uri: String,
}

impl XmlNamespace {
    pub fn new<P: Into<String>, U: Into<String>>(prefix: P, uri: U) -> XmlNamespace {
        XmlNamespace {
            prefix: prefix.into(),
            uri: uri.into(),
        }
    }
}

pub fn query_xpath_as_string(
    namespaces: &Vec<XmlNamespace>,
    query: &str,
    xml: &str,
) -> Result<String> {
    let mut context = Context::new();
    for namespace in namespaces {
        context.set_namespace(&namespace.prefix, &namespace.uri)
    }

    let package = parser::parse(xml)?;
    let document = package.as_document();
    let root = document.root().children()[0];

    Ok(Factory::new()
        .build(query)?
        .ok_or_else(|| user_error("No XPath query was available"))?
        .evaluate(&context, root)?
        .string())
}
