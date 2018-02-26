#![feature(proc_macro)]
#![recursion_limit = "128"]
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use syn::{Block, Expr, ExprBlock, ExprForLoop, ExprLit, ExprRange, Item, ItemFn, Lit, Pat,
          PatIdent, RangeLimits, Stmt};
use syn::token::Brace;
use proc_macro::TokenStream;

/// Attribute used to unroll for loops found inside a function block.
#[proc_macro_attribute]
pub fn unroll_for_loops(_meta: TokenStream, input: TokenStream) -> TokenStream {
    let item: Item = syn::parse(input).expect("Failed to parse input.");

    if let Item::Fn(item_fn) = item {
        let new_block = {
            let &ItemFn {
                block: ref box_block,
                ..
            } = &item_fn;
            unroll_in_block(&**box_block)
        };
        let new_item = Item::Fn(ItemFn {
            block: Box::new(new_block),
            ..item_fn
        });
        quote! ( #new_item ).into()
    } else {
        quote! ( #item ).into()
    }
}

/// Routine to unroll for loops within a block
fn unroll_in_block(block: &Block) -> Block {
    let &Block {
        ref brace_token,
        ref stmts,
    } = block;
    let mut new_stmts = Vec::new();
    for stmt in stmts.iter() {
        new_stmts.push(unroll(stmt));
    }
    Block {
        brace_token: brace_token.clone(),
        stmts: new_stmts,
    }
}

/// Routine to unroll a for loop statement, or return the statement unchanged if it's not a for
/// loop.
fn unroll(stmt: &Stmt) -> Stmt {
    // impose a scope that we can break out of so we can return stmt without copying it.
    if let &Stmt::Expr(Expr::ForLoop(ref for_loop)) = stmt {
        let ExprForLoop {
            ref pat,
            expr: ref range_expr,
            ref body,
            ..
        } = *for_loop;
        let recurse_on_forloop_body = || {
            Stmt::Expr(Expr::ForLoop(ExprForLoop {
                body: unroll_in_block(&*body),
                ..(*for_loop).clone()
            }))
        };

        if let Pat::Ident(PatIdent {
            ref by_ref,
            ref mutability,
            ref ident,
            ref subpat,
        }) = **pat
        {
            // Don't know how to deal with these so skip and return the original.
            if !by_ref.is_none() || !mutability.is_none() || !subpat.is_none() {
                return recurse_on_forloop_body();
            }
            let idx = ident; // got the index variable name

            if let Expr::Range(ExprRange {
                from: ref mb_box_from,
                ref limits,
                to: ref mb_box_to,
                ..
            }) = **range_expr
            {
                // Parse mb_box_from
                let begin = if let Some(ref box_from) = *mb_box_from {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Int(ref lit_int),
                        ..
                    }) = **box_from
                    {
                        lit_int.value() as usize
                    } else {
                        return recurse_on_forloop_body();
                    }
                } else {
                    0
                };

                // Parse mb_box_to
                let end = if let Some(ref box_to) = *mb_box_to {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Int(ref lit_int),
                        ..
                    }) = **box_to
                    {
                        lit_int.value() as usize
                    } else {
                        return recurse_on_forloop_body();
                    }
                } else {
                    // we need to know where the limit is to know how much to unroll by.
                    return recurse_on_forloop_body();
                } + if let &RangeLimits::Closed(_) = limits {
                    1
                } else {
                    0
                };

                let mut stmts = Vec::new();
                for i in begin..end {
                    let block_ts: TokenStream = quote!(
                        #[allow(non_upper_case_globals)]
                        {
                            const #idx: usize = #i;
                            #body
                        }).into();
                    stmts.push(
                        syn::parse::<Stmt>(block_ts).expect("Couldn't parse block into stmt."),
                    );
                }
                let block = Block {
                    brace_token: Brace::default(),
                    stmts,
                };
                return Stmt::Expr(Expr::Block(ExprBlock {
                    attrs: Vec::new(),
                    block,
                }));
            } else {
                return recurse_on_forloop_body();
            }
        } else {
            return recurse_on_forloop_body();
        }
    } else if let &Stmt::Expr(Expr::Block(ref expr_block)) = stmt {
        let ExprBlock { ref block, .. } = *expr_block;
        Stmt::Expr(Expr::Block(ExprBlock {
            block: unroll_in_block(&*block),
            ..(*expr_block).clone()
        }))
    } else if let &Stmt::Semi(Expr::Block(ref expr_block), semi) = stmt {
        let ExprBlock { ref block, .. } = *expr_block;
        Stmt::Semi(
            Expr::Block(ExprBlock {
                block: unroll_in_block(&*block),
                ..(*expr_block).clone()
            }),
            semi,
        )
    } else {
        (*stmt).clone()
    }
}
