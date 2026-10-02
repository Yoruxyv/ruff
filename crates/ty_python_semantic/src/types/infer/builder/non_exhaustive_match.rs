//! Diagnostics and display-only fixes for non-exhaustive `match` statements.

use std::borrow::Cow;

use itertools::{Either, Itertools};
use ruff_db::diagnostic::{Annotation, Span};
use ruff_db::parsed::parsed_module;
use ruff_db::source::source_text;
use ruff_diagnostics::{Edit, Fix};
use ruff_python_ast as ast;
use ruff_python_trivia::indentation_at_offset;
use ruff_source_file::{LineRanges, UniversalNewlineIterator, find_newline};
use ruff_text_size::{Ranged, TextRange};
use smallvec::SmallVec;

use crate::diagnostic::format_enumeration;
use crate::types::class::{ClassLiteral, DynamicEnumLiteral};
use crate::types::display::DisplaySettings;
use crate::types::enums::enum_member_literals;
use crate::types::equality::is_same_enum_domain;
use crate::types::literal::LiteralValueTypeKind;
use crate::types::{EnumLiteralType, KnownClass, Type, diagnostic::NON_EXHAUSTIVE_MATCH};
use crate::{Db, FxIndexMap, ProgramEnvironment};

use super::TypeInferenceBuilder;

