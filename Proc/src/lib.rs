mod unit_expr;

use proc_macro::TokenStream;

#[proc_macro]
pub fn unit_expr(input: TokenStream) -> TokenStream {
    unit_expr::unit_expr(input)
}