use swc_core::common::{DUMMY_SP, FileName};
use swc_core::common::util::take::Take;

use swc_core::ecma::ast;
use swc_core::ecma::utils::{quote_ident, quote_str};
use swc_core::ecma::visit::{VisitMut, VisitMutWith};

use swc_core::{quote_expr};

pub struct TransformVisitor {
    pub filename: FileName,
}

fn interlace_vectors<T>(vec1: Vec<T>, vec2: Vec<T>) -> Vec<T> where T: Clone {
    let mut result: Vec<T> = Vec::new();
    let length1 = vec1.len();
    let length2 = vec2.len();
    let max_length = if length1 > length2 { length1 } else { length2 };

    for i in 0..max_length {
        if i < length1 {
            result.push(vec1[i].clone());
        }
        if i < length2 {
            result.push(vec2[i].clone());
        }
    }

    result
}

impl VisitMut for TransformVisitor {
    fn visit_mut_module(&mut self, e: &mut ast::Module) {
        e.visit_mut_children_with(self);

        e.body = [
            ast::ModuleItem::Stmt(
                ast::Stmt::Expr(ast::ExprStmt {
                    span: DUMMY_SP,
                    expr: ast::CallExpr {
                        span: DUMMY_SP,
                        callee: ast::Callee::Expr(quote_expr!("$esfuse$.define")),
                        args: [
                            ast::ExprOrSpread {
                                spread: None,
                                expr: self.filename.to_string().into(),
                            },
                            ast::ExprOrSpread {
                                spread: None,
                                expr: ast::ArrowExpr {
                                    span: DUMMY_SP,
                                    params: [
                                        ast::Pat::Ident(ast::BindingIdent {
                                            id: quote_ident!("module"),
                                            type_ann: None,
                                        }),
                                        ast::Pat::Ident(ast::BindingIdent {
                                            id: quote_ident!("exports"),
                                            type_ann: None,
                                        }),
                                        ast::Pat::Ident(ast::BindingIdent {
                                            id: quote_ident!("require"),
                                            type_ann: None,
                                        }),
                                    ].to_vec(),
                                    body: ast::BlockStmtOrExpr::BlockStmt(ast::BlockStmt {
                                        span: DUMMY_SP,
                                        stmts: e.body.to_owned().into_iter().filter_map(|i| match i {
                                            ast::ModuleItem::Stmt(stmt) => Some(stmt),
                                            _ => None
                                        }).collect(),
                                    }),
                                    is_async: false,
                                    is_generator: false,
                                    type_params: Take::dummy(),
                                    return_type: Take::dummy(),
                                }.into(),    
                            }.into(),
                        ].to_vec(),
                        type_args: Take::dummy(),
                    }.into(),
                })
            ),
        ].to_vec();
    }

    fn visit_mut_expr(&mut self, e: &mut ast::Expr) {
        if let Some(call) = e.as_call() {
            if call.callee.is_import() {
                if call.args.len() == 1 {
                    if let ast::Expr::Tpl(arg) = &*call.args[0].expr {
                        if arg.exprs.len() > 0 {
                            let new_import_specifier = interlace_vectors(
                                arg.quasis.iter().map(|q| q.cooked.as_ref().map_or("".to_string(), |v| v.to_string())).collect(),
                                arg.exprs.iter().enumerate().map(|(i, _q)| format!("[...t{i}]")).collect(),
                            ).join("");

                            let dynamic_parameters = ast::ObjectLit {
                                span: DUMMY_SP,
                                props: arg.exprs.iter().enumerate().map(|(i, q)| ast::PropOrSpread::Prop(ast::Prop::KeyValue(ast::KeyValueProp {
                                    key: swc_core::ecma::ast::PropName::Ident(quote_ident!(format!("t{i}"))),
                                    value: q.to_owned(),
                                }).into()).into()).collect(),
                            };

                            *e = *quote_expr!(
                                "(args => import($path).then(m => m.fetch(args)))($params)",
                                path: Expr = quote_str!(new_import_specifier).into(),
                                params: Expr = dynamic_parameters.into(),
                            );
                        }
                    }
                }
            }
        }

        e.visit_mut_children_with(self);
    }

    fn visit_mut_call_expr(&mut self, e: &mut ast::CallExpr) {
        e.visit_mut_children_with(self);

        if e.callee.is_import() {
            e.callee = ast::Callee::Expr(quote_expr!("$esfuse$.import"));
        }
    }
}
