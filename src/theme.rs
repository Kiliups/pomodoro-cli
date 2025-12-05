#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub enum Base16 {
    Base00,
    Base01,
    Base02,
    Base03,
    Base04,
    Base05,
    Base06,
    Base07,
    Base08,
    Base09,
    Base0A,
    Base0B,
    Base0C,
    Base0D,
    Base0E,
    Base0F,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct Theme {
    pub scheme: String,
    pub author: String,
    pub base00: String, // Default Background
    pub base01: String, // Lighter Background (Used for status bars, line number and folding marks)
    pub base02: String, // Selection Background
    pub base03: String, // Comments, Invisibles, Line Highlighting
    pub base04: String, // Dark Foreground (Used for status bars)
    pub base05: String, // Default Foreground, Caret, Delimiters, Operators
    pub base06: String, // Light Foreground (Not often used)
    pub base07: String, // Light Background (Not often used)
    pub base08: String, // Variables, XML Tags, Markup Link Text, Markup Lists, Diff Deleted
    pub base09: String, // Integers, Boolean, Constants, XML Attributes, Markup Link Url
    pub base0A: String, // Classes, Markup Bold, Search Text Background
    pub base0B: String, // Strings, Inherited Class, Markup Code, Diff Inserted
    pub base0C: String, // Support, Regular Expressions, Escape Characters, Markup Quotes
    pub base0D: String, // Blue
    pub base0E: String, // Mauve
    pub base0F: String, // Rosewater
}

impl Theme {
    pub fn get_color(&self, code: Base16) -> &str {
        match code {
            Base16::Base00 => &self.base00,
            Base16::Base01 => &self.base01,
            Base16::Base02 => &self.base02,
            Base16::Base03 => &self.base03,
            Base16::Base04 => &self.base04,
            Base16::Base05 => &self.base05,
            Base16::Base06 => &self.base06,
            Base16::Base07 => &self.base07,
            Base16::Base08 => &self.base08,
            Base16::Base09 => &self.base09,
            Base16::Base0A => &self.base0A,
            Base16::Base0B => &self.base0B,
            Base16::Base0C => &self.base0C,
            Base16::Base0D => &self.base0D,
            Base16::Base0E => &self.base0E,
            Base16::Base0F => &self.base0F,
        }
    }
}

/*
system: "base16"
name: "Catppuccin Macchiato"
author: "https://github.com/catppuccin/catppuccin"
variant: "dark"
palette:
  base00: "#24273a" # base
  base01: "#1e2030" # mantle
  base02: "#363a4f" # surface0
  base03: "#494d64" # surface1
  base04: "#5b6078" # surface2
  base05: "#cad3f5" # text
  base06: "#f4dbd6" # rosewater
  base07: "#b7bdf8" # lavender
  base08: "#ed8796" # red
  base09: "#f5a97f" # peach
  base0A: "#eed49f" # yellow
  base0B: "#a6da95" # green
  base0C: "#8bd5ca" # teal
  base0D: "#8aadf4" # blue
  base0E: "#c6a0f6" # mauve
  base0F: "#f0c6c6" # flamingo
*/

impl Default for Theme {
    fn default() -> Self {
        Self {
            scheme: "Catppuccin Macchiato".to_string(),
            author: "https://github.com/catppuccin/catppuccin".to_string(),
            base00: "#24273a".to_string(), // base
            base01: "#1e2030".to_string(), // mantle
            base02: "#363a4f".to_string(), // surface0
            base03: "#494d64".to_string(), // surface1
            base04: "#5b6078".to_string(), // surface2
            base05: "#cad3f5".to_string(), // text
            base06: "#f4dbd6".to_string(), // rosewater
            base07: "#b7bdf8".to_string(), // lavender
            base08: "#ed8796".to_string(), // red
            base09: "#f5a97f".to_string(), // peach
            base0A: "#eed49f".to_string(), // yellow
            base0B: "#a6da95".to_string(), // green
            base0C: "#8bd5ca".to_string(), // teal
            base0D: "#8aadf4".to_string(), // blue
            base0E: "#c6a0f6".to_string(), // mauve
            base0F: "#f0c6c6".to_string(), // flamingo
        }
    }
}
