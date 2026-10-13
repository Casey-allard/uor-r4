//! Tensor Operation Enums and Traits
//!
#![allow(clippy::redundant_closure_call)]
use crate::Tensor;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Le,
    Ge,
    Lt,
    Gt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReduceOp {
    Sum,
    Min,
    Max,
    ArgMin,
    ArgMax,
}

impl ReduceOp {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::ArgMax => "argmax",
            Self::ArgMin => "argmin",
            Self::Min => "min",
            Self::Max => "max",
            Self::Sum => "sum",
        }
    }
}

// These ops return the same type as their input type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Mul,
    Sub,
    Div,
    Maximum,
    Minimum,
}

// Unary ops with no argument
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Exp,
    Log,
    Sin,
    Cos,
    Abs,
    Neg,
    Recip,
    Sqr,
    Sqrt,
    Gelu,
    GeluErf,
    Erf,
    Relu,
    Silu,
    Tanh,
    Floor,
    Ceil,
    Round,
    Sign,
}

#[derive(Clone)]
pub enum Op {
    Binary(Tensor, Tensor, BinaryOp),
    Unary(Tensor, UnaryOp),
    Cmp(Tensor, CmpOp),
    // The third argument is the reduced shape with `keepdim=true`.
    Reduce(Tensor, ReduceOp, Vec<usize>),
    Matmul(Tensor, Tensor),
    Gather(Tensor, Tensor, usize),
    Scatter(Tensor, Tensor, Tensor, usize),
    ScatterAdd(Tensor, Tensor, Tensor, usize),
    IndexSelect(Tensor, Tensor, usize),
    IndexAdd(Tensor, Tensor, Tensor, usize),
    WhereCond(Tensor, Tensor, Tensor),

    #[allow(dead_code)]
    Conv1D {
        arg: Tensor,
        kernel: Tensor,
        padding: usize,
        stride: usize,
        dilation: usize,
    },

    #[allow(dead_code)]
    ConvTranspose1D {
        arg: Tensor,
        kernel: Tensor,
        padding: usize,
        output_padding: usize,
        stride: usize,
        dilation: usize,
    },

    #[allow(dead_code)]
    Conv2D {
        arg: Tensor,
        kernel: Tensor,
        padding: usize,
        stride: usize,
        dilation: usize,
    },

    #[allow(dead_code)]
    ConvTranspose2D {
        arg: Tensor,
        kernel: Tensor,
        padding: usize,
        output_padding: usize,
        stride: usize,
        dilation: usize,
    },

    AvgPool2D {
        arg: Tensor,
        kernel_size: (usize, usize),
        stride: (usize, usize),
    },

    MaxPool2D {
        arg: Tensor,
        kernel_size: (usize, usize),
        stride: (usize, usize),
    },

    UpsampleNearest1D {
        arg: Tensor,
        target_size: usize,
    },
    UpsampleNearest2D {
        arg: Tensor,
        target_h: usize,
        target_w: usize,
    },
    UpsampleBilinear2D {
        arg: Tensor,
        target_h: usize,
        target_w: usize,
        align_corners: bool,
    },

    Cat(Vec<Tensor>, usize),

    #[allow(dead_code)] // add is currently unused.
    Affine {
        arg: Tensor,
        mul: f64,
        add: f64,
    },
    ToDType(Tensor),
    Copy(Tensor),
    Broadcast(Tensor),
    Narrow(Tensor, usize, usize, usize),
    SliceScatter0(Tensor, Tensor, usize),
    Reshape(Tensor),
    ToDevice(Tensor),
    Transpose(Tensor, usize, usize),
    Permute(Tensor, Vec<usize>),
    Elu(Tensor, f64),
    Powf(Tensor, f64),
    CustomOp1(
        Tensor,
        std::sync::Arc<Box<dyn crate::CustomOp1 + Send + Sync>>,
    ),
    CustomOp2(
        Tensor,
        Tensor,
        std::sync::Arc<Box<dyn crate::CustomOp2 + Send + Sync>>,
    ),
    CustomOp3(
        Tensor,
        Tensor,
        Tensor,
        std::sync::Arc<Box<dyn crate::CustomOp3 + Send + Sync>>,
    ),
}

