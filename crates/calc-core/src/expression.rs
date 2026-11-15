use crate::{evaluate, ArithmeticError, Operator};
use std::{fmt, ops::Range};

const MAX_DEPTH: usize = 128;

type Span = Range<usize>;
type NodeId = usize;

#[derive(Debug)]
pub struct ExpressionError {
    kind: ErrorKind,
    span: Span,
}

#[derive(Debug)]
enum ErrorKind {
    Syntax(&'static str),
    LiteralOutOfRange,
    TooDeep,
    Arithmetic(ArithmeticError),
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Syntax(message) => write!(f, "{message}")?,
            ErrorKind::LiteralOutOfRange => write!(f, "integer literal out of range")?,
            ErrorKind::TooDeep => write!(f, "expression nesting exceeds {MAX_DEPTH}")?,
            ErrorKind::Arithmetic(error) => write!(f, "{error}")?,
        }
        write!(f, " at bytes {}..{}", self.span.start, self.span.end)
    }
}

impl std::error::Error for ExpressionError {}

#[derive(Clone, Copy)]
enum TokenKind<'input> {
    Number(&'input str),
    Operator(Operator),
    Open,
    Close,
    End,
}

struct Token<'input> {
    kind: TokenKind<'input>,
    span: Span,
}

struct Lexer<'input> {
    input: &'input str,
    offset: usize,
}

impl<'input> Lexer<'input> {
    fn next(&mut self) -> Result<Token<'input>, ExpressionError> {
        while let Some(character) = self.input[self.offset..].chars().next() {
            if !character.is_whitespace() {
                break;
            }
            self.offset += character.len_utf8();
        }
        let start = self.offset;
        let kind = if let Some(character) = self.input[start..].chars().next() {
            self.offset += character.len_utf8();
            match character {
                '0'..='9' => {
                    while self
                        .input
                        .as_bytes()
                        .get(self.offset)
                        .is_some_and(u8::is_ascii_digit)
                    {
                        self.offset += 1;
                    }
                    TokenKind::Number(&self.input[start..self.offset])
                }
                '+' => TokenKind::Operator(Operator::Add),
                '-' => TokenKind::Operator(Operator::Subtract),
                '*' => TokenKind::Operator(Operator::Multiply),
                '/' => TokenKind::Operator(Operator::Divide),
                '(' => TokenKind::Open,
                ')' => TokenKind::Close,
                _ => {
                    return Err(ExpressionError {
                        kind: ErrorKind::Syntax("unexpected character"),
                        span: start..self.offset,
                    })
                }
            }
        } else {
            TokenKind::End
        };
        Ok(Token {
            kind,
            span: start..self.offset,
        })
    }
}

enum Operation {
    Negate(NodeId),
    Binary(Operator, NodeId, NodeId),
}

enum NodeKind<Literal> {
    Literal(Literal),
    Operation(Operation),
}

struct Node<Literal> {
    kind: NodeKind<Literal>,
    span: Span,
}

struct Tree<Literal> {
    nodes: Vec<Node<Literal>>,
    root: NodeId,
}

struct Parser<'input> {
    lexer: Lexer<'input>,
    lookahead: Token<'input>,
    nodes: Vec<Node<&'input str>>,
}

impl<'input> Parser<'input> {
    fn parse(input: &'input str) -> Result<Tree<&'input str>, ExpressionError> {
        let mut lexer = Lexer { input, offset: 0 };
        let lookahead = lexer.next()?;
        let mut parser = Self {
            lexer,
            lookahead,
            nodes: Vec::new(),
        };
        let root = parser.expression(0, 0)?;
        if !matches!(parser.lookahead.kind, TokenKind::End) {
            return Err(parser.syntax("expected end of expression"));
        }
        Ok(Tree {
            nodes: parser.nodes,
            root,
        })
    }

    fn bump(&mut self) -> Result<Token<'input>, ExpressionError> {
        let next = self.lexer.next()?;
        Ok(std::mem::replace(&mut self.lookahead, next))
    }

