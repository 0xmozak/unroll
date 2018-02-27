#![feature(proc_macro)]
#![recursion_limit = "128"]
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use syn::{Block, Expr, ExprBlock, ExprForLoop, ExprLit, ExprRange, Item, ItemFn, Lit, Pat,
          PatIdent, RangeLimits, Stmt, ExprIf, ExprIfLet};
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
        if let &Stmt::Expr(ref expr) = stmt {
            new_stmts.push(Stmt::Expr(unroll(expr)));
        } else if let &Stmt::Semi(ref expr, semi) = stmt {
            new_stmts.push(Stmt::Semi(unroll(expr), semi));
        }
    }
    Block {
        brace_token: brace_token.clone(),
        stmts: new_stmts,
    }
}

/// Routine to unroll a for loop statement, or return the statement unchanged if it's not a for
/// loop.
fn unroll(expr: &Expr) -> Expr {
    // impose a scope that we can break out of so we can return stmt without copying it.
    if let &Expr::ForLoop(ref for_loop) = expr {
        let ExprForLoop {
            ref pat,
            expr: ref range_expr,
            ref body,
            ..
        } = *for_loop;

        let new_body = unroll_in_block(&*body);

        let forloop_with_body = |body| {
            Expr::ForLoop(ExprForLoop {
                body,
                ..(*for_loop).clone()
            })
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
                return forloop_with_body(new_body);
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
                        return forloop_with_body(new_body);
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
                        return forloop_with_body(new_body);
                    }
                } else {
                    // we need to know where the limit is to know how much to unroll by.
                    return forloop_with_body(new_body);
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
                            #new_body
                        }).into();
                    stmts.push(
                        syn::parse::<Stmt>(block_ts).expect("Couldn't parse block into stmt."),
                    );
                }
                let block = Block {
                    brace_token: Brace::default(),
                    stmts,
                };
                return Expr::Block(ExprBlock {
                    attrs: Vec::new(),
                    block,
                });
            } else {
                forloop_with_body(new_body)
            }
        } else {
            forloop_with_body(new_body)
        }
    } else if let &Expr::If(ref if_expr) = expr {
        let ExprIf {
            ref cond,
            ref then_branch,
            ref else_branch,
            ..
        } = *if_expr;
        Expr::If(ExprIf {
            cond: Box::new(unroll(&**cond)),
            then_branch: unroll_in_block(&*then_branch),
            else_branch: else_branch.as_ref().map(|x| (x.0, Box::new(unroll(&*x.1)))),
            ..(*if_expr).clone()
        })
    } else if let &Expr::IfLet(ref if_expr) = expr {
        let ExprIfLet {
            ref expr,
            ref then_branch,
            ref else_branch,
            ..
        } = *if_expr;
        Expr::IfLet(ExprIfLet {
            expr: Box::new(unroll(&**expr)),
            then_branch: unroll_in_block(&*then_branch),
            else_branch: else_branch.as_ref().map(|x| (x.0, Box::new(unroll(&*x.1)))),
            ..(*if_expr).clone()
        })
    } else if let &Expr::Block(ref expr_block) = expr {
        let ExprBlock { ref block, .. } = *expr_block;
        Expr::Block(ExprBlock {
            block: unroll_in_block(&*block),
            ..(*expr_block).clone()
        })
    } else {
        (*expr).clone()
    }
}
