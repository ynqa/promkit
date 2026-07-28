use std::{fmt, io::Read};

use serde::{
    Deserialize,
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor},
};

use super::jsonz::{ContainerNode, ContainerType, JsonNode, Row};

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
        }
        .deserialize(deserializer)?;
        Ok(Self(rows))
    }
}

struct RowsSeed<'a> {
    rows: &'a mut Vec<Row>,
    depth: usize,
    key: Option<String>,
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
        })
    }
}

struct RowsVisitor<'a> {
    rows: &'a mut Vec<Row>,
    depth: usize,
    key: Option<String>,
}

impl RowsVisitor<'_> {
    fn push(self, node: JsonNode) -> usize {
        self.rows.push(Row {
            depth: self.depth,
            key: self.key,
            node,
        });
        self.rows.len() - 1
    }
}

impl<'de> Visitor<'de> for RowsVisitor<'_> {
    type Value = usize;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(self.push(JsonNode::Null))
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(self.push(JsonNode::Boolean(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(self.push(JsonNode::Number(value.into())))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(self.push(JsonNode::Number(value.into())))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let number = serde_json::Number::from_f64(value)
            .ok_or_else(|| E::custom("non-finite float is not valid JSON"))?;
        Ok(self.push(JsonNode::Number(number)))
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
        Ok(self.push(JsonNode::String(value)))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let Self { rows, depth, key } = self;
        let open_index = rows.len();
        rows.push(Row {
            depth,
            key,
            node: JsonNode::Container(ContainerNode::Open {
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
            })?
            .is_some()
        {
            is_empty = false;
        }

        if is_empty {
            rows[open_index].node = JsonNode::Container(ContainerNode::Empty {
                typ: ContainerType::Array,
            });
            return Ok(open_index);
        }

        let close_index = rows.len();
        rows.push(Row {
            depth,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index,
            }),
        });
        rows[open_index].node = JsonNode::Container(ContainerNode::Open {
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
        let Self { rows, depth, key } = self;
        let open_index = rows.len();
        rows.push(Row {
            depth,
            key,
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 0,
            }),
        });

        let mut is_empty = true;
        while let Some(key) = mapping.next_key::<String>()? {
            mapping.next_value_seed(RowsSeed {
                rows: &mut *rows,
                depth: depth + 1,
                key: Some(key),
            })?;
            is_empty = false;
        }

        if is_empty {
            rows[open_index].node = JsonNode::Container(ContainerNode::Empty {
                typ: ContainerType::Object,
            });
            return Ok(open_index);
        }

        let close_index = rows.len();
        rows.push(Row {
            depth,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index,
            }),
        });
        rows[open_index].node = JsonNode::Container(ContainerNode::Open {
            typ: ContainerType::Object,
            collapsed: false,
            close_index,
        });
        Ok(open_index)
    }
}

pub fn from_str(input: &str) -> Result<Vec<Row>, serde_json::Error> {
    collect(serde_json::Deserializer::from_str(input).into_iter::<ParsedRows>())
}

pub fn from_reader<R: Read>(reader: R) -> Result<Vec<Row>, serde_json::Error> {
    collect(serde_json::Deserializer::from_reader(reader).into_iter::<ParsedRows>())
}

fn collect<I>(documents: I) -> Result<Vec<Row>, serde_json::Error>
where
    I: IntoIterator<Item = Result<ParsedRows, serde_json::Error>>,
{
    let mut documents = documents.into_iter();
    let Some(first) = documents.next() else {
        return Ok(Vec::new());
    };
    let mut rows = first?.0;

    for document in documents {
        let mut document_rows = document?.0;
        let offset = rows.len();
        for row in &mut document_rows {
            rebase_container_indices(&mut row.node, offset);
        }
        rows.extend(document_rows);
    }
    Ok(rows)
}

fn rebase_container_indices(node: &mut JsonNode, offset: usize) {
    match node {
        JsonNode::Container(ContainerNode::Open { close_index, .. }) => {
            *close_index += offset;
        }
        JsonNode::Container(ContainerNode::Close { open_index, .. }) => {
            *open_index += offset;
        }
        _ => {}
    }
}
