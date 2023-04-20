use swc_core::common::{DUMMY_SP};

use swc_core::ecma::ast::{self};
use swc_core::ecma::utils::{quote_ident, quote_str};
use swc_core::ecma::visit::{VisitMut, VisitMutWith};

use swc_core::{quote_expr};

use super::OnTransformSwcOpts;
pub struct TransformVisitor<'a> {
  pub opts: &'a OnTransformSwcOpts,
  pub url: String,
  pub imports: Vec<(String, swc_common::Span)>,
}

fn interlace_vectors<T>(vec1: Vec<T>, vec2: Vec<T>) -> Vec<T> where T: Clone {
  let mut result: Vec<T> = Vec::new();

  let length1 = vec1.len();
  let length2 = vec2.len();

  let max_length = if length1 > length2 {
    length1
  } else {
    length2
  };

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

impl<'a> TransformVisitor<'a> {
  fn register_import(&mut self, expr: &ast::Expr) {
    match expr {
      ast::Expr::Lit(ast::Lit::Str(lit_str)) => {
        let import = lit_str.value.to_string();
        let span = lit_str.span;
        
        self.imports.push((import, span));
      }

      ast::Expr::Tpl(tpl) => {
        if tpl.quasis.len() == 1 {
          let first_quasi = tpl.quasis.first()
            .expect("Should have a quasi");
          
          let quasi_value = first_quasi.cooked.as_ref()
            .expect("Should have a cooked value");
          
          self.imports.push((quasi_value.to_string(), tpl.span));
        }
      }

      _ => {}
    }
  }
}

impl<'a> VisitMut for TransformVisitor<'a> {
  fn visit_mut_module(&mut self, e: &mut ast::Module) {
    e.visit_mut_children_with(self);

    let mut stmts: Vec<ast::Stmt> = e.body.clone().into_iter().filter_map(|i| match i {
      ast::ModuleItem::Stmt(stmt) => Some(stmt),
      _ => None
    }).collect();

    if self.opts.promisify_body {
      let async_fn = ast::Expr::Arrow(ast::ArrowExpr {
        span: DUMMY_SP,
        params: vec![],
        body: ast::BlockStmtOrExpr::BlockStmt(ast::BlockStmt {
          span: DUMMY_SP,
          stmts,
        }),
        is_async: true,
        is_generator: false,
        type_params: None,
        return_type: None,
      });

      let async_expr = quote_expr!("
        module.exports = Promise.resolve({
          exports: {},
        }).then((module, exports = module.exports) => {
          return ($async_fn)().then(() => module.exports);
        })
      ",
        async_fn: Expr = async_fn,
      );

      stmts = vec![
        ast::ExprStmt {
          span: DUMMY_SP,
          expr: async_expr,
        }.into(),
      ];
    }

    if self.opts.use_esfuse_runtime {
      let mod_vars = vec![
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
        ast::Pat::Ident(ast::BindingIdent {
          id: quote_ident!("__filename"),
          type_ann: None,
        }),
        ast::Pat::Ident(ast::BindingIdent {
          id: quote_ident!("__dirname"),
          type_ann: None,
        }),
      ];

      let mod_fn = ast::Expr::Arrow(ast::ArrowExpr {
        span: DUMMY_SP,
        params: mod_vars,
        body: ast::BlockStmtOrExpr::BlockStmt(ast::BlockStmt {
          span: DUMMY_SP,
          stmts,
        }),
        is_async: false,
        is_generator: false,
        type_params: None,
        return_type: None,
      });

      let mod_expr = quote_expr!("
        $esfuse$.define($mod_url, $mod_fn);
      ",
        mod_url:Expr = quote_str!(self.url.clone()).into(),
        mod_fn: Expr = mod_fn,
      );

      stmts = vec![
        ast::ExprStmt {
          span: DUMMY_SP,
          expr: mod_expr,
        }.into(),
      ];
    }

    e.body = stmts.drain(0..).map(|stmt| {
      ast::ModuleItem::Stmt(stmt)
    }).collect();
}

  fn visit_mut_expr(&mut self, e: &mut ast::Expr) {
    if let Some(call) = e.as_call() {
      if call.callee.is_import() && call.args.len() == 1 {
        if let ast::Expr::Tpl(arg) = &*call.args[0].expr {
          if !arg.exprs.is_empty() {
            let new_import_specifier = interlace_vectors(
              arg.quasis.iter().map(|q| q.cooked.as_ref().map_or("".to_string(), |v| v.to_string())).collect(),
              arg.exprs.iter().enumerate().map(|(i, _q)| format!("[...t{i}]")).collect(),
            ).join("");

            let dynamic_parameters = ast::ObjectLit {
              span: DUMMY_SP,
              props: arg.exprs.iter().enumerate().map(|(i, q)| ast::PropOrSpread::Prop(ast::Prop::KeyValue(ast::KeyValueProp {
                key: swc_core::ecma::ast::PropName::Ident(quote_ident!(format!("t{i}"))),
                value: q.to_owned(),
              }).into())).collect(),
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

    e.visit_mut_children_with(self);
  }

  fn visit_mut_call_expr(&mut self, e: &mut ast::CallExpr) {
    e.visit_mut_children_with(self);

    if e.callee.is_import() {
      self.register_import(&e.args[0].expr);
      e.callee = ast::Callee::Expr(quote_expr!("require.import"));
    }

    if let ast::Callee::Expr(callee) = &e.callee {
      if let ast::Expr::Ident(callee_ident) = &**callee {
        if callee_ident.sym.to_string() == "require" {
          self.register_import(&e.args[0].expr);
        }
      }
    }
  }
}
