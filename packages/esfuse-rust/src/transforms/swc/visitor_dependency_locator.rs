use swc_core::ecma::ast;
use swc_core::ecma::visit::VisitMutWith;

pub struct DependencyLocatorVisitor {
  refs: Vec<&'static mut ast::ImportDecl>,
}

impl VisitMutWith for DependencyLocatorVisitor {
  fn visit_mut_import_decl(&mut self, e: &mut ast::ImportDecl) {
    self.refs.push(e);
  }
}
