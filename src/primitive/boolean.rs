use crate::error_format::data::ErrorInfo;
use crate::interpreter::data::MemoryType;
use crate::interpreter::message::Message;
use crate::parser::ast::Interval;
use crate::parser::literal::Literal;
use crate::primitive::int::PrimitiveInt;
use crate::primitive::object::PrimitiveObject;
use crate::primitive::string::PrimitiveString;
use crate::primitive::tools::check_usage;
use crate::primitive::Right;
use crate::primitive::{Primitive, PrimitiveType};
use lazy_static::*;
use std::cmp::Ordering;
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

type PrimitiveMethod = fn(
    boolean: &mut PrimitiveBoolean,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo>;

lazy_static! {
    static ref FUNCTIONS: HashMap<&'static str, (PrimitiveMethod, Right)> = {
        let mut map = HashMap::new();

        // type_of() -> Primitive<String>
        map.insert("type_of", (type_of as PrimitiveMethod, Right::Read));

        // to_string() -> Primitive<String>
        map.insert("to_string", (to_string as PrimitiveMethod, Right::Read));

        map
    };
}

#[derive(PartialEq, Debug, Clone)]
pub struct PrimitiveBoolean {
    pub value: bool,
}

////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn type_of(
    _boolean: &mut PrimitiveBoolean,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "type_of()", interval)?;

    Ok(PrimitiveString::get_literal("string", "boolean", interval))
}

fn to_string(
    boolean: &mut PrimitiveBoolean,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "to_string()", interval)?;

    Ok(PrimitiveString::get_literal(
        "string",
        &boolean.to_string(),
        interval,
    ))
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveBoolean {
    pub fn new(value: bool) -> Self {
        Self { value }
    }

    pub fn get_literal(content_type: &str, boolean: bool, interval: Interval) -> Literal {
        let primitive = Box::new(PrimitiveBoolean::new(boolean));

        Literal {
            content_type: content_type.to_owned(),
            primitive,
            interval,
        }
    }
}

impl Primitive for PrimitiveBoolean {
    fn do_exec(
        &mut self,
        name: &str,
        args: &[Literal],
        interval: Interval,
        _mem_type: &MemoryType,
    ) -> Result<(Literal, Right), ErrorInfo> {
        if let Some((f, right)) = FUNCTIONS.get(name) {
            let res = f(self, args, interval)?;

            return Ok((res, *right));
        }

        Err(ErrorInfo {
            message: format!("unknown method '{}' for type Boolean", name),
            interval,
        })
    }

    fn is_eq(&self, other: &dyn Primitive) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.value == other.value
        } else {
            false
        }
    }

    fn is_cmp(&self, other: &dyn Primitive) -> Option<Ordering> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            return self.value.partial_cmp(&other.value);
        }

        None
    }

    fn do_add(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let result = self.value as i64 + other.value as i64;

            return Ok(Box::new(PrimitiveInt::new(result)));
        }

        Err(ErrorInfo {
            message: "[!] Add: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn do_sub(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let result = self.value as i64 - other.value as i64;

            return Ok(Box::new(PrimitiveInt::new(result)));
        }

        Err(ErrorInfo {
            message: "[!] Sub: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn do_div(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let result = self.value as i64 / other.value as i64;

            return Ok(Box::new(PrimitiveInt::new(result)));
        }

        Err(ErrorInfo {
            message: "[!] Div: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn do_mul(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let result = self.value as i64 * other.value as i64;

            return Ok(Box::new(PrimitiveInt::new(result)));
        }

        Err(ErrorInfo {
            message: "[!] Mul: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn do_rem(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let result = self.value as i64 % other.value as i64;

            return Ok(Box::new(PrimitiveInt::new(result)));
        }

        Err(ErrorInfo {
            message: "[!] Rem: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn do_bitand(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let result = self.value as i64 & other.value as i64;

            return Ok(Box::new(PrimitiveInt::new(result)));
        }

        Err(ErrorInfo {
            message: "[!] BitAnd: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn do_bitor(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let result = self.value as i64 | other.value as i64;

            return Ok(Box::new(PrimitiveInt::new(result)));
        }

        Err(ErrorInfo {
            message: "[!] BitOr: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_type(&self) -> PrimitiveType {
        PrimitiveType::PrimitiveBoolean
    }

    fn as_box_clone(&self) -> Box<dyn Primitive> {
        Box::new((*self).clone())
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!(self.value)
    }

    fn to_string(&self) -> String {
        self.value.to_string()
    }

    fn as_bool(&self) -> bool {
        self.value
    }

    fn get_value(&self) -> &dyn std::any::Any {
        &self.value
    }

    fn get_mut_value(&mut self) -> &mut dyn std::any::Any {
        &mut self.value
    }

    fn to_msg(&self, _content_type: String) -> Message {
        let mut hashmap: HashMap<String, Literal> = HashMap::new();

        hashmap.insert(
            "text".to_owned(),
            Literal {
                content_type: "boolean".to_owned(),
                primitive: Box::new(PrimitiveString::new(&self.to_string())),
                interval: Interval { column: 0, line: 0 },
            },
        );

        let result =
            PrimitiveObject::get_literal("text", &hashmap, Interval { column: 0, line: 0 });

        Message {
            content_type: result.content_type,
            content: result.primitive.to_json(),
        }
    }
}