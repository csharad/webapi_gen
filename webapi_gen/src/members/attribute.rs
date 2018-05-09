use weedle::{types::Type, interface::AttributeInterfaceMember};
use types::Types;
use traits::{IsDefined, WriteBindings, ToSafeName};
use std::io::Write;
use result::GResult;
use heck::SnakeCase;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Attribute {
    pub is_global: bool,
    pub interface: String,
    pub readonly: bool,
    pub identifier: String,
    pub type_: Type
}

impl Attribute {
    pub fn scrape(from: &AttributeInterfaceMember, is_global: bool, interface: &str, types: &Types) -> Option<Attribute> {
        if from.type_.type_.is_defined(types) {
            let readonly = from.readonly.is_some();
            let identifier = from.identifier.name.clone();
            let type_ = from.type_.type_.clone();

            Some(Attribute {
                is_global,
                interface: interface.to_string(),
                readonly,
                identifier,
                type_
            })
        } else {
            None
        }
    }
}

impl WriteBindings for Attribute {
    fn write_bindings<T: Write>(&self, buf: &mut T) -> GResult<()> {
        // Currently bindgen does not support rust keywords on macro
        if self.identifier.is_unsafe_name() {
            return Ok(());
        }

        let snake_name = self.identifier.to_snake_case();
        let safe_name = snake_name.to_safe_name();
        if self.is_global {
            if self.identifier != safe_name {
                writeln!(buf, "#[wasm_bindgen(js_name = {})]", self.identifier)?;
            }

            write!(buf, "pub static {}: ", safe_name)?;
            self.type_.write_bindings(buf)?;
            writeln!(buf, ";\n")?;
        } else {
            if self.identifier == safe_name {
                writeln!(buf, "#[wasm_bindgen(method, getter, structural)]")?;
            } else {
                writeln!(buf, "#[wasm_bindgen(method, getter = {}, structural)]", self.identifier)?;
            }

            write!(buf, "pub fn {name}(this: &{interface}) -> ", name = safe_name, interface = self.interface)?;
            self.type_.write_bindings(buf)?;
            writeln!(buf, ";\n")?;

            if !self.readonly {
                if self.identifier == snake_name {
                    writeln!(buf, "#[wasm_bindgen(method, setter, structural)]")?;
                } else {
                    writeln!(buf, "#[wasm_bindgen(method, setter = {}, structural)]", self.identifier)?;
                }

                write!(buf, "pub fn set_{name}(this: &{interface}, val: ", name = snake_name, interface = self.interface)?;
                self.type_.write_bindings(buf)?;
                writeln!(buf, ");\n")?;
            }
        }
        Ok(())
    }
}
