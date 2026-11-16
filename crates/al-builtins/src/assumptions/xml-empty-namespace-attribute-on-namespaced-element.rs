XmlEmptyNamespaceAttributeOnNamespacedElement => {
    id: "xml-empty-namespace-attribute-on-namespaced-element",
    assumed: "XmlAttributeCollection.Set(Name, '', Value) and XmlElement.SetAttribute(Name, '', Value) with a plain name on an element in a namespace add or replace an attribute with no namespace and no prefix, as .NET's SetAttribute(localName, \"\", value) does: a default namespace never applies to an attribute. Pinned (no assumption) on an element without a namespace (xml/WdXmlEmptyNsAndUnassigned.al).",
    tests: "Continia Base's CTS-SYS XML Attribute List.SetNamedItem on UBL elements (Delivery Network tests).",
    probe: "XmlElement.Create('e', 'urn:e'), then Attributes().Set('a', '', '1') and Set('a', '', '2'); WriteTo and Count after each; the same on a prefixed element Create('p:e', 'urn:e').",
}
