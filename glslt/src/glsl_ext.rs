//! glsl_lang extensions

use glsl_lang::ast;

/// Extensions for [`glsl_lang::ast::FunIdentifier`]
pub trait FunIdentifierExt {
    /// Return the function name as a string reference
    fn as_ident_or_type_name(&self) -> Option<&String>;
    /// Return the function name as a mutable string reference
    fn as_ident_or_type_name_mut(&mut self) -> Option<&mut String>;
}

impl FunIdentifierExt for ast::FunIdentifier {
    fn as_ident_or_type_name(&self) -> Option<&String> {
        match self {
            Self::Expr(expr) => match &**expr {
                ast::Expr::Variable(ident) => Some(&ident.0),
                _ => None,
            },
            Self::TypeSpecifier(ast::TypeSpecifier {
                ty: ast::TypeSpecifierNonArray::TypeName(tn),
                array_specifier: None,
            }) => Some(&tn.0),
            _ => None,
        }
    }

    fn as_ident_or_type_name_mut(&mut self) -> Option<&mut String> {
        match self {
            Self::Expr(expr) => match &mut **expr {
                ast::Expr::Variable(ident) => Some(&mut ident.0),
                _ => None,
            },
            Self::TypeSpecifier(ast::TypeSpecifier {
                ty: ast::TypeSpecifierNonArray::TypeName(tn),
                array_specifier: None,
            }) => Some(&mut tn.0),
            _ => None,
        }
    }
}
