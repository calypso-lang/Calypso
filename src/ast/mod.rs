use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fmt::{self, Display},
    marker::PhantomData,
};

use crate::{
    arena::{Arena, IdLike},
    ctxt::GlobalCtxt,
    parse::Span,
    symbol::{Ident, Symbol},
};

pub const DUMMY_AST_ID: AstId = AstId(0);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AstId(u32);

impl AstId {
    pub fn from_raw(id: u32) -> AstId {
        AstId(id)
    }

    pub fn into_raw(self) -> u32 {
        self.0
    }
}

impl Display for AstId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Item<Id: Copy + Clone>(usize, PhantomData<Id>);

impl<Id: Copy + Clone> IdLike for Item<Id> {
    fn from_raw(index: usize) -> Self {
        Self(index, PhantomData)
    }

    fn into_raw(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct ItemData<Id: Copy + Clone> {
    pub id: AstId,
    pub kind: ItemKind<Id>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ItemKind<Id: Copy + Clone> {
    Function {
        name: Id,
        args: im::Vector<(Id, Ty<Id>)>,
        ret_ty: Option<Ty<Id>>,
        body: Expr<Id>,
    },
}

impl Item<Ident> {
    pub fn new(gcx: &GlobalCtxt, kind: ItemKind<Ident>, span: Span) -> Item<Ident> {
        let id = gcx.arenas.ast.next_ast_id();
        let item = gcx
            .arenas
            .ast
            .item
            .borrow_mut()
            .push(ItemData { id, kind, span });
        gcx.arenas.ast.insert_node(id, Node::Item(item));
        item
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Expr<Id: Copy + Clone>(usize, PhantomData<Id>);

impl<Id: Copy + Clone> IdLike for Expr<Id> {
    fn from_raw(index: usize) -> Self {
        Self(index, PhantomData)
    }

    fn into_raw(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct ExprData<Id: Copy + Clone> {
    pub id: AstId,
    pub kind: ExprKind<Id>,
    pub span: Span,
}

impl Expr<Ident> {
    pub fn new(gcx: &GlobalCtxt, kind: ExprKind<Ident>, span: Span) -> Expr<Ident> {
        let id = gcx.arenas.ast.next_ast_id();
        let expr = gcx
            .arenas
            .ast
            .expr
            .borrow_mut()
            .push(ExprData { id, kind, span });
        gcx.arenas.ast.insert_node(id, Node::Expr(expr));
        expr
    }
}

#[derive(Clone, Debug)]
pub enum ExprKind<Id: Copy + Clone> {
    Let {
        is_mut: bool,
        name: Id,
        ty: Option<Ty<Id>>,
        val: Expr<Id>,
    },
    BinaryOp {
        left: Expr<Id>,
        kind: BinOpKind,
        right: Expr<Id>,
    },
    UnaryMinus(Expr<Id>),
    UnaryNot(Expr<Id>),
    Do {
        exprs: im::Vector<Expr<Id>>,
    },
    Numeral(Numeral),
    Ident(Id),
    Bool(bool),
    Error,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Ty<Id: Copy + Clone>(usize, PhantomData<Id>);

impl<Id: Copy + Clone> IdLike for Ty<Id> {
    fn from_raw(index: usize) -> Self {
        Self(index, PhantomData)
    }

    fn into_raw(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct TyData<Id: Copy + Clone> {
    pub id: AstId,
    pub kind: TyKind<Id>,
    pub span: Span,
}

impl Ty<Ident> {
    pub fn new(gcx: &GlobalCtxt, kind: TyKind<Ident>, span: Span) -> Ty<Ident> {
        let id = gcx.arenas.ast.next_ast_id();
        let ty = gcx
            .arenas
            .ast
            .ty
            .borrow_mut()
            .push(TyData { id, kind, span });
        gcx.arenas.ast.insert_node(id, Node::Ty(ty));
        ty
    }
}

#[derive(Clone, Debug)]
pub enum TyKind<Id: Copy + Clone> {
    Primitive(Primitive),
    Function(im::Vector<Ty<Id>>, Option<Ty<Id>>),
}

#[derive(Copy, Clone, Debug)]
pub enum Primitive {
    Bool,
    UInt,
    Int,
}

#[derive(Copy, Clone, Debug)]
pub enum BinOpKind {
    LogicalOr,
    LogicalAnd,
    BitOr,
    BitAnd,
    BitXor,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    BitShiftLeft,
    BitShiftRight,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
}

#[derive(Debug, Default)]
pub struct Parentage {
    pub map: HashMap<AstId, AstId>,
}

#[derive(Debug)]
pub struct AstArenas {
    pub expr: RefCell<Arena<Expr<Ident>, ExprData<Ident>>>,
    pub ty: RefCell<Arena<Ty<Ident>, TyData<Ident>>>,
    pub item: RefCell<Arena<Item<Ident>, ItemData<Ident>>>,
    pub parentage: RefCell<Parentage>,
    next_ast_id: Cell<u32>,
    ast_id_to_node: RefCell<HashMap<AstId, Node<Ident>>>,
}

impl AstArenas {
    pub fn clear(&self) {
        self.next_ast_id.set(1);
        self.ast_id_to_node.borrow_mut().clear();
        self.parentage.borrow_mut().map.clear();
    }

    pub fn expr(&self, id: Expr<Ident>) -> ExprData<Ident> {
        self.expr.borrow()[id].clone()
    }

    pub fn ty(&self, id: Ty<Ident>) -> TyData<Ident> {
        self.ty.borrow()[id].clone()
    }

    pub fn item(&self, id: Item<Ident>) -> ItemData<Ident> {
        self.item.borrow()[id].clone()
    }

    pub fn next_ast_id(&self) -> AstId {
        let id = self.next_ast_id.get();
        assert!(id < u32::MAX);
        self.next_ast_id.set(id + 1);
        AstId::from_raw(id)
    }

    pub fn get_node_by_id(&self, id: AstId) -> Option<Node<Ident>> {
        self.ast_id_to_node.borrow().get(&id).copied()
    }

    pub fn into_iter_nodes(&self) -> impl Iterator<Item = Node<Ident>> {
        let v = self.ast_id_to_node.borrow();
        v.values().copied().collect::<Vec<_>>().into_iter()
    }

    fn insert_node(&self, id: AstId, node: Node<Ident>) {
        self.ast_id_to_node.borrow_mut().insert(id, node);
    }
}

impl Default for AstArenas {
    fn default() -> Self {
        Self {
            expr: Default::default(),
            ty: Default::default(),
            item: Default::default(),
            parentage: Default::default(),
            next_ast_id: Cell::new(1),
            ast_id_to_node: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Node<Id: Copy + Clone> {
    Expr(Expr<Id>),
    Ty(Ty<Id>),
    Item(Item<Id>),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
/// Number radixes.
pub enum Radix {
    /// No prefix (`0d` by default)
    None,
    /// `0d`
    Decimal,
    /// `0b`
    Binary,
    /// `0o`
    Octal,
    /// `0x`
    Hexadecimal,
}

impl Radix {
    #[must_use]
    pub fn radix(self) -> u32 {
        match self {
            Self::None | Self::Decimal => 10,
            Self::Binary => 2,
            Self::Octal => 8,
            Self::Hexadecimal => 16,
        }
    }
}

impl Display for Radix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decimal => write!(f, "0d"),
            Self::Binary => write!(f, "0b"),
            Self::Octal => write!(f, "0o"),
            Self::Hexadecimal => write!(f, "0x"),
            Self::None => Ok(()),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
/// Number suffixes.
pub enum Suffix {
    /// `u`
    Uint,
    /// `s`
    Sint,
}

impl Display for Suffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uint => write!(f, "u"),
            Self::Sint => write!(f, "s"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Numeral {
    Integer {
        suffix: Option<Suffix>,
        radix: Radix,
        sym: Symbol,
    },
    Float {
        from_integer: bool,
        sym: Symbol,
    },
}
