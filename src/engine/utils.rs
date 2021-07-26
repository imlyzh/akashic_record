use lazy_static::*;
use sexpr_ir::syntax::sexpr::one_unit_parse;
use sexpr_process::pattern::ListPattern;

macro_rules! impl_pattern {
    ($name:ident, $e:expr) => {
        lazy_static! {
            pub static ref $name: ListPattern =
                ListPattern::from(&one_unit_parse($e, "<akashic_record>").unwrap()).unwrap();
        }
    };
}


impl_pattern!(DEFINE_PATTERN, "('define name expr)");

impl_pattern!(FACT_PATTERN, "('fact name exprs ...)");

impl_pattern!(RULE_PATTERN, "('rule prarms exprs ...)");

impl_pattern!(QUERY_PATTERN, "('exists prarms 'forall exprs ...)");


impl_pattern!(RULE_PARAMS_PATTERN, "(name args ...)");

impl_pattern!(FUNCTION_CALL_PATTERN, "(name args ...)");

impl_pattern!(QUERY_VAR_PATTERN, "(prarms ...)");

impl_pattern!(FACT_QUERY_PATTERN, "(name args ...)");

impl_pattern!(SYMBOL_LITERIAL_PATTERN, "('quote sym)");

impl_pattern!(TUPLE_PATTERN_PATTERN, "('tuple args ...)");

impl_pattern!(LIST_HAS_EXTEND_PATTERN_PATTERN, "('list args ... . extend)");
impl_pattern!(LIST_PATTERN_PATTERN, "('list args ... . extends)");
