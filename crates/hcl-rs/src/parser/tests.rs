use super::*;
use pest::*;

#[test]
fn parse_identifier() {
    parses_to! {
        parser: HclParser,
        input: "_an-id3nt1fieR",
        rule: Rule::Identifier,
        tokens: [
            Identifier(0, 14)
        ]
    };
}

#[test]
fn parse_string() {
    parses_to! {
        parser: HclParser,
        input: "\"a string\"",
        rule: Rule::StringLit,
        tokens: [
            StringLit(0, 10, [
                String(1, 9)
            ])
        ]
    };
}

#[test]
fn parse_number() {
    parses_to! {
        parser: HclParser,
        input: "12e+10",
        rule: Rule::NumericLit,
        tokens: [
            Float(0, 6)
        ]
    };

    parses_to! {
        parser: HclParser,
        input: "42",
        rule: Rule::NumericLit,
        tokens: [
            Int(0, 2)
        ]
    };
}

#[test]
fn parse_conditional() {
    parses_to! {
        parser: HclParser,
        input: "var.enabled ? 1 : 0",
        rule: Rule::Expression,
        tokens: [
            Expression(0, 19, [
                ExprTerm(0, 11, [
                    Variable(0, 3),
                    GetAttr(3, 11, [
                        Identifier(4, 11)
                    ])
                ]),
                Expression(14, 16, [
                    ExprTerm(14, 16, [
                        Int(14, 15)
                    ])
                ]),
                Expression(18, 19, [
                    ExprTerm(18, 19, [
                        Int(18, 19)
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parse_collections() {
    parses_to! {
        parser: HclParser,
        input: r#"foo = ["bar", ["baz"]]"#,
        rule: Rule::Attribute,
        tokens: [
            Attribute(0, 22, [
                Identifier(0, 3),
                Expression(6, 22, [
                    ExprTerm(6, 22, [
                        Tuple(6, 22, [
                            Expression(7, 12, [
                                ExprTerm(7, 12, [
                                    StringLit(7, 12, [
                                        String(8, 11)
                                    ])
                                ])
                            ]),
                            Expression(14, 21, [
                                ExprTerm(14, 21, [
                                    Tuple(14, 21, [
                                        Expression(15, 20, [
                                            ExprTerm(15, 20, [
                                                StringLit(15, 20, [
                                                    String(16, 19)
                                                ])
                                            ])
                                        ])
                                    ])
                                ])
                            ])
                        ])
                    ])
                ])
            ])
        ]
    };

    parses_to! {
        parser: HclParser,
        input: r#"foo = {"bar" = "baz","qux" = ident }"#,
        rule: Rule::Attribute,
        tokens: [
            Attribute(0, 36, [
                Identifier(0, 3),
                Expression(6, 36, [
                    ExprTerm(6, 36, [
                        Object(6, 36, [
                            Expression(7, 13, [
                                ExprTerm(7, 13, [
                                    StringLit(7, 12, [
                                        String(8, 11)
                                    ])
                                ])
                            ]),
                            Expression(15, 20, [
                                ExprTerm(15, 20, [
                                    StringLit(15, 20, [
                                        String(16, 19)
                                    ])
                                ])
                            ]),
                            Expression(21, 27, [
                                ExprTerm(21, 27, [
                                    StringLit(21, 26, [
                                        String(22, 25)
                                    ])
                                ])
                            ]),
                            Expression(29, 35, [
                                ExprTerm(29, 35, [
                                    Variable(29, 34)
                                ])
                            ])
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parse_template() {
    parses_to! {
        parser: HclParser,
        input: "<<HEREDOC\n${foo}\n%{if asdf}qux%{endif}\nheredoc\nHEREDOC",
        rule: Rule::ExprTerm,
        tokens: [
            ExprTerm(0, 54, [
                TemplateExpr(0, 54, [
                    Heredoc(0, 54, [
                        HeredocIntroNormal(0, 2),
                        Identifier(2, 9),
                        HeredocTemplate(10, 46, [
                            TemplateInterpolation(10, 16, [
                                TemplateIExprStartNormal(10, 12),
                                Expression(12, 15, [
                                    ExprTerm(12, 15, [
                                        Variable(12, 15)
                                    ])
                                ]),
                                TemplateExprEndNormal(15, 16)
                            ]),
                            HeredocLiteral(16, 17),
                            TemplateDirective(17, 38, [
                                TemplateIf(17, 38, [
                                    TemplateIfExpr(17, 27, [
                                        TemplateDExprStartNormal(17, 19),
                                        Expression(22, 26, [
                                            ExprTerm(22, 26, [
                                                Variable(22, 26)
                                            ])
                                        ]),
                                        TemplateExprEndNormal(26, 27),
                                    ]),
                                    Template(27, 30, [
                                        TemplateLiteral(27, 30)
                                    ]),
                                    TemplateEndIfExpr(30, 38, [
                                        TemplateDExprStartNormal(30, 32),
                                        TemplateExprEndNormal(37, 38),
                                    ])
                                ])
                            ]),
                            HeredocLiteral(38, 46)
                        ])
                    ])
                ])
            ])
        ]
    };

    parses_to! {
        parser: HclParser,
        input: r#""foo ${bar} $${baz}, %{if cond ~} qux %{~ endif}""#,
        rule: Rule::ExprTerm,
        tokens: [
            ExprTerm(0, 49, [
                TemplateExpr(0, 49, [
                    QuotedStringTemplate(0, 49, [
                        QuotedStringTemplateInner(1, 48, [
                            QuotedStringTemplateLiteral(1, 5),
                            TemplateInterpolation(5, 11, [
                                TemplateIExprStartNormal(5, 7),
                                Expression(7, 10, [
                                    ExprTerm(7, 10, [
                                        Variable(7, 10)
                                    ])
                                ]),
                                TemplateExprEndNormal(10, 11)
                            ]),
                            QuotedStringTemplateLiteral(11, 21),
                            TemplateDirective(21, 48, [
                                TemplateIf(21, 48, [
                                    TemplateIfExpr(21, 33, [
                                        TemplateDExprStartNormal(21, 23),
                                        Expression(26, 31, [
                                            ExprTerm(26, 31, [
                                                Variable(26, 30)
                                            ])
                                        ]),
                                        TemplateExprEndStrip(31, 33),
                                    ]),
                                    Template(33, 38, [
                                        TemplateLiteral(33, 38)
                                    ]),
                                    TemplateEndIfExpr(38, 48, [
                                        TemplateDExprStartStrip(38, 41),
                                        TemplateExprEndNormal(47, 48),
                                    ])
                                ])
                            ])
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parse_cond_in_interpolation() {
    parses_to! {
        parser: HclParser,
        input: r#"name = "${var.l ? "us-east-1." : ""}""#,
        rule: Rule::Attribute,
        tokens: [
            Attribute(0, 37, [
                Identifier(0, 4),
                Expression(7, 37, [
                    ExprTerm(7, 37, [
                        TemplateExpr(7, 37, [
                            QuotedStringTemplate(7, 37, [
                                QuotedStringTemplateInner(8, 36, [
                                    TemplateInterpolation(8, 36, [
                                        TemplateIExprStartNormal(8, 10),
                                        Expression(10, 35, [
                                            ExprTerm(10, 15, [
                                                Variable(10, 13),
                                                GetAttr(13, 15, [
                                                    Identifier(14, 15)
                                                ])
                                            ]),
                                            Expression(18, 31, [
                                                ExprTerm(18, 31, [
                                                    StringLit(18, 30, [
                                                        String(19, 29)
                                                    ])
                                                ])
                                            ]),
                                            Expression(33, 35, [
                                                ExprTerm(33, 35, [
                                                    StringLit(33, 35, [
                                                        String(34, 34)
                                                    ])
                                                ])
                                            ])
                                        ]),
                                        TemplateExprEndNormal(35, 36)
                                    ])
                                ])
                            ])
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parse_nested_function_call_with_splat() {
    parses_to! {
        parser: HclParser,
        input: r#"element(concat(aws_kms_key.key-one.*.arn, aws_kms_key.key-two.*.arn), 0)"#,
        rule: Rule::FunctionCall,
        tokens: [
            FunctionCall(0, 72, [
                Identifier(0, 7),
                Arguments(7, 72, [
                    Expression(8, 68, [
                        ExprTerm(8, 68, [
                            FunctionCall(8, 68, [
                                Identifier(8, 14),
                                Arguments(14, 68, [
                                    Expression(15, 40, [
                                        ExprTerm(15, 40, [
                                            Variable(15, 26),
                                            GetAttr(26, 34, [
                                                Identifier(27, 34)
                                            ]),
                                            AttrSplat(34, 36),
                                            GetAttr(36, 40, [
                                                Identifier(37, 40)
                                            ]),
                                        ])
                                    ]),
                                    Expression(42, 67, [
                                        ExprTerm(42, 67, [
                                            Variable(42, 53),
                                            GetAttr(53, 61, [
                                                Identifier(54, 61)
                                            ]),
                                            AttrSplat(61, 63),
                                            GetAttr(63, 67, [
                                                Identifier(64, 67)
                                            ])
                                        ])
                                    ])
                                ])
                            ])
                        ])
                    ]),
                    Expression(70, 71, [
                        ExprTerm(70, 71, [
                            Int(70, 71)
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parse_escaped_slash_in_string() {
    parses_to! {
        parser: HclParser,
        input: r#""\\""#,
        rule: Rule::StringLit,
        tokens: [
            StringLit(0, 4, [
                String(1, 3),
            ])
        ]
    };
}
