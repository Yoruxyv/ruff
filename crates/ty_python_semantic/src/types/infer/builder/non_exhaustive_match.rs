use ruff_db::diagnostic::{Annotation, Span};
use ruff_db::parsed::parsed_module;
use ruff_db::source::source_text;
use ruff_diagnostics::{Edit, Fix};
use ruff_python_ast as ast;
use ruff_python_trivia::indentation_at_offset;
use ruff_source_file::{LineRanges, UniversalNewlineIterator, find_newline};
use ruff_text_size::{Ranged, TextRange};

use crate::diagnostic::format_enumeration;
use crate::types::class::{ClassLiteral, DynamicEnumLiteral};
use crate::types::display::DisplaySettings;
use crate::types::enums::enum_member_literals;
use crate::types::literal::LiteralValueTypeKind;
use crate::types::{EnumLiteralType, KnownClass, Type, diagnostic::NON_EXHAUSTIVE_MATCH};
use crate::{Db, ProgramEnvironment};

use super::TypeInferenceBuilder;

impl<'db> TypeInferenceBuilder<'db, '_> {
    pub(super) fn report_non_exhaustive_match(
        &self,
        match_statement: &ast::StmtMatch,
        subject_ty: Type<'db>,
        remaining: Type<'db>,
    ) {
        let db = self.db();
        let env = self.program_environment();
        let Some(builder) = self
            .context
            .report_lint(&NON_EXHAUSTIVE_MATCH, match_statement.subject.as_ref())
        else {
            return;
        };

        let missing = is_finite_match_subject(db, env, subject_ty)
            .then(|| finite_values(db, env, remaining))
            .flatten();
        let limit = if db.verbose() { usize::MAX } else { 3 };
        let message = if let Some(values) = missing.as_ref().filter(|values| !values.is_empty()) {
            let display_settings =
                DisplaySettings::from_possibly_ambiguous_types(db, env, values.iter().copied());
            let displayed: Vec<_> = values
                .iter()
                .take(limit)
                .map(|value| {
                    value
                        .display_literal_value_with(db, env, display_settings.clone())
                        .to_string()
                })
                .collect();
            let omitted = values.len() - displayed.len();
            let names = if omitted > 0 {
                format!(
                    "{} and {omitted} more",
                    displayed
                        .iter()
                        .map(|value| format!("`{value}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else if let [one] = displayed.as_slice() {
                format!("`{one}`")
            } else {
                format_enumeration(displayed.iter())
            };
            let (noun, verb) = match values.as_slice() {
                [value] if value.is_none(db) => ("", "is"),
                [_] => ("value ", "is"),
                _ => ("values ", "are"),
            };
            format!("Match is not exhaustive: {noun}{names} {verb} not covered")
        } else {
            format!(
                "Match is not exhaustive: objects of type `{}` are not covered",
                remaining.display(db, env)
            )
        };

        let mut diagnostic = builder.into_diagnostic(message.as_str());
        diagnostic.set_primary_annotation_message(format_args!(
            "Subject has type `{}`",
            subject_ty.display(db, env)
        ));

        if let Some(values) = missing {
            let single_enum_subject = match subject_ty.expand_top_level_aliases(db, env) {
                Type::NominalInstance(instance) => instance
                    .class_literal(db, env)
                    .into_enum_class(db)
                    .is_some(),
                Type::EnumComplement(_) => true,
                ty => ty.as_enum_literal().is_some(),
            };
            let display_settings =
                DisplaySettings::from_possibly_ambiguous_types(db, env, values.iter().copied());
            let mut dynamic_enums: Vec<(DynamicEnumLiteral<'db>, Vec<String>)> = Vec::new();
            for value in values.iter().take(limit) {
                if let Some(member) = value.as_enum_literal() {
                    let name = if single_enum_subject {
                        member.name(db).to_string()
                    } else {
                        value
                            .display_literal_value_with(db, env, display_settings.clone())
                            .to_string()
                    };
                    if let Some(definition) = member.definition(db) {
                        let module = parsed_module(db, definition.python_file(db)).load(db);
                        diagnostic.annotate(
                            Annotation::secondary(Span::from(definition.focus_range(db, &module)))
                                .message(format_args!("enum variant `{name}` is not covered")),
                        );
                    } else if let ClassLiteral::DynamicEnum(class) = member.enum_class(db) {
                        if let Some((_, names)) =
                            dynamic_enums.iter_mut().find(|(c, _)| *c == class)
                        {
                            names.push(name);
                        } else {
                            dynamic_enums.push((class, vec![name]));
                        }
                    }
                }
            }
            for (class, names) in dynamic_enums {
                let names_range = class.definition(db).and_then(|definition| {
                    let module = parsed_module(db, definition.python_file(db)).load(db);
                    definition
                        .kind(db)
                        .value(&module)
                        .and_then(ast::Expr::as_call_expr)
                        .and_then(|call| call.arguments.find_argument_value("names", 1))
                        .map(Ranged::range)
                });
                let span = Span::from(class.scope(db).file(db))
                    .with_range(names_range.unwrap_or_else(|| class.header_range(db)));
                let (label, names, verb) = if let [name] = names.as_slice() {
                    ("Enum variant", format!("`{name}`"), "is")
                } else {
                    ("Enum variants", format_enumeration(names.iter()), "are")
                };
                diagnostic.annotate(
                    Annotation::secondary(span)
                        .message(format_args!("{label} {names} {verb} not covered")),
                );
            }
            if values.len() > limit {
                diagnostic.info(format_args!(
                    "Use `--verbose` to see all {} uncovered values",
                    values.len()
                ));
            }
        }

        if contains_flag_instance(db, env, remaining) {
            diagnostic.info("`enum.Flag` can have unnamed combinations of members");
            diagnostic
                .info("See https://docs.python.org/3/howto/enum.html#combining-members-of-flag");
        }

        if let Some(fix) = self.non_exhaustive_match_fix(match_statement, remaining) {
            diagnostic.help("Add a `case` branch for the remaining values");
            diagnostic.set_fix(fix);
        }
    }

    fn non_exhaustive_match_fix(
        &self,
        match_statement: &ast::StmtMatch,
        remaining: Type<'db>,
    ) -> Option<Fix> {
        let db = self.db();
        let source = source_text(db, self.file());
        let last_case = match_statement.cases.last()?;
        let case_indent = indentation_at_offset(last_case.start(), &source)?;
        let body_indent = last_case
            .body
            .first()
            .and_then(|statement| indentation_at_offset(statement.start(), &source))
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{case_indent}{}", self.context.importer().indentation()));
        let line_ending = find_newline(&source)
            .map(|(_, ending)| ending)
            .unwrap_or_default()
            .as_str();
        let mut end = source.full_line_end(last_case.end());
        for line in UniversalNewlineIterator::with_offset(&source[usize::from(end)..], end) {
            if line.trim().is_empty() {
                continue;
            }
            if line.starts_with(&body_indent) && line.trim_start().starts_with('#') {
                end = line.full_end();
            } else {
                break;
            }
        }
        let leading_newline = if source.line_start(end) == end {
            ""
        } else {
            line_ending
        };
        let env = self.program_environment();
        let values = finite_values(db, env, remaining)
            .filter(|values| !values.is_empty() && values.len() <= 5);
        let pattern = values.map_or_else(
            || "_".to_owned(),
            |values| {
                values
                    .into_iter()
                    .map(|value| {
                        if let Some(member) = value.as_enum_literal()
                            && let Some(qualifier) = match_statement
                                .cases
                                .iter()
                                .find_map(|case| self.enum_pattern_qualifier(&case.pattern, member))
                        {
                            format!("{}.{}", &source[qualifier], member.name(db))
                        } else {
                            value.display_literal_value(db, env).to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" | ")
            },
        );
        Some(Fix::display_only_edit(Edit::insertion(
            format!(
                "{leading_newline}{case_indent}case {pattern}:{line_ending}\
                 {body_indent}raise NotImplementedError(\"TODO\"){line_ending}"
            ),
            end,
        )))
    }

    fn enum_pattern_qualifier(
        &self,
        pattern: &ast::Pattern,
        member: EnumLiteralType<'db>,
    ) -> Option<TextRange> {
        match pattern {
            ast::Pattern::MatchValue(value)
                if let ast::Expr::Attribute(attribute) = value.value.as_ref()
                    && self
                        .try_expression_type(&value.value)
                        .and_then(Type::as_enum_literal)
                        .is_some_and(|other| {
                            other.enum_class_literal(self.db())
                                == member.enum_class_literal(self.db())
                        }) =>
            {
                Some(attribute.value.range())
            }
            ast::Pattern::MatchOr(or) => or
                .patterns
                .iter()
                .find_map(|pattern| self.enum_pattern_qualifier(pattern, member)),
            ast::Pattern::MatchAs(as_pattern) => as_pattern
                .pattern
                .as_deref()
                .and_then(|pattern| self.enum_pattern_qualifier(pattern, member)),
            _ => None,
        }
    }
}

fn is_literal_value<'db>(db: &'db dyn Db, ty: Type<'db>) -> bool {
    if ty.is_none(db) {
        return true;
    }
    let Some(kind) = ty.as_literal_value_kind() else {
        return false;
    };
    match kind {
        LiteralValueTypeKind::Int(_)
        | LiteralValueTypeKind::Bool(_)
        | LiteralValueTypeKind::String(_)
        | LiteralValueTypeKind::Bytes(_)
        | LiteralValueTypeKind::Enum(_) => true,
        LiteralValueTypeKind::LiteralString => false,
    }
}

fn is_finite_match_subject<'db>(
    db: &'db dyn Db,
    env: &ProgramEnvironment<'db>,
    ty: Type<'db>,
) -> bool {
    let ty = ty.expand_top_level_aliases(db, env);
    if is_literal_value(db, ty) {
        return true;
    }
    match ty {
        Type::Union(union) => union
            .elements(db)
            .iter()
            .all(|element| is_finite_match_subject(db, env, *element)),
        Type::NominalInstance(instance) => instance
            .class_literal(db, env)
            .into_enum_class(db)
            .is_some_and(|class| class.members_are_exhaustive(db)),
        Type::EnumComplement(_) => true,
        _ => false,
    }
}

fn contains_flag_instance<'db>(
    db: &'db dyn Db,
    env: &ProgramEnvironment<'db>,
    ty: Type<'db>,
) -> bool {
    let ty = ty.expand_top_level_aliases(db, env);
    match ty {
        Type::Union(union) => union
            .elements(db)
            .iter()
            .any(|element| contains_flag_instance(db, env, *element)),
        _ => {
            !is_literal_value(db, ty)
                && ty.is_subtype_of(db, env, KnownClass::Flag.to_instance(db, env))
        }
    }
}

fn finite_values<'db>(
    db: &'db dyn Db,
    env: &ProgramEnvironment<'db>,
    ty: Type<'db>,
) -> Option<Vec<Type<'db>>> {
    let ty = ty.expand_top_level_aliases(db, env);
    if is_literal_value(db, ty) {
        return Some(vec![ty]);
    }
    match ty {
        Type::Union(union) => {
            let mut values = Vec::new();
            for element in union.elements(db) {
                values.extend(finite_values(db, env, *element)?);
            }
            Some(values)
        }
        Type::NominalInstance(instance) => {
            Some(enum_member_literals(db, instance.class_literal(db, env), None)?.collect())
        }
        Type::EnumComplement(complement) => {
            let values = complement.remaining_literal_types(db, env);
            values
                .iter()
                .all(|value| is_literal_value(db, *value))
                .then_some(values)
        }
        _ => None,
    }
}