/// CUDA operation metadata. CPU evaluation is deliberately absent; the CPU
/// model backend returns a recoverable refusal before invoking an operator.
pub trait UnaryOpT {
    const NAME: &'static str;
    const KERNEL: &'static str;
    const V: Self;
}

pub trait BinaryOpT {
    const NAME: &'static str;
    const KERNEL: &'static str;
    const V: Self;
}

macro_rules! unary_descriptor {
    ($op:ident, $name:literal) => {
        pub struct $op;
        impl UnaryOpT for $op {
            const NAME: &'static str = $name;
            const KERNEL: &'static str = concat!("u", $name);
            const V: Self = $op;
        }
    };
}
macro_rules! binary_descriptor {
    ($op:ident, $name:literal) => {
        pub struct $op;
        impl BinaryOpT for $op {
            const NAME: &'static str = $name;
            const KERNEL: &'static str = concat!("b", $name);
            const V: Self = $op;
        }
    };
}

binary_descriptor!(Add, "add");
binary_descriptor!(Sub, "sub");
binary_descriptor!(Mul, "mul");
binary_descriptor!(Div, "div");
binary_descriptor!(Maximum, "maximum");
binary_descriptor!(Minimum, "minimum");
unary_descriptor!(Exp, "exp");
unary_descriptor!(Log, "log");
unary_descriptor!(Sin, "sin");
unary_descriptor!(Cos, "cos");
unary_descriptor!(Abs, "abs");
unary_descriptor!(Neg, "neg");
unary_descriptor!(Recip, "recip");
unary_descriptor!(Sqr, "sqr");
unary_descriptor!(Sqrt, "sqrt");
unary_descriptor!(Gelu, "gelu");
unary_descriptor!(GeluErf, "gelu_erf");
unary_descriptor!(Erf, "erf");
unary_descriptor!(Relu, "relu");
unary_descriptor!(Silu, "silu");
unary_descriptor!(Tanh, "tanh");
unary_descriptor!(Floor, "floor");
unary_descriptor!(Ceil, "ceil");
unary_descriptor!(Round, "round");
unary_descriptor!(Sign, "sign");
#[derive(Clone)]
pub struct BackpropOp(Option<Op>);

impl BackpropOp {
    pub fn none() -> Self {
        BackpropOp(None)
    }

    pub(crate) fn new1(arg: &Tensor, f: impl Fn(Tensor) -> Op) -> Self {
        let op = if arg.track_op() {
            Some(f(arg.clone()))
        } else {
            None
        };
        Self(op)
    }

    pub(crate) fn new2(arg1: &Tensor, arg2: &Tensor, f: impl Fn(Tensor, Tensor) -> Op) -> Self {
        let op = if arg1.track_op() || arg2.track_op() {
            Some(f(arg1.clone(), arg2.clone()))
        } else {
            None
        };
        Self(op)
    }

    pub(crate) fn new3(
        arg1: &Tensor,
        arg2: &Tensor,
        arg3: &Tensor,
        f: impl Fn(Tensor, Tensor, Tensor) -> Op,
    ) -> Self {
        let op = if arg1.track_op() || arg2.track_op() || arg3.track_op() {
            Some(f(arg1.clone(), arg2.clone(), arg3.clone()))
        } else {
            None
        };
        Self(op)
    }

    pub(crate) fn new<A: AsRef<Tensor>>(args: &[A], f: impl Fn(Vec<Tensor>) -> Op) -> Self {
        let op = if args.iter().any(|arg| arg.as_ref().track_op()) {
            let args: Vec<Tensor> = args.iter().map(|arg| arg.as_ref().clone()).collect();
            Some(f(args))
        } else {
            None
        };
        Self(op)
    }

    pub(crate) fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl std::ops::Deref for BackpropOp {
    type Target = Option<Op>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
