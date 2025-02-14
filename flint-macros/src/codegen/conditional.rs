use super::{generate_widget_code, WidgetHandlerOptions};
use crate::{
    arg::{Arg, ArgKind},
    widget::{Widget, WidgetKind},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Ident};

pub fn handle_conditional_widget(
    condition: &Expr,
    if_child: &Box<Widget>,
    else_child: &Option<Box<Widget>>,
    options: &WidgetHandlerOptions,
) -> TokenStream {
    let layout = Widget {
        args: vec![Arg {
            value: syn::parse2(quote! {
                if #condition {
                    [Constraint::Min(0)]
                } else {
                    [Constraint::Length(0)]
                }
            })
            .expect("Failed to parse constraints expression"),
            kind: ArgKind::Named(Ident::new("constraints", proc_macro2::Span::call_site())),
        }],
        kind: WidgetKind::Layout {
            name: Ident::new("Layout", proc_macro2::Span::call_site()),
            children: vec![*if_child.clone()],
        },
    };

    let else_layout = else_child.as_ref().map(|else_child| Widget {
        args: vec![Arg {
            value: syn::parse2(quote! {
                if #condition {
                    [Constraint::Length(0)]
                } else {
                    [Constraint::Min(0)]
                }
            })
            .expect("Failed to parse constraints expression"),
            kind: ArgKind::Named(Ident::new("constraints", proc_macro2::Span::call_site())),
        }],
        kind: WidgetKind::Layout {
            name: Ident::new("Layout", proc_macro2::Span::call_site()),
            children: vec![(**else_child).clone()],
        },
    });

    let if_layout = generate_widget_code(&layout, options);

    if let Some(else_layout) = else_layout {
        let else_code = generate_widget_code(&else_layout, options);
        quote! {
            {
                #if_layout
                #else_code
            }
        }
    } else {
        if_layout
    }
}
