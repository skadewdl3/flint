use syn::Ident;

pub fn is_layout_widget(name: &Ident) -> bool {
    name.to_string() == "Layout"
}
