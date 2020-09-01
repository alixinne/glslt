use std::collections::HashMap;

use glsl::syntax::*;
use glsl::visitor::*;

use lazy_static::lazy_static;

use crate::{Error, Result};

use super::{template::TemplateDefinition, Scope};

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
    pub gen_id: Identifier,
    pub decl_type: TypeSpecifier,
    pub array: Option<ArraySpecifier>,
}

#[derive(Default)]
pub struct InstantiateTemplate {
    error: Option<Error>,
    symbol_table: HashMap<String, DeclaredSymbol>,
}

impl InstantiateTemplate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn instantiate(
        mut self,
        scope: &mut dyn Scope,
        mut def: FunctionDefinition,
    ) -> Result<Vec<FunctionDefinition>> {
        // Transform definition. The visitor is responsible for instantiating templates
        let mut tgt = InstantiateTemplateUnit {
            instantiator: &mut self,
            scope,
        };

        def.visit(&mut tgt);

        // Push new function declarations
        let mut res = tgt.scope.take_instanced_templates();
        res.push(def);

        if let Some(error) = self.error.take() {
            Err(error)
        } else {
            Ok(res)
        }
    }

    pub fn get_symbol(&self, name: &str) -> Option<&DeclaredSymbol> {
        self.symbol_table.get(name)
    }

    fn new_gen_id(&self) -> Identifier {
        IdentifierData(format!("{}_lp{}", crate::PREFIX, self.symbol_table.len())).into()
    }

    pub(in crate::transform) fn visit_fun_call<'s>(
        &mut self,
        expr: &mut Expr,
        scope: &'s mut dyn Scope,
    ) {
        match expr {
            Expr::FunCall(fun, args) => {
                // First visit the arguments to transform inner lambdas first
                for arg in args.iter_mut() {
                    arg.visit(&mut InstantiateTemplateUnit {
                        instantiator: self,
                        scope,
                    });
                }

                // Only consider raw identifiers for function names
                if let FunIdentifier::Identifier(ident) = fun {
                    if BUILTIN_FUNCTION_NAMES
                        .binary_search(&ident.0.as_str())
                        .is_err()
                    {
                        // Look up arguments first
                        match scope.transform_arg_call(expr, self) {
                            Ok(()) => {}
                            Err(Error::TransformAsTemplate) => {
                                if let Expr::FunCall(FunIdentifier::Identifier(ident), args) = expr
                                {
                                    if let Some(template) = scope.get_template(&ident.0) {
                                        if let Err(error) =
                                            self.transform_call(&*template, ident, args, scope)
                                        {
                                            self.error = Some(error);
                                        }
                                    } else {
                                        debug!("no template for function call: {}", ident.0);
                                    }
                                }
                            }
                            Err(error) => {
                                self.error = Some(error);
                            }
                        }
                    }
                }
            }
            other => panic!(
                "expected Expr::FunCall in InstantiateTemplate::visit_fun_call, got {:?}",
                other
            ),
        }
    }

    fn transform_call<'s>(
        &mut self,
        template: &TemplateDefinition,
        fun: &mut Identifier,
        args: &mut Vec<Expr>,
        scope: &'s mut dyn Scope,
    ) -> Result<()> {
        debug!("found template function call: {}({:?})", fun.0, args);

        // We found a template whose name matches the identifier
        // Thus, transform the function call

        // Create the local scope
        let mut local_scope = super::LocalScope::new(template, args, &self.symbol_table, scope)?;
        trace!("symbol table: {:?}", self.symbol_table);
        trace!("entering local scope: {:#?}", local_scope);

        // Instantiate the template if needed
        if !local_scope.template_instance_declared(&local_scope.name()) {
            let template = template.instantiate(&mut local_scope, self)?;
            local_scope.register_template_instance(template);
        }

        // The identifier should be replaced by the mangled name
        fun.0 = local_scope.name().to_owned();

        // Add the captured parameters to the end of the call
        for ep in local_scope.captured_parameters().iter() {
            // TODO: Preserve span information
            args.push(Expr::Variable(IdentifierData(ep.clone()).into()));
        }

        Ok(())
    }

    fn add_declared_symbol(
        &mut self,
        scope: &dyn Scope,
        name: String,
        decl_type: TypeSpecifier,
        array: Option<ArraySpecifier>,
    ) {
        if let TypeSpecifierNonArray::TypeName(tn) = &decl_type.ty {
            if scope.declared_pointer_types().contains_key(tn.0.as_str()) {
                // This is a template function argument, do not register it for capture
                return;
            }
        }

        self.symbol_table.insert(
            name,
            DeclaredSymbol {
                symbol_id: self.symbol_table.len(),
                gen_id: self.new_gen_id(),
                decl_type,
                array,
            },
        );
    }
}

struct InstantiateTemplateUnit<'c> {
    instantiator: &'c mut InstantiateTemplate,
    scope: &'c mut dyn Scope,
}

impl Visitor for InstantiateTemplateUnit<'_> {
    fn visit_function_parameter_declarator(
        &mut self,
        p: &mut FunctionParameterDeclarator,
    ) -> Visit {
        // Register a declared parameter
        self.instantiator.add_declared_symbol(
            self.scope,
            p.ident.ident.0.clone(),
            p.ty.clone(),
            p.ident.array_spec.clone(),
        );

        Visit::Children
    }

    fn visit_init_declarator_list(&mut self, idl: &mut InitDeclaratorList) -> Visit {
        // Register all declared variables
        self.instantiator.add_declared_symbol(
            self.scope,
            idl.head.name.as_ref().unwrap().0.clone(),
            idl.head.ty.ty.clone(),
            idl.head.array_specifier.clone(),
        );

        // Add tail
        for t in &idl.tail {
            self.instantiator.add_declared_symbol(
                self.scope,
                t.ident.ident.0.clone(),
                idl.head.ty.ty.clone(),
                idl.head.array_specifier.clone(),
            );
        }

        Visit::Children
    }

    fn visit_expr(&mut self, e: &mut Expr) -> Visit {
        if let Expr::FunCall(_, _) = e {
            self.instantiator.visit_fun_call(e, self.scope);

            // We already visited arguments in pre-order
            return Visit::Parent;
        }

        Visit::Children
    }
}
