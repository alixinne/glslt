//! GLSLT utilities

// Keep this sorted
#[rustfmt::skip]
static BUILTIN_FUNCTION_NAMES: &[&str] = &[
    "EmitStreamVertex", "EmitVertex", "EndPrimitive", "EndStreamPrimitive", "abs", "acos",
    "acosh", "all", "any", "asin", "asinh", "atan", "atanh", "atomicAdd", "atomicAnd",
    "atomicCompSwap", "atomicCounter", "atomicCounterDecrement", "atomicCounterIncrement",
    "atomicExchange", "atomicMax", "atomicMin", "atomicOr", "atomicXor", "barrier", "bitCount",
    "bitfieldExtract", "bitfieldInsert", "bitfieldReverse", "ceil", "clamp", "cos", "cosh",
    "cross", "dFdx", "dFdxCoarse", "dFdxFine", "dFdy", "dFdyCoarse", "dFdyFine", "degrees",
    "determinant", "distance", "dot", "equal", "exp", "exp2", "faceforward", "findLSB",
    "findMSB", "floatBitsToInt", "floatBitsToUint", "floor", "fma", "fract", "frexp", "fwidth",
    "fwidthCoarse", "fwidthFine", "greaterThan", "greaterThanEqual", "groupMemoryBarrier",
    "imageAtomicAdd", "imageAtomicAnd", "imageAtomicCompSwap", "imageAtomicExchange",
    "imageAtomicMax", "imageAtomicMin", "imageAtomicOr", "imageAtomicXor", "imageLoad",
    "imageSamples", "imageSize", "imageStore", "imulExtended", "intBitsToFloat",
    "interpolateAtCentroid", "interpolateAtOffset", "interpolateAtSample", "inverse",
    "inversesqrt", "isinf", "isnan", "ldexp", "length", "lessThan", "lessThanEqual", "log",
    "log2", "matrixCompMult", "max", "memoryBarrier", "memoryBarrierAtomicCounter",
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
    "trunc", "uaddCarry", "uintBitsToFloat", "umulExtended", "unpackDouble2x32",
    "unpackHalf2x16", "unpackSnorm2x16", "unpackSnorm4x8", "unpackUnorm", "unpackUnorm2x16",
    "unpackUnorm4x8", "usubBorrow",
];

/// Return `true` if `name` is the name of a built-in GLSL function
pub fn is_builtin_glsl_function(name: &str) -> bool {
    BUILTIN_FUNCTION_NAMES.binary_search(&name).is_ok()
}