    fn syntax(&self, message: &'static str) -> ExpressionError {
        ExpressionError {
            kind: ErrorKind::Syntax(message),
            span: self.lookahead.span.clone(),
        }
    }

    fn push(&mut self, kind: NodeKind<&'input str>, span: Span) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Node { kind, span });
        id
    }

    fn expression(&mut self, minimum: u8, depth: usize) -> Result<NodeId, ExpressionError> {
        check_depth(depth, &self.lookahead.span)?;
        let mut left = match self.lookahead.kind {
            TokenKind::Number(value) => {
                let token = self.bump()?;
                self.push(NodeKind::Literal(value), token.span)
            }
            TokenKind::Operator(Operator::Subtract) => {
                let token = self.bump()?;
                let operand = self.expression(5, depth + 1)?;
                self.push(NodeKind::Operation(Operation::Negate(operand)), token.span)
            }
            TokenKind::Open => {
                self.bump()?;
                let inner = self.expression(0, depth + 1)?;
                if !matches!(self.lookahead.kind, TokenKind::Close) {
                    return Err(self.syntax("expected closing parenthesis"));
                }
                self.bump()?;
                inner
            }
            _ => return Err(self.syntax("expected a number, unary minus, or opening parenthesis")),
        };

        while let TokenKind::Operator(operator) = self.lookahead.kind {
            let precedence = match operator {
                Operator::Add | Operator::Subtract => 1,
                Operator::Multiply | Operator::Divide => 3,
            };
            if precedence < minimum {
                break;
            }
            let token = self.bump()?;
            let right = self.expression(precedence + 1, depth + 1)?;
            left = self.push(
                NodeKind::Operation(Operation::Binary(operator, left, right)),
                token.span,
            );
        }
        Ok(left)
    }
}

fn check_depth(depth: usize, span: &Span) -> Result<(), ExpressionError> {
    if depth >= MAX_DEPTH {
        return Err(ExpressionError {
            kind: ErrorKind::TooDeep,
            span: span.clone(),
        });
    }
    Ok(())
}

fn checked_literal(text: &str, span: &Span) -> Result<i64, ExpressionError> {
    text.parse().map_err(|_| ExpressionError {
        kind: ErrorKind::LiteralOutOfRange,
        span: span.clone(),
    })
}

impl Tree<&str> {
    fn validate(&self) -> Result<Tree<i64>, ExpressionError> {
        let mut validated = Tree {
            nodes: Vec::with_capacity(self.nodes.len()),
            root: 0,
        };
        validated.root = self.validate_node(self.root, 0, &mut validated.nodes)?;
        Ok(validated)
    }

    fn validate_node(
        &self,
        id: NodeId,
        depth: usize,
        nodes: &mut Vec<Node<i64>>,
    ) -> Result<NodeId, ExpressionError> {
        let node = &self.nodes[id];
        check_depth(depth, &node.span)?;
        let kind = match node.kind {
            NodeKind::Literal(text) => NodeKind::Literal(checked_literal(text, &node.span)?),
            NodeKind::Operation(Operation::Negate(child)) => match self.nodes[child].kind {
                NodeKind::Literal(text)
                    if text.trim_start_matches('0') == "9223372036854775808" =>
                {
                    check_depth(depth + 1, &self.nodes[child].span)?;
                    NodeKind::Literal(i64::MIN)
                }
                _ => NodeKind::Operation(Operation::Negate(self.validate_node(
                    child,
                    depth + 1,
                    nodes,
                )?)),
            },
            NodeKind::Operation(Operation::Binary(operator, left, right)) => {
                let left = self.validate_node(left, depth + 1, nodes)?;
                let right = self.validate_node(right, depth + 1, nodes)?;
                NodeKind::Operation(Operation::Binary(operator, left, right))
            }
        };
        let id = nodes.len();
        nodes.push(Node {
            kind,
            span: node.span.clone(),
        });
        Ok(id)
    }
}

