use std::collections::HashMap;
use std::fs;
use anyhow::{Result, Context};
use xml::reader::{EventReader, XmlEvent};
use crate::types::FlashSegment;

pub fn parse_xml(xml_path: &std::path::PathBuf) -> Result<Vec<FlashSegment>> {
    let xml_content = fs::read_to_string(xml_path)
        .context("Failed to read XML file")?;
    
    let xml_content = regex::Regex::new(r#" xmlns="[^"]+""#)
        .unwrap()
        .replace(&xml_content, "");
    
    let parser = EventReader::from_str(&xml_content);
    let mut segments = Vec::new();
    let mut current_element = String::new();
    let mut in_flash_segment = false;
    let mut current_segment = FlashSegment {
        source_start_addr: 0,
        source_end_addr: 0,
        target_start_addr: 0,
        target_end_addr: 0,
        is_compressed: false,
    };
    let mut element_attrs = HashMap::new();
    
    for event in parser {
        match event? {
            XmlEvent::StartElement { name, attributes, .. } => {
                current_element = name.local_name.clone();
                element_attrs.clear();
                for attr in attributes {
                    element_attrs.insert(attr.name.local_name.clone(), attr.value);
                }
                
                if current_element == "FLASH-SEGMENT" {
                    in_flash_segment = true;
                    current_segment.is_compressed = element_attrs.get("COMPRESSION-STATUS")
                        .map(|s| s == "COMPRESSED")
                        .unwrap_or(false);
                }
            }
            XmlEvent::Characters(text) => {
                if in_flash_segment {
                    match current_element.as_str() {
                        "SOURCE-START-ADDRESS" => {
                            current_segment.source_start_addr = u32::from_str_radix(&text, 16)
                                .context("Invalid source start address")?;
                        }
                        "SOURCE-END-ADDRESS" => {
                            current_segment.source_end_addr = u32::from_str_radix(&text, 16)
                                .context("Invalid source end address")?;
                        }
                        "TARGET-START-ADDRESS" => {
                            current_segment.target_start_addr = u32::from_str_radix(&text, 16)
                                .context("Invalid target start address")?;
                        }
                        "TARGET-END-ADDRESS" => {
                            current_segment.target_end_addr = u32::from_str_radix(&text, 16)
                                .context("Invalid target end address")?;
                        }
                        _ => {}
                    }
                }
            }
            XmlEvent::EndElement { name } => {
                if name.local_name == "FLASH-SEGMENT" && in_flash_segment {
                    segments.push(current_segment);
                    current_segment = FlashSegment {
                        source_start_addr: 0,
                        source_end_addr: 0,
                        target_start_addr: 0,
                        target_end_addr: 0,
                        is_compressed: false,
                    };
                    in_flash_segment = false;
                }
            }
            _ => {}
        }
    }
    
    Ok(segments)
} 