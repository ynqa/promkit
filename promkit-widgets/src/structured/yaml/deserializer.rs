use std::{collections::HashSet, fmt, io::Read};

use serde::{
    Deserialize,
    de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor},
};

use super::yamlz::{
    ContainerNode, ContainerType, Row, YamlNode, normalize_mapping_key_for_display,
};

struct ParsedRows(Vec<Row>);

impl<'de> Deserialize<'de> for ParsedRows {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut rows = Vec::new();
        RowsSeed {
            rows: &mut rows,
            depth: 0,
            key: None,
            is_sequence_item: false,
        }
        .deserialize(deserializer)?;
        Ok(Self(rows))
    }
}

struct RowsSeed<'a> {
    rows: &'a mut Vec<Row>,
    depth: usize,
    key: Option<String>,
    is_sequence_item: bool,
}

impl<'de> DeserializeSeed<'de> for RowsSeed<'_> {
    type Value = usize;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(RowsVisitor {
            rows: self.rows,
            depth: self.depth,
            key: self.key,
            is_sequence_item: self.is_sequence_item,
        })
    }
}

struct RowsVisitor<'a> {
    rows: &'a mut Vec<Row>,
    depth: usize,
    key: Option<String>,
    is_sequence_item: bool,
}

impl RowsVisitor<'_> {
    fn push(self, node: YamlNode) -> usize {
        self.rows.push(Row {
            depth: self.depth,
            key: self.key,
            node,
            is_sequence_item: self.is_sequence_item,
        });
        self.rows.len() - 1
    }
}

impl<'de> Visitor<'de> for RowsVisitor<'_> {
    type Value = usize;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a YAML value")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(self.push(YamlNode::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(self.push(YamlNode::Null))
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(self.push(YamlNode::Boolean(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(self.push(YamlNode::Number(value.into())))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(self.push(YamlNode::Number(value.into())))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
        Ok(self.push(YamlNode::Number(value.into())))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(value.to_owned())
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(value.to_owned())
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(self.push(YamlNode::String(value)))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let Self {
            rows,
            depth,
            key,
            is_sequence_item,
        } = self;
        let open_index = rows.len();
        rows.push(Row {
            depth,
            key,
            is_sequence_item,
            node: YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 0,
            }),
        });

        let mut is_empty = true;
        while sequence
            .next_element_seed(RowsSeed {
                rows: &mut *rows,
                depth: depth + 1,
                key: None,
                is_sequence_item: true,
            })?
            .is_some()
        {
            is_empty = false;
        }

        if is_empty {
            rows[open_index].node = YamlNode::Container(ContainerNode::Empty {
                typ: ContainerType::Array,
            });
            return Ok(open_index);
        }

        let close_index = rows.len();
        rows.push(Row {
            depth,
            key: None,
            is_sequence_item: false,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index,
            }),
        });
        rows[open_index].node = YamlNode::Container(ContainerNode::Open {
            typ: ContainerType::Array,
            collapsed: false,
            close_index,
        });
        Ok(open_index)
    }

    fn visit_map<A>(self, mut mapping: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let Self {
            rows,
            depth,
            key,
            is_sequence_item,
        } = self;
        let open_index = rows.len();
        rows.push(Row {
            depth,
            key,
            is_sequence_item,
            node: YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 0,
            }),
        });

        let mut keys = HashSet::new();
        while let Some(mapping_key) = mapping.next_key::<serde_yaml::Value>()? {
            let key = normalize_mapping_key_for_display(&mapping_key);
            if !keys.insert(mapping_key) {
                return Err(de::Error::custom("duplicate entry in YAML map"));
            }
            mapping.next_value_seed(RowsSeed {
                rows: &mut *rows,
                depth: depth + 1,
                key,
                is_sequence_item: false,
            })?;
        }

        if keys.is_empty() {
            rows[open_index].node = YamlNode::Container(ContainerNode::Empty {
                typ: ContainerType::Object,
            });
            return Ok(open_index);
        }

        let close_index = rows.len();
        rows.push(Row {
            depth,
            key: None,
            is_sequence_item: false,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index,
            }),
        });
        rows[open_index].node = YamlNode::Container(ContainerNode::Open {
            typ: ContainerType::Object,
            collapsed: false,
            close_index,
        });
        Ok(open_index)
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        let (tag, contents) = data.variant::<String>()?;
        let Self {
            rows,
            depth,
            key,
            is_sequence_item,
        } = self;
        let index = contents.newtype_variant_seed(RowsSeed {
            rows: &mut *rows,
            depth,
            key,
            is_sequence_item,
        })?;
        rows[index].node = YamlNode::Tagged {
            tag: serde_yaml::value::Tag::new(tag).to_string(),
            node: Box::new(rows[index].node.clone()),
        };
        Ok(index)
    }
}

pub fn from_str(input: &str) -> Result<Vec<Row>, serde_yaml::Error> {
    collect(serde_yaml::Deserializer::from_str(input).map(ParsedRows::deserialize))
}

pub fn from_reader<R: Read>(reader: R) -> Result<Vec<Row>, serde_yaml::Error> {
    collect(serde_yaml::Deserializer::from_reader(reader).map(ParsedRows::deserialize))
}

fn collect<I>(documents: I) -> Result<Vec<Row>, serde_yaml::Error>
where
    I: IntoIterator<Item = Result<ParsedRows, serde_yaml::Error>>,
{
    let mut documents = documents.into_iter();
    let Some(first) = documents.next() else {
        return Ok(Vec::new());
    };
    let mut rows = first?.0;

    for document in documents {
        rows.push(Row {
            depth: 0,
            key: None,
            node: YamlNode::DocumentSeparator,
            is_sequence_item: false,
        });
        let mut document_rows = document?.0;
        let offset = rows.len();
        for row in &mut document_rows {
            rebase_container_indices(&mut row.node, offset);
        }
        rows.extend(document_rows);
    }
    Ok(rows)
}

fn rebase_container_indices(node: &mut YamlNode, offset: usize) {
    match node {
        YamlNode::Tagged { node, .. } => rebase_container_indices(node, offset),
        YamlNode::Container(ContainerNode::Open { close_index, .. }) => {
            *close_index += offset;
        }
        YamlNode::Container(ContainerNode::Close { open_index, .. }) => {
            *open_index += offset;
        }
        _ => {}
    }
}
