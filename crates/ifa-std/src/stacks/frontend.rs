//! # Frontend Stack
//!
//! Extensions for web frontend development.
//!
//! **Security Note**: This module includes XSS protection via HTML escaping.
//! Always use `safe_text()` for user input. Only use `dangerous_html()` for
//! trusted content.
//!
//! Targets: WASM compilation via wasm-bindgen
//!
//! Uses: web-sys, wasm-bindgen, leptos/dioxus concepts

use std::collections::HashMap;

/// Escape HTML special characters to prevent XSS attacks
pub fn escape_html(text: &str) -> String {
    text.chars().map(|c| match c {
        '&' => "&amp;".to_string(),
        '<' => "&lt;".to_string(),
        '>' => "&gt;".to_string(),
        '"' => "&quot;".to_string(),
        '\'' => "&#x27;".to_string(),
        '/' => "&#x2F;".to_string(),
        _ => c.to_string(),
    }).collect()
}

/// Escape for HTML attribute context
pub fn escape_attr(text: &str) -> String {
    escape_html(text)
}

/// Wrapper for explicitly trusted HTML content
#[derive(Debug, Clone)]
pub struct SafeHtml(String);

impl SafeHtml {
    /// Create from trusted HTML - ONLY use for known-safe content
    pub fn from_trusted(html: impl Into<String>) -> Self {
        SafeHtml(html.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Virtual DOM Element
#[derive(Debug, Clone)]
pub struct Element {
    pub tag: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub styles: HashMap<String, String>,
    pub children: Vec<Node>,
    pub events: HashMap<String, String>,
}

/// DOM Node (element or text)
#[derive(Debug, Clone)]
pub enum Node {
    Element(Box<Element>),  // Boxed to reduce enum size
    Text(String),  // Already escaped
    RawHtml(SafeHtml),  // Trusted, not escaped
}

impl Element {
    /// Create new element
    pub fn new(tag: &str) -> Self {
        Element {
            tag: tag.to_string(),
            id: None,
            classes: Vec::new(),
            attributes: HashMap::new(),
            styles: HashMap::new(),
            children: Vec::new(),
            events: HashMap::new(),
        }
    }
    
    /// Set ID (escaped)
    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(escape_attr(id));
        self
    }
    
    /// Add class (escaped)
    pub fn class(mut self, class: &str) -> Self {
        self.classes.push(escape_attr(class));
        self
    }
    
    /// Set attribute (both key and value escaped)
    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(escape_attr(key), escape_attr(value));
        self
    }
    
    /// Set style property (escaped)
    pub fn style(mut self, property: &str, value: &str) -> Self {
        self.styles.insert(escape_attr(property), escape_attr(value));
        self
    }
    
    /// Add child element
    pub fn child(mut self, node: Node) -> Self {
        self.children.push(node);
        self
    }
    
    /// Add text child - SAFE: automatically escaped
    pub fn text(mut self, content: &str) -> Self {
        self.children.push(Node::Text(escape_html(content)));
        self
    }
    
    /// Alias for text() - clearly indicates safety
    pub fn safe_text(self, content: &str) -> Self {
        self.text(content)
    }
    
    /// Add raw HTML - DANGEROUS: only for trusted content
    /// 
    /// WARNING: This does NOT escape content. Only use for:
    /// - Sanitized HTML from a trusted sanitizer
    /// - Static HTML templates
    /// - Developer-controlled content
    pub fn dangerous_html(mut self, html: SafeHtml) -> Self {
        self.children.push(Node::RawHtml(html));
        self
    }
    
    /// Add event handler
    pub fn on(mut self, event: &str, handler: &str) -> Self {
        self.events.insert(escape_attr(event), escape_attr(handler));
        self
    }
    
    /// Render to HTML string
    pub fn render(&self) -> String {
        let mut html = format!("<{}", self.tag);
        
        if let Some(ref id) = self.id {
            html.push_str(&format!(" id=\"{}\"", id));
        }
        
        if !self.classes.is_empty() {
            html.push_str(&format!(" class=\"{}\"", self.classes.join(" ")));
        }
        
        for (key, value) in &self.attributes {
            html.push_str(&format!(" {}=\"{}\"", key, value));
        }
        
        if !self.styles.is_empty() {
            let style_str: String = self.styles.iter()
                .map(|(k, v)| format!("{}: {};", k, v))
                .collect::<Vec<_>>()
                .join(" ");
            html.push_str(&format!(" style=\"{}\"", style_str));
        }
        
        for (event, handler) in &self.events {
            html.push_str(&format!(" on{}=\"{}\"", event, handler));
        }
        
        html.push('>');
        
        for child in &self.children {
            match child {
                Node::Element(el) => html.push_str(&el.render()),
                Node::Text(t) => html.push_str(t), // Already escaped
                Node::RawHtml(h) => html.push_str(h.as_str()),
            }
        }
        
        html.push_str(&format!("</{}>", self.tag));
        html
    }
}

/// Helper functions for common elements
pub fn div() -> Element { Element::new("div") }
pub fn span() -> Element { Element::new("span") }
pub fn p() -> Element { Element::new("p") }
pub fn h1() -> Element { Element::new("h1") }
pub fn h2() -> Element { Element::new("h2") }
pub fn h3() -> Element { Element::new("h3") }
pub fn a(href: &str) -> Element { Element::new("a").attr("href", href) }
pub fn img(src: &str) -> Element { Element::new("img").attr("src", src) }
pub fn button() -> Element { Element::new("button") }
pub fn input(input_type: &str) -> Element { Element::new("input").attr("type", input_type) }
pub fn form() -> Element { Element::new("form") }
pub fn ul() -> Element { Element::new("ul") }
pub fn li() -> Element { Element::new("li") }
pub fn table() -> Element { Element::new("table") }
pub fn tr() -> Element { Element::new("tr") }
pub fn td() -> Element { Element::new("td") }
pub fn th() -> Element { Element::new("th") }

/// CSS helper
#[derive(Debug, Clone, Default)]
pub struct Style {
    rules: Vec<(String, HashMap<String, String>)>,
}

impl Style {
    pub fn new() -> Self {
        Style { rules: Vec::new() }
    }
    
