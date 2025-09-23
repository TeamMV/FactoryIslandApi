use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, spanned::Spanned, BinOp, Expr, ExprBinary, ExprGroup, ExprParen, ExprPath,
    Result,
};

pub fn unit_expr(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    match lower_expr(&expr) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn lower_expr(expr: &Expr) -> Result<TokenStream2> {
    match expr {
        // Parentheses (or grouped) — just recurse on the inside.
        Expr::Paren(ExprParen { expr: inner, .. }) => lower_expr(inner),
        Expr::Group(ExprGroup { expr: inner, .. }) => lower_expr(inner),

        // Binary ops: only * and / are allowed.
        Expr::Binary(ExprBinary { left, op, right, .. }) => {
            let lhs = lower_expr(left)?;
            let rhs = lower_expr(right)?;
            match op {
                BinOp::Mul(_) => Ok(quote! { <#lhs as ::core::ops::Mul<#rhs>>::Output }),
                BinOp::Div(_) => Ok(quote! { <#lhs as ::core::ops::Div<#rhs>>::Output }),
                // Anything else is a compile error with a nice message.
                other => Err(syn::Error::new(
                    other.span(),
                    "only `*` and `/` operators are allowed in `type_ops!`",
                )),
            }
        }

        // Bare paths (idents or paths like foo::bar::Baz) — emit as-is.
        Expr::Path(ExprPath { path, .. }) => Ok(path.to_token_stream()),

        // Anything else is not supported in this mini-DSL.
        _ => Err(syn::Error::new(
            expr.span(),
            "expected a type path, possibly combined with `*`, `/`, and parentheses",
        )),
    }
}