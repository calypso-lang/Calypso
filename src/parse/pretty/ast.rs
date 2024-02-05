use pretty::BoxDoc;

use crate::ast::{BinOpKind, Expr, ExprKind, Numeral, Primitive, Ty, TyKind};

use super::Printer;

impl<'gcx> Printer<'gcx> {
    pub fn print_expr(&self, expr: Expr) -> BoxDoc {
        let arena = &self.gcx.arenas.ast;
        match arena.expr(expr).kind {
            ExprKind::Let {
                is_mut,
                name,
                ty,
                val,
            } => {
                // TODO
                let varlist = vec![(is_mut, name, ty, val)];
                BoxDoc::text("(")
                    .append("let")
                    .append(BoxDoc::space())
                    .append(BoxDoc::text("["))
                    .append(
                        BoxDoc::intersperse(
                            varlist.into_iter().map(|(is_mut, var, ty, expr)| {
                                if is_mut {
                                    BoxDoc::text("(mut").append(BoxDoc::space())
                                } else {
                                    BoxDoc::nil()
                                }
                                .append(BoxDoc::text(var.as_str()))
                                .append(if is_mut {
                                    BoxDoc::text(")")
                                } else {
                                    BoxDoc::nil()
                                })
                                .append(if let Some(ty) = ty {
                                    BoxDoc::space()
                                        .append(self.print_ty(ty))
                                        .nest((var.as_str().len() + 1) as isize)
                                } else {
                                    BoxDoc::nil()
                                })
                                .append(
                                    BoxDoc::space()
                                        .append(self.print_expr(expr))
                                        .nest((var.as_str().len() + 1) as isize),
                                )
                                .group()
                            }),
                            BoxDoc::line(),
                        )
                        .nest(6),
                    )
                    .append(BoxDoc::text("]"))
            }
            ExprKind::BinaryOp { left, kind, right } => {
                BoxDoc::text(format!("({}", self.print_binopkind(kind)))
                    .append(
                        BoxDoc::space()
                            .append(self.print_expr(left))
                            .append(BoxDoc::line())
                            .append(self.print_expr(right))
                            .group()
                            .append(BoxDoc::text(")")),
                    )
                    .nest((self.print_binopkind(kind).len() + 2) as isize)
            }
            ExprKind::UnaryMinus(expr) => BoxDoc::text("(neg")
                .append(BoxDoc::space().append(self.print_expr(expr)).nest(5))
                .append(BoxDoc::text(")")),
            ExprKind::UnaryNot(expr) => BoxDoc::text("(not")
                .append(BoxDoc::space().append(self.print_expr(expr)).nest(5))
                .append(BoxDoc::text(")")),
            ExprKind::Do { exprs } => BoxDoc::text("(do").append(
                BoxDoc::line()
                    .append(
                        BoxDoc::intersperse(
                            exprs.into_iter().map(|expr| self.print_expr(expr).group()),
                            BoxDoc::line(),
                        )
                        .append(BoxDoc::text(")")),
                    )
                    .nest(4),
            ),
            ExprKind::Numeral(Numeral::Float { sym, .. } | Numeral::Integer { sym, .. }) => {
                BoxDoc::text(sym.as_str())
            }
            ExprKind::Ident(ident) => BoxDoc::text(ident.as_str()),
            ExprKind::Bool(b) => BoxDoc::text(format!("{}", b)),
            ExprKind::Error => BoxDoc::text("<error>"),
        }
    }

    fn print_binopkind(&self, kind: BinOpKind) -> &'static str {
        match kind {
            BinOpKind::LogicalOr => "||",
            BinOpKind::LogicalAnd => "&&",
            BinOpKind::BitOr => "|",
            BinOpKind::BitAnd => "&",
            BinOpKind::BitXor => "^",
            BinOpKind::Equal => "==",
            BinOpKind::NotEqual => "!=",
            BinOpKind::LessThan => "<",
            BinOpKind::GreaterThan => ">",
            BinOpKind::LessEqual => "<=",
            BinOpKind::GreaterEqual => ">=",
            BinOpKind::BitShiftLeft => "<<",
            BinOpKind::BitShiftRight => ">>",
            BinOpKind::Add => "+",
            BinOpKind::Subtract => "-",
            BinOpKind::Multiply => "*",
            BinOpKind::Divide => "/",
            BinOpKind::Modulo => "%",
            BinOpKind::Power => "**",
        }
    }

    pub fn print_ty(&self, ty: Ty) -> BoxDoc {
        let arena = &self.gcx.arenas.ast;
        match arena.ty(ty).kind {
            TyKind::Primitive(Primitive::Bool) => BoxDoc::text("bool"),
            TyKind::Primitive(Primitive::Uint) => BoxDoc::text("uint"),
        }
    }
}
