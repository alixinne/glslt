use std::collections::HashMap;

use glsl::syntax::*;
use glsl::visitor::*;

use lazy_static::lazy_static;

use crate::{Error, Result};

use super::{template::TemplateDefinition, TransformUnit};

lazy_static! {
    // Keep this sorted
    #[rustfmt::skip]
    static ref BUILTIN_FUNCTION_NAMES: &'static [&'static str] = &[
        "EmitStreamVertex", "EmitVertex", "EndPrimitive", "EndStreamPrimitive", "abs", "acos",
        "acosh", "all", "any", "asin", "asinh", "atan", "atanh", "atomicAdd", "atomicAnd",
        "atomicCompSwap", "atomicCounter", "atomicCounterDecrement", "atomicCounterIncrement",
        "atomicExchange", "atomicMax", "atomicMin", "atomicOr", "atomicXor", "barrier", "bitCount",
        "bitfieldExtract", "bitfieldInsert", "bitfieldReverse", "ceil", "clamp", "cos", "cosh",
        "cross", "dFdx", "dFdxCoarse", "dFdxFine", "dFdy", "dFdyCoarse", "dFdyFine", "degrees",
        "determinant", "distance", "dot", "equal", "exp", "exp2", "faceforward", "findLSB",
        "findMSB", "float", "floatBitsToInt", "floatBitsToUint", "floor", "fma", "fract", "frexp",
        "fwidth", "fwidthCoarse", "fwidthFine", "greaterThan", "greaterThanEqual",
        "groupMemoryBarrier", "imageAtomicAdd", "imageAtomicAnd", "imageAtomicCompSwap",
        "imageAtomicExchange", "imageAtomicMax", "imageAtomicMin", "imageAtomicOr",
        "imageAtomicXor", "imageLoad", "imageSamples", "imageSize", "imageStore", "imulExtended",
        "int", "intBitsToFloat", "interpolateAtCentroid", "interpolateAtOffset",
        "interpolateAtSample", "inverse", "inversesqrt", "isinf", "isnan", "ivec2", "ivec3",
        "ivec4", "ldexp", "length", "lessThan", "lessThanEqual", "log", "log2", "mat2", "mat3",
        "mat4", "matrixCompMult", "max", "memoryBarrier", "memoryBarrierAtomicCounter",
        "memoryBarrierBuffer", "memoryBarrierImage", "memoryBarrierShared", "min", "mix", "mod",
        "modf", "noise", "noise1", "noise2", "noise3", "noise4", "normalize", "not", "notEqual",
        "outerProduct", "packDouble2x32", "packHalf2x16", "packSnorm2x16", "packSnorm4x8",
        "packUnorm", "packUnorm2x16", "packUnorm4x8", "pow", "radians", "reflect", "refract",
        "removedTypes", "round", "roundEven", "sign", "sin", "sinh", "smoothstep", "sqrt", "step",
        "tan", "tanh", "texelFetch", "texelFetchOffset", "texture", "textureGather",
        "textureGatherOffset", "textureGatherOffsets", "textureGrad", "textureGradOffset",
        "textureLod", "textureLodOffset", "textureOffset", "textureProj", "textureProjGrad",
        "textureProjGradOffset", "textureProjLod", "textureProjLodOffset", "textureProjOffset",
        "textureQueryLevels", "textureQueryLod", "textureSamples", "textureSize", "transpose",
        "trunc", "uaddCarry", "uint", "uintBitsToFloat", "umulExtended", "unpackDouble2x32",
        "unpackHalf2x16", "unpackSnorm2x16", "unpackSnorm4x8", "unpackUnorm", "unpackUnorm2x16",
        "unpackUnorm4x8", "usubBorrow", "uvec2", "uvec3", "uvec4", "vec2", "vec3", "vec4",
    ];
}

#[derive(Debug, Clone)]
pub struct DeclaredSymbol {
    pub symbol_id: usize,
    pub gen_id: Node<Identifier>,
    pub decl_type: TypeSpecifier,
    pub array: Option<ArraySpecifier>,
}

pub struct InstantiateTemplate {
    error: Option<Error>,
    symbol_table: HashMap<String, DeclaredSymbol>,
}

impl InstantiateTemplate {
    pub fn new() -> Self {
        Self {
            error: None,
            symbol_table: HashMap::new(),
        }
    }

    pub fn instantiate(
        mut self,
        unit: &mut dyn TransformUnit,
        mut def: Node<FunctionDefinition>,
    ) -> Result<()> {
        // Transform definition. The visitor is responsible for instantiating templates
        let mut tgt = InstantiateTemplateUnit {
            instantiator: &mut self,
            unit,
        };

        def.visit(&mut tgt);

        if let Some(error) = self.error.take() {
            return Err(error);
        }

        unit.push_function_declaration(def);

        Ok(())
    }

    fn new_gen_id(&self) -> Node<Identifier> {
        Node::new(
            Identifier::new(format!("_glslt_lp{}", self.symbol_table.len())).unwrap(),
            None,
        )
    }

