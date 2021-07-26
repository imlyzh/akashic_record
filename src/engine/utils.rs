use lazy_static::*;
use sexpr_ir::syntax::sexpr::one_unit_parse;
use sexpr_process::pattern::Pattern;

macro_rules! impl_pattern {
    ($name:ident, $e:expr) => {
        lazy_static! {
            pub static ref $name: Pattern =
                Pattern::from(&one_unit_parse($e, "<akashic_record>").unwrap()).unwrap();
        }
    };
}


impl_pattern!(FACT_PATTERN, "('fact name expr ...)");

impl_pattern!(RULE_PATTERN, "('rule prarms expr ...)");

impl_pattern!(RULE_PARAMS_PATTERN, "(name args ...)");

impl_pattern!(FUNCTION_CALL_PATTERN, "(name args ...)");

impl_pattern!(QUERY_PATTERN, "('define prarms expr ...)");

impl_pattern!(QUERY_VAR_PATTERN, "(prarms ...)");

impl_pattern!(FACT_QUERY_PATTERN, "(name args ...)");

impl_pattern!(TUPLE_PATTERN_PATTERN, "('tuple args ...)");

impl_pattern!(LIST_HAS_EXTEND_PATTERN_PATTERN, "('list args ... . extend)");
impl_pattern!(LIST_PATTERN_PATTERN, "('list args ... . extends)");

impl_pattern!(SYMBOL_LITERIAL_PATTERN, "('quote sym)");