impl Tree<i64> {
    fn evaluate_node(&self, id: NodeId) -> Result<i64, ExpressionError> {
        let node = &self.nodes[id];
        let result = match node.kind {
            NodeKind::Literal(value) => Ok(value),
            NodeKind::Operation(Operation::Negate(child)) => self
                .evaluate_node(child)?
                .checked_neg()
                .ok_or(ArithmeticError::Overflow),
            NodeKind::Operation(Operation::Binary(operator, left, right)) => {
                let left = self.evaluate_node(left)?;
                let right = self.evaluate_node(right)?;
                evaluate(left, operator, right)
            }
        };
        result.map_err(|error| ExpressionError {
            kind: ErrorKind::Arithmetic(error),
            span: node.span.clone(),
        })
    }
}

pub fn evaluate_expression(input: &str) -> Result<i64, ExpressionError> {
    let tree = Parser::parse(input)?.validate()?;
    tree.evaluate_node(tree.root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_precedence_associativity_and_signed_boundaries() {
        for (input, expected) in [
            ("2 + 3 * 4", 14),
            ("(2 + 3) * 4", 20),
            ("20 / 5 / 2", 2),
            ("8 - 3 - 2", 3),
            ("2 * -(3 + 4)", -14),
            ("-7 / 3", -2),
            ("1\u{2003}+\u{2003}2", 3),
            ("-9223372036854775808", i64::MIN),
            ("-(0009223372036854775808)", i64::MIN),
            ("9223372036854775807", i64::MAX),
        ] {
            assert_eq!(evaluate_expression(input).unwrap(), expected, "{input}");
        }
    }

    #[test]
    fn reports_syntax_semantic_and_runtime_errors_with_byte_spans() {
        for (input, expected) in [
            (
                "",
                "expected a number, unary minus, or opening parenthesis at bytes 0..0",
            ),
            ("(1 + 2", "expected closing parenthesis at bytes 6..6"),
            ("1 2", "expected end of expression at bytes 2..3"),
            ("1 + é", "unexpected character at bytes 4..6"),
            (
                "9223372036854775808",
                "integer literal out of range at bytes 0..19",
            ),
            ("--9223372036854775808", "integer overflow at bytes 0..1"),
            (
                "(1 / 0) + (9223372036854775807 + 1)",
                "division by zero at bytes 3..4",
            ),
        ] {
            assert_eq!(
                evaluate_expression(input).unwrap_err().to_string(),
                expected,
                "{input}"
            );
        }
    }

    #[test]
    fn bounds_parser_recursion_and_left_associated_tree_depth() {
        let nested = format!("{}1{}", "(".repeat(MAX_DEPTH), ")".repeat(MAX_DEPTH));
        let chain = format!("{}1", "1+".repeat(MAX_DEPTH));
        for input in [nested, chain] {
            assert!(matches!(
                evaluate_expression(&input).unwrap_err().kind,
                ErrorKind::TooDeep
            ));
        }
    }

    #[test]
    fn bounds_negated_literals_before_folding_minimum() {
        for (literal, expected) in [("-1", -1), ("-9223372036854775808", i64::MIN)] {
            let allowed = format!("{literal}{}", "+0".repeat(MAX_DEPTH - 2));
            assert_eq!(evaluate_expression(&allowed).unwrap(), expected);
            let rejected = format!("{allowed}+0");
            let error = evaluate_expression(&rejected).unwrap_err();
            assert!(matches!(error.kind, ErrorKind::TooDeep));
            assert_eq!(error.span, 1..literal.len());
        }
    }

    #[test]
    fn validates_all_literals_before_evaluating_left_to_right() {
        let error = evaluate_expression("1/0 + 9223372036854775808").unwrap_err();
        assert!(matches!(error.kind, ErrorKind::LiteralOutOfRange));
        assert_eq!(error.span, 6..25);

        let error = evaluate_expression("1/0 + (9223372036854775807 + 1)").unwrap_err();
        assert!(matches!(
            error.kind,
            ErrorKind::Arithmetic(ArithmeticError::DivisionByZero)
        ));
        assert_eq!(error.span, 1..2);
    }
}