    /// Add rule
    pub fn rule(mut self, selector: &str, properties: &[(&str, &str)]) -> Self {
        let props: HashMap<String, String> = properties.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.rules.push((selector.to_string(), props));
        self
    }
    
    /// Render to CSS string
    pub fn render(&self) -> String {
        let mut css = String::new();
        for (selector, props) in &self.rules {
            css.push_str(&format!("{} {{\n", selector));
            for (prop, value) in props {
                css.push_str(&format!("  {}: {};\n", prop, value));
            }
            css.push_str("}\n");
        }
        css
    }
}

/// Component trait
pub trait Component {
    fn render(&self) -> Element;
}

/// Router for SPA
#[derive(Debug, Clone)]
pub struct Router {
    routes: HashMap<String, String>,
    current: String,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
            current: "/".to_string(),
        }
    }
    
    pub fn route(mut self, path: &str, component: &str) -> Self {
        self.routes.insert(path.to_string(), component.to_string());
        self
    }
    
    pub fn navigate(&mut self, path: &str) {
        self.current = path.to_string();
        println!("[ROUTER] Navigating to: {}", path);
    }
    
    pub fn current_component(&self) -> Option<&String> {
        self.routes.get(&self.current)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

/// State management (simple reactive store)
#[derive(Debug, Clone)]
pub struct Store<T: Clone> {
    state: T,
    subscribers: Vec<String>,
}

impl<T: Clone> Store<T> {
    pub fn new(initial: T) -> Self {
        Store {
            state: initial,
            subscribers: Vec::new(),
        }
    }
    
    pub fn get(&self) -> &T {
        &self.state
    }
    
    pub fn set(&mut self, new_state: T) {
        self.state = new_state;
        self.notify();
    }
    
    pub fn subscribe(&mut self, callback: &str) {
        self.subscribers.push(callback.to_string());
    }
    
    fn notify(&self) {
        for sub in &self.subscribers {
            println!("[STORE] Notifying: {}", sub);
        }
    }
}

/// Fetch API wrapper
pub struct Fetch;

impl Fetch {
    /// GET request
    pub fn get(url: &str) -> FetchBuilder {
        FetchBuilder::new("GET", url)
    }
    
    /// POST request
    pub fn post(url: &str) -> FetchBuilder {
        FetchBuilder::new("POST", url)
    }
    
    /// PUT request
    pub fn put(url: &str) -> FetchBuilder {
        FetchBuilder::new("PUT", url)
    }
    
    /// DELETE request
    pub fn delete(url: &str) -> FetchBuilder {
        FetchBuilder::new("DELETE", url)
    }
}

/// Fetch request builder
#[derive(Debug, Clone)]
pub struct FetchBuilder {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl FetchBuilder {
    pub fn new(method: &str, url: &str) -> Self {
        FetchBuilder {
            method: method.to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }
    
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
    
    pub fn json(mut self, data: &str) -> Self {
        self.body = Some(data.to_string());
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self
    }
    
    pub fn send(&self) -> FetchResponse {
        println!("[FETCH] {} {}", self.method, self.url);
        if let Some(ref body) = self.body {
            println!("[FETCH] Body: {}", body);
        }
        // Placeholder response
        FetchResponse {
            status: 200,
            body: "{}".to_string(),
        }
    }
}

/// Fetch response
#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub status: u16,
    pub body: String,
}

/// Local storage wrapper
pub struct LocalStorage;

impl LocalStorage {
    pub fn get(key: &str) -> Option<String> {
        println!("[STORAGE] Get: {}", key);
        None // Placeholder
    }
    
    pub fn set(key: &str, value: &str) {
        println!("[STORAGE] Set: {} = {}", key, value);
    }
    
    pub fn remove(key: &str) {
        println!("[STORAGE] Remove: {}", key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_element_render() {
        let el = div()
            .id("main")
            .class("container")
            .style("color", "red")
            .text("Hello!");
        
        let html = el.render();
        assert!(html.contains("id=\"main\""));
        assert!(html.contains("class=\"container\""));
        assert!(html.contains("Hello!"));
    }
    
    #[test]
    fn test_nested_elements() {
        let el = ul()
            .child(Node::Element(Box::new(li().text("Item 1"))))
            .child(Node::Element(Box::new(li().text("Item 2"))));
        
        let html = el.render();
        assert!(html.contains("<li>Item 1</li>"));
        assert!(html.contains("<li>Item 2</li>"));
    }
    
    #[test]
    fn test_style_render() {
        let style = Style::new()
            .rule(".container", &[("max-width", "1200px"), ("margin", "0 auto")]);
        
        let css = style.render();
        assert!(css.contains(".container"));
        assert!(css.contains("max-width: 1200px"));
    }
}
