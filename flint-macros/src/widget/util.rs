use syn::Ident;

/// Checks if the given identifier represents a Layout widget
///
/// # Arguments
/// * `name` - The identifier to check
///
/// # Returns
/// `true` if the identifier is "Layout", `false` otherwise
pub fn is_layout_widget(name: &Ident) -> bool {
    name.to_string() == "Layout"
}