impl<'db> TypeInferenceBuilder<'db, '_> {
    /// Report values left uncovered by a `match` statement.
    ///
    /// `subject_type` is the type at the start of the match; `remaining` is the type left after
    /// all cases have been considered. The diagnostic identifies individual missing values when
    /// possible and may provide a display-only fix.
    pub(super) fn report_non_exhaustive_match(
        &self,
        match_statement: &ast::StmtMatch,
        subject_type: Type<'db>,
        remaining: Type<'db>,
    ) {
        let db = self.db();
        let env = self.program_environment();

        let Some(builder) = self
            .context
            .report_lint(&NON_EXHAUSTIVE_MATCH, &*match_statement.subject)
        else {
            return;
        };

        let missing = is_finite_match_subject(db, env, subject_type)
            .then(|| finite_values(db, env, remaining))
            .flatten();

        let limit = if db.verbose() { usize::MAX } else { 3 };

        let message = if let Some(missing_values) = missing.as_ref()
            && !missing_values.is_empty()
        {
            let display_settings =
                DisplaySettings::from_possibly_ambiguous_types(db, env, missing_values);

            let displayed: Vec<_> = missing_values
                .iter()
                .take(limit)
                .map(|value| value.display_literal_value_with(db, env, display_settings.clone()))
                .collect();

            let omitted = missing_values.len() - displayed.len();

            let names = if omitted > 0 {
                format!(
                    "{} and {omitted} more",
                    displayed
                        .iter()
                        .map(|value| format!("`{value}`"))
                        .join(", ")
                )
            } else if let [one] = displayed.as_slice() {
                format!("`{one}`")
            } else {
                format_enumeration(&displayed)
            };

            let (noun, verb) = match missing_values.as_slice() {
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

        let mut diagnostic = builder.into_diagnostic(&message);

        let subject_type_display = subject_type.display(db, env);

        diagnostic.set_primary_annotation_message(format_args!(
            "Subject has type `{subject_type_display}`",
        ));

        if subject_type.is_dynamic() {
            diagnostic.set_concise_message(format_args!(
                "Match is not exhaustive: subject has type `{subject_type_display}`",
            ));
        } else {
            diagnostic.set_concise_message(message);
        }

        if let Some(missing_values) = missing {
            // If the subject is confined to one enum, its annotations need only member names.
            let is_single_enum_subject = missing_values
                .iter()
                .find_map(|value| value.as_enum_literal())
                .is_some_and(|member| is_same_enum_domain(db, env, subject_type, member));

            let display_settings =
                DisplaySettings::from_possibly_ambiguous_types(db, env, &missing_values);

            // Functional enum members share a definition instead of having individual ones.
            // Group their missing names by enum so the second loop can annotate each shared
            // definition once with all of its displayed missing members. Preserve the order in
            // which enums are encountered so their related locations in editor diagnostics remain
            // stable.
            let mut dynamic_enums: FxIndexMap<DynamicEnumLiteral<'_>, SmallVec<[_; 1]>> =
                FxIndexMap::default();

            for value in missing_values.iter().take(limit) {
                let Some(member) = value.as_enum_literal() else {
                    continue;
                };

                let name = if is_single_enum_subject {
                    Either::Left(member.name(db))
                } else {
                    let display =
                        value.display_literal_value_with(db, env, display_settings.clone());
                    Either::Right(display)
                };

                if let Some(definition) = member.definition(db) {
                    let module = parsed_module(db, definition.python_file(db)).load(db);
                    diagnostic.annotate(
                        Annotation::secondary(Span::from(definition.focus_range(db, &module)))
                            .message(format_args!("enum variant `{name}` is not covered")),
                    );
                } else if let ClassLiteral::DynamicEnum(class) = member.enum_class(db) {
                    dynamic_enums.entry(class).or_default().push(name);
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
                    ("Enum variants", format_enumeration(&names), "are")
                };

                diagnostic.annotate(
                    Annotation::secondary(span)
                        .message(format_args!("{label} {names} {verb} not covered")),
                );
            }

            if missing_values.len() > limit {
                diagnostic.info(format_args!(
                    "Use `--verbose` to see all {} uncovered values",
                    missing_values.len()
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

    /// Suggest a display-only case for the remaining values at the end of the match.
    ///
    /// When all remaining values can be enumerated and there are one to five of them, list them as
    /// alternatives; otherwise use a wildcard. The case body raises `NotImplementedError` as a
    /// placeholder.
    fn non_exhaustive_match_fix(
        &self,
        match_statement: &ast::StmtMatch,
        remaining: Type<'db>,
    ) -> Option<Fix> {
        let db = self.db();
        let env = self.program_environment();
        let source = source_text(db, self.file());
        let last_case = match_statement.cases.last()?;
        let case_indent = indentation_at_offset(last_case.start(), &source)?;

        let body_indent = last_case
            .body
            .first()
            .and_then(|statement| indentation_at_offset(statement.start(), &source))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| {
                Cow::Owned(format!(
                    "{case_indent}{}",
                    self.context.importer().indentation()
                ))
            });

        let line_ending = find_newline(&source)
            .map(|(_, ending)| ending)
            .unwrap_or_default()
            .as_str();

        let mut end = source.full_line_end(last_case.end());

        for line in UniversalNewlineIterator::with_offset(&source[usize::from(end)..], end) {
            if line.trim().is_empty() {
                continue;
            }
            if line.starts_with(&*body_indent) && line.trim_start().starts_with('#') {
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

        let pattern = finite_values(db, env, remaining)
            .filter(|values| !values.is_empty() && values.len() <= 5)
            .map(|values| {
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
                    .join(" | ")
            })
            .unwrap_or_else(|| "_".to_string());

        let insertion = format!(
            "{leading_newline}{case_indent}case {pattern}:{line_ending}\
            {body_indent}raise NotImplementedError(\"TODO\"){line_ending}"
        );

        Some(Fix::display_only_edit(Edit::insertion(insertion, end)))
    }

    /// Find the source range of an enum reference in a pattern for the same enum as `member`.
    ///
    /// This searches value patterns and the alternatives inside `|` and `as` patterns. The
    /// returned range excludes the member name, so the reference can be reused for another member
    /// in a suggested case. It does not check whether the reference will still resolve to that
    /// enum when the suggested case runs.
    fn enum_pattern_qualifier(
        &self,
        pattern: &ast::Pattern,
        member: EnumLiteralType<'db>,
    ) -> Option<TextRange> {
        let db = self.db();

        match pattern {
            ast::Pattern::MatchValue(value)
                if let ast::Expr::Attribute(attribute) = &*value.value
                    && let Some(expression_type) = self.try_expression_type(&value.value)
                    && let Some(enum_literal) = expression_type.as_enum_literal()
                    && enum_literal.enum_class_literal(db) == member.enum_class_literal(db) =>
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

/// Return whether `ty` represents a single value usable in a match pattern.
///
/// This includes `None` and enum literals, but excludes `LiteralString`, which describes many
/// possible strings.
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

/// Return whether the subject is recognized as finite by the non-exhaustive-match diagnostic.
///
/// This determines whether the diagnostic attempts to list individual uncovered values. Literal
/// values, unions of recognized finite types, exhaustive enum instances, and enum complements are
/// recognized. A complement can retain other positive intersection components that prevent its
/// remaining values from being enumerated by [`finite_values`].
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

/// Return whether `ty`, or an element of a union in `ty`, is a non-literal `enum.Flag` subtype.
fn contains_flag_instance<'db>(
    db: &'db dyn Db,
    env: &ProgramEnvironment<'db>,
    ty: Type<'db>,
) -> bool {
    match ty.expand_top_level_aliases(db, env) {
        Type::Union(union) => union
            .elements(db)
            .iter()
            .any(|element| contains_flag_instance(db, env, *element)),

        ty => {
            !is_literal_value(db, ty)
                && ty.is_subtype_of(db, env, KnownClass::Flag.to_instance(db, env))
        }
    }
}

/// Enumerate all literal values represented by `ty` when its type shape is supported.
///
/// This expands unions, exhaustive enum instances, and enum complements as well as single
/// literal values. It returns `None` for unsupported shapes and for enum complements whose
/// remaining members, after applying additional intersection constraints, do not all have literal
/// types.
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
            let elements = union.elements(db);
            let mut values = Vec::with_capacity(elements.len());
            for element in elements {
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
