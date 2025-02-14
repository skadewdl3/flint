use super::WidgetHandlerOptions;
use crate::{
    arg::ArgKind,
    codegen::{generate_widget_code, util::generate_unique_id},
    widget::{Widget, WidgetKind},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn handle_layout_widget(
    widget: &Widget,
    name: &Ident,
    children: &Vec<Widget>,
    options: &WidgetHandlerOptions,
) -> TokenStream {
    let WidgetHandlerOptions {
        is_top_level,
        parent_id,
        child_index,
        frame,
    } = options;

    let args = &widget.args;
    let layout_index = generate_unique_id() as usize;
    let layout_ident = proc_macro2::Ident::new(&format!("layout_{}", layout_index), name.span());
    let parent_ident = proc_macro2::Ident::new(&format!("chunks_{}", parent_id), name.span());

    let positional_args: Vec<_> = args
        .iter()
        .filter_map(|arg| match &arg.kind {
            ArgKind::Positional => Some(&arg.value),
            _ => None,
        })
        .collect();

    let mut layout_code = quote! {
        let mut #layout_ident = #name::default(#(#positional_args),*)
    };

    // Add named arguments as method calls
    for arg in args {
        if let ArgKind::Named(name) = &arg.kind {
            let value = &arg.value;
            layout_code.extend(quote! {
                .#name(#value)
            });
        }
    }

    // Always end with semicolon after configuration
    layout_code.extend(quote! { ; });

    // Create chunks vector
    let chunks_ident = proc_macro2::Ident::new(&format!("chunks_{}", layout_index), name.span());

    // Split the area - for top level use frame.area(), for nested use the parent's chunk
    let split_code = if *is_top_level {
        quote! {
            let #chunks_ident = #layout_ident.split(#frame .area());
        }
    } else {
        quote! {
            let #chunks_ident = #layout_ident.split(#parent_ident[#child_index]);
        }
    };

    let mut render_statements = quote! {};
    for (idx, child) in children.iter().enumerate() {
        let new_options = WidgetHandlerOptions::new(false, layout_index, idx, frame);

        let child_widget = generate_widget_code(child, &new_options);

        if let WidgetKind::Layout { .. } = child.kind {
            render_statements.extend(quote! {
                #child_widget
            });
        } else if let WidgetKind::Conditional { .. } = child.kind {
            render_statements.extend(quote! {
                #child_widget
            });
        } else {
            render_statements.extend(quote! {
                #frame .render_widget(#child_widget, #chunks_ident[#idx]);
            });
        }
    }

    // Combine everything into a block
    quote! {
        {
            #layout_code
            #split_code
            #render_statements
        }
    }
}