    pub(in crate::transform) fn visit_fun_call(
        &mut self,
        fun: &mut FunIdentifier,
        args: &mut Vec<Expr>,
        unit: &mut dyn TransformUnit,
    ) {
        // First visit the arguments to transform inner lambdas first
        for arg in args.iter_mut() {
            arg.visit(&mut InstantiateTemplateUnit {
                instantiator: self,
                unit,
            });
        }

        // Only consider raw identifiers for function names
        if let FunIdentifier::Identifier(ident) = fun {
            if BUILTIN_FUNCTION_NAMES
                .binary_search(&ident.0.as_str())
                .is_err()
            {
                if let Some(template) = unit.ctx().get_template(&ident.0) {
                    if let Err(error) = self.transform_call(&*template, ident, args, unit) {
                        self.error = Some(error);
                    }
                } else {
                    debug!("no template for function call: {}", ident.0);
                }
            }
        }
    }

    fn transform_call(
        &mut self,
        template: &TemplateDefinition,
        fun: &mut Identifier,
        args: &mut Vec<Expr>,
        unit: &mut dyn TransformUnit,
    ) -> Result<()> {
        debug!("found template function call: {}({:?})", fun.0, args);

        // We found a template whose name matches the identifier
        // Thus, transform the function call

        // Extract arguments
        let mut template_parameters = template.extract_template_parameters(args)?;

        // Generate name
        let template_name = template.generate_id(&template_parameters);

        // Extract the set of captured variables
        struct Capturer<'ds> {
            st: &'ds HashMap<String, super::instantiate::DeclaredSymbol>,
            captured: HashMap<String, super::instantiate::DeclaredSymbol>,
        }

        impl Visitor for Capturer<'_> {
            fn visit_expr(&mut self, e: &mut Expr) -> Visit {
                if let Expr::Variable(ident) = e {
                    // This is a variable. If it's in the symbol table, it needs to be
                    // captured and renamed

                    if let Some(sb) = self.st.get(ident.0.as_str()) {
                        // Add it to the list of captured variables
                        self.captured.insert(ident.0.clone(), (*sb).clone());
                        // Rename the reference
                        *ident = sb.gen_id.clone();
                    }
                }

                Visit::Children
            }
        }

        // Visit the input expressions
        let mut capturer = Capturer {
            st: &self.symbol_table,
            captured: HashMap::new(),
        };

        for tp in &mut template_parameters {
            tp.visit(&mut capturer);
        }

        // Extract the list of captured variables ordered by symbol_id
        let mut extra_parameters: Vec<_> = capturer.captured.into_iter().collect();
        extra_parameters.sort_by_key(|ep| ep.1.symbol_id);

        // Instantiate the template if needed
        if !unit.template_instance_declared(&template_name) {
            let template = template.instantiate(
                &template_name,
                &template_parameters,
                &extra_parameters,
                self,
                unit,
            );

            unit.register_template_instance(&template_name, template);
        }

        // The identifier should be replaced by the mangled name
        fun.0 = template_name;

        // Add the captured parameters to the end of the call
        for ep in extra_parameters.into_iter() {
            // TODO: Preserve span information
            args.push(Expr::Variable(Node::new(
                Identifier::new(ep.0).unwrap(),
                None,
            )));
        }
        Ok(())
    }
}

struct InstantiateTemplateUnit<'c> {
    instantiator: &'c mut InstantiateTemplate,
    unit: &'c mut dyn TransformUnit,
}

impl Visitor for InstantiateTemplateUnit<'_> {
    fn visit_function_parameter_declarator(
        &mut self,
        p: &mut FunctionParameterDeclarator,
    ) -> Visit {
        // Register a declared parameter
        self.instantiator.symbol_table.insert(
            p.ident.ident.0.clone(),
            DeclaredSymbol {
                symbol_id: self.instantiator.symbol_table.len(),
                gen_id: self.instantiator.new_gen_id(),
                decl_type: p.ty.clone(),
                array: p.ident.array_spec.clone(),
            },
        );

        Visit::Children
    }

    fn visit_init_declarator_list(&mut self, idl: &mut InitDeclaratorList) -> Visit {
        // Register all declared variables
        self.instantiator.symbol_table.insert(
            idl.head.name.as_ref().unwrap().0.clone(),
            DeclaredSymbol {
                symbol_id: self.instantiator.symbol_table.len(),
                gen_id: self.instantiator.new_gen_id(),
                decl_type: idl.head.ty.ty.clone(),
                array: idl.head.array_specifier.clone(),
            },
        );

        // Add tail
        for t in &idl.tail {
            self.instantiator.symbol_table.insert(
                t.ident.ident.0.clone(),
                DeclaredSymbol {
                    symbol_id: self.instantiator.symbol_table.len(),
                    gen_id: self.instantiator.new_gen_id(),
                    decl_type: idl.head.ty.ty.clone(),
                    array: idl.head.array_specifier.clone(),
                },
            );
        }

        Visit::Children
    }

    fn visit_expr(&mut self, e: &mut Expr) -> Visit {
        if let Expr::FunCall(fun, args) = e {
            self.instantiator.visit_fun_call(fun, args, self.unit);

            // We already visited arguments in pre-order
            return Visit::Parent;
        }

        Visit::Children
    }
}
