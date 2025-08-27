use crate::infrastructure::lexer::{Lexer, Token};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during GraphQL query parsing
#[derive(Debug, Clone, Error, Serialize, Deserialize, PartialEq)]
pub enum QueryParseError {
    /// Unexpected token during parsing
    #[error("Unexpected token at position {position}: expected {expected}, found {found}")]
    UnexpectedToken {
        /// The expected token type
        expected: String,
        /// The token that was found instead
        found: String,
        /// Position in the input
        position: usize,
    },

    /// Unexpected end of input during parsing
    #[error("Unexpected end of input: expected {expected}")]
    UnexpectedEof {
        /// The expected token type
        expected: String,
    },

    /// Invalid syntax error
    #[error("Invalid syntax at position {position}: {message}")]
    InvalidSyntax {
        /// Position in the input where the syntax error occurred
        position: usize,
        /// Description of the syntax error
        message: String,
    },
}

/// GraphQL Document representing a parsed query
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub definitions: Vec<Definition>,
}

/// Top-level definition in a GraphQL document
#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    Operation(OperationDefinition),
    Fragment(FragmentDefinition),
}

/// GraphQL operation definition (query, mutation, subscription)
#[derive(Debug, Clone, PartialEq)]
pub struct OperationDefinition {
    pub operation_type: OperationType,
    pub name: Option<String>,
    pub variable_definitions: Vec<VariableDefinition>,
    pub directives: Vec<Directive>,
    pub selection_set: SelectionSet,
}

/// Type of GraphQL operation
#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

/// Variable definition in an operation
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDefinition {
    pub variable: String,
    pub type_: TypeRef,
    pub default_value: Option<Value>,
    pub directives: Vec<Directive>,
}

/// GraphQL type reference
#[derive(Debug, Clone, PartialEq)]
pub enum TypeRef {
    Named(String),
    List(Box<TypeRef>),
    NonNull(Box<TypeRef>),
}

/// GraphQL selection set (fields in braces)
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionSet {
    pub selections: Vec<Selection>,
}

/// Individual selection within a selection set
#[derive(Debug, Clone, PartialEq)]
pub enum Selection {
    Field(Field),
    InlineFragment(InlineFragment),
    FragmentSpread(FragmentSpread),
}

/// Field selection
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub alias: Option<String>,
    pub name: String,
    pub arguments: Vec<Argument>,
    pub directives: Vec<Directive>,
    pub selection_set: Option<SelectionSet>,
}

/// Field argument
#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub name: String,
    pub value: Value,
}

/// Inline fragment
#[derive(Debug, Clone, PartialEq)]
pub struct InlineFragment {
    pub type_condition: Option<String>,
    pub directives: Vec<Directive>,
    pub selection_set: SelectionSet,
}

/// Fragment spread
#[derive(Debug, Clone, PartialEq)]
pub struct FragmentSpread {
    pub name: String,
    pub directives: Vec<Directive>,
}

/// Fragment definition
#[derive(Debug, Clone, PartialEq)]
pub struct FragmentDefinition {
    pub name: String,
    pub type_condition: String,
    pub directives: Vec<Directive>,
    pub selection_set: SelectionSet,
}

/// Directive application
#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
    pub name: String,
    pub arguments: Vec<Argument>,
}

/// GraphQL value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Variable(String),
    Int(i32),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Enum(String),
    List(Vec<Value>),
    Object(HashMap<String, Value>),
}

/// Parser for GraphQL query documents
pub struct QueryParser<'input> {
    lexer: Lexer<'input>,
}

impl<'input> QueryParser<'input> {
    /// Create a new query parser
    pub fn new(input: &'input str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    /// Parse a complete GraphQL query document
    pub fn parse_document(&mut self) -> Result<Document, QueryParseError> {
        let mut definitions = Vec::new();

        while self.lexer.current_token().is_some() {
            definitions.push(self.parse_definition()?);
        }

        Ok(Document { definitions })
    }

    /// Parse a definition (operation or fragment)
    fn parse_definition(&mut self) -> Result<Definition, QueryParseError> {
        match self.lexer.current_token() {
            Some(Token::Query) | Some(Token::Mutation) | Some(Token::Subscription) => {
                Ok(Definition::Operation(self.parse_operation_definition()?))
            },
            Some(Token::Fragment) => Ok(Definition::Fragment(self.parse_fragment_definition()?)),
            Some(Token::LeftBrace) => {
                // Anonymous query operation
                Ok(Definition::Operation(OperationDefinition {
                    operation_type: OperationType::Query,
                    name: None,
                    variable_definitions: Vec::new(),
                    directives: Vec::new(),
                    selection_set: self.parse_selection_set()?,
                }))
            },
            Some(token) => Err(QueryParseError::UnexpectedToken {
                expected: "operation or fragment".to_string(),
                found: format!("{:?}", token),
                position: self.lexer.position(),
            }),
            None => Err(QueryParseError::UnexpectedEof {
                expected: "operation or fragment".to_string(),
            }),
        }
    }

    /// Parse an operation definition
    fn parse_operation_definition(&mut self) -> Result<OperationDefinition, QueryParseError> {
        let operation_type = match self.lexer.current_token() {
            Some(Token::Query) => {
                self.lexer.advance();
                OperationType::Query
            },
            Some(Token::Mutation) => {
                self.lexer.advance();
                OperationType::Mutation
            },
            Some(Token::Subscription) => {
                self.lexer.advance();
                OperationType::Subscription
            },
            Some(token) => {
                return Err(QueryParseError::UnexpectedToken {
                    expected: "query, mutation, or subscription".to_string(),
                    found: format!("{:?}", token),
                    position: self.lexer.position(),
                })
            },
            None => {
                return Err(QueryParseError::UnexpectedEof {
                    expected: "query, mutation, or subscription".to_string(),
                })
            },
        };

        let name = if let Some(Token::Name(name)) = self.lexer.current_token() {
            let name = name.clone();
            self.lexer.advance();
            Some(name)
        } else {
            None
        };

        let variable_definitions = if self.is_current_token(&Token::LeftParen) {
            self.parse_variable_definitions()?
        } else {
            Vec::new()
        };

        let directives = self.parse_directives()?;
        let selection_set = self.parse_selection_set()?;

        Ok(OperationDefinition {
            operation_type,
            name,
            variable_definitions,
            directives,
            selection_set,
        })
    }

    /// Parse variable definitions
    fn parse_variable_definitions(&mut self) -> Result<Vec<VariableDefinition>, QueryParseError> {
        self.expect_token(&Token::LeftParen)?;
        let mut variables = Vec::new();

        while !self.is_current_token(&Token::RightParen) {
            variables.push(self.parse_variable_definition()?);
        }

        self.expect_token(&Token::RightParen)?;
        Ok(variables)
    }

    /// Parse a single variable definition
    fn parse_variable_definition(&mut self) -> Result<VariableDefinition, QueryParseError> {
        self.expect_token(&Token::Dollar)?;
        let variable = self.parse_name()?;
        self.expect_token(&Token::Colon)?;
        let type_ = self.parse_type_ref()?;

        let default_value = if self.is_current_token(&Token::Equals) {
            self.lexer.advance();
            Some(self.parse_value()?)
        } else {
            None
        };

        let directives = self.parse_directives()?;

        Ok(VariableDefinition {
            variable,
            type_,
            default_value,
            directives,
        })
    }

    /// Parse type reference
    fn parse_type_ref(&mut self) -> Result<TypeRef, QueryParseError> {
        let mut base_type = if self.is_current_token(&Token::LeftBracket) {
            self.lexer.advance();
            let inner_type = self.parse_type_ref()?;
            self.expect_token(&Token::RightBracket)?;
            TypeRef::List(Box::new(inner_type))
        } else {
            TypeRef::Named(self.parse_name()?)
        };

        if self.is_current_token(&Token::Bang) {
            self.lexer.advance();
            base_type = TypeRef::NonNull(Box::new(base_type));
        }

        Ok(base_type)
    }

    /// Parse selection set
    fn parse_selection_set(&mut self) -> Result<SelectionSet, QueryParseError> {
        self.expect_token(&Token::LeftBrace)?;
        let mut selections = Vec::new();

        while !self.is_current_token(&Token::RightBrace) {
            selections.push(self.parse_selection()?);
        }

        self.expect_token(&Token::RightBrace)?;
        Ok(SelectionSet { selections })
    }

    /// Parse a selection
    fn parse_selection(&mut self) -> Result<Selection, QueryParseError> {
        if self.is_current_token(&Token::Spread) {
            self.lexer.advance();
            if let Some(Token::Name(name)) = self.lexer.current_token() {
                // Fragment spread
                let name = name.clone();
                self.lexer.advance();
                let directives = self.parse_directives()?;
                Ok(Selection::FragmentSpread(FragmentSpread {
                    name,
                    directives,
                }))
            } else {
                // Inline fragment
                let type_condition = if self.is_current_token(&Token::On) {
                    self.lexer.advance();
                    Some(self.parse_name()?)
                } else {
                    None
                };
                let directives = self.parse_directives()?;
                let selection_set = self.parse_selection_set()?;
                Ok(Selection::InlineFragment(InlineFragment {
                    type_condition,
                    directives,
                    selection_set,
                }))
            }
        } else {
            Ok(Selection::Field(self.parse_field()?))
        }
    }

    /// Parse a field
    fn parse_field(&mut self) -> Result<Field, QueryParseError> {
        let first_name = self.parse_name()?;

        let (alias, name) = if self.is_current_token(&Token::Colon) {
            self.lexer.advance();
            let name = self.parse_name()?;
            (Some(first_name), name)
        } else {
            (None, first_name)
        };

        let arguments = if self.is_current_token(&Token::LeftParen) {
            self.parse_arguments()?
        } else {
            Vec::new()
        };

        let directives = self.parse_directives()?;

        let selection_set = if self.is_current_token(&Token::LeftBrace) {
            Some(self.parse_selection_set()?)
        } else {
            None
        };

        Ok(Field {
            alias,
            name,
            arguments,
            directives,
            selection_set,
        })
    }

    /// Parse arguments
    fn parse_arguments(&mut self) -> Result<Vec<Argument>, QueryParseError> {
        self.expect_token(&Token::LeftParen)?;
        let mut arguments = Vec::new();

        while !self.is_current_token(&Token::RightParen) {
            let name = self.parse_name()?;
            self.expect_token(&Token::Colon)?;
            let value = self.parse_value()?;
            arguments.push(Argument { name, value });
        }

        self.expect_token(&Token::RightParen)?;
        Ok(arguments)
    }

    /// Parse directives
    fn parse_directives(&mut self) -> Result<Vec<Directive>, QueryParseError> {
        let mut directives = Vec::new();

        while self.is_current_token(&Token::At) {
            self.lexer.advance();
            let name = self.parse_name()?;
            let arguments = if self.is_current_token(&Token::LeftParen) {
                self.parse_arguments()?
            } else {
                Vec::new()
            };
            directives.push(Directive { name, arguments });
        }

        Ok(directives)
    }

    /// Parse fragment definition
    fn parse_fragment_definition(&mut self) -> Result<FragmentDefinition, QueryParseError> {
        self.expect_token(&Token::Fragment)?;
        let name = self.parse_name()?;
        self.expect_token(&Token::On)?;
        let type_condition = self.parse_name()?;
        let directives = self.parse_directives()?;
        let selection_set = self.parse_selection_set()?;

        Ok(FragmentDefinition {
            name,
            type_condition,
            directives,
            selection_set,
        })
    }

    /// Parse value
    fn parse_value(&mut self) -> Result<Value, QueryParseError> {
        match self.lexer.current_token() {
            Some(Token::Dollar) => {
                self.lexer.advance();
                Ok(Value::Variable(self.parse_name()?))
            },
            Some(Token::Integer(i)) => {
                let value = *i as i32; // Convert i64 to i32
                self.lexer.advance();
                Ok(Value::Int(value))
            },
            Some(Token::Float(f)) => {
                let value = *f;
                self.lexer.advance();
                Ok(Value::Float(value))
            },
            Some(Token::String(s)) => {
                let value = s.clone();
                self.lexer.advance();
                Ok(Value::String(value))
            },
            Some(Token::True) => {
                self.lexer.advance();
                Ok(Value::Boolean(true))
            },
            Some(Token::False) => {
                self.lexer.advance();
                Ok(Value::Boolean(false))
            },
            Some(Token::Null) => {
                self.lexer.advance();
                Ok(Value::Null)
            },
            Some(Token::Name(name)) => {
                let value = name.clone();
                self.lexer.advance();
                Ok(Value::Enum(value))
            },
            Some(Token::LeftBracket) => self.parse_list_value(),
            Some(Token::LeftBrace) => self.parse_object_value(),
            Some(token) => Err(QueryParseError::UnexpectedToken {
                expected: "value".to_string(),
                found: format!("{:?}", token),
                position: self.lexer.position(),
            }),
            None => Err(QueryParseError::UnexpectedEof {
                expected: "value".to_string(),
            }),
        }
    }

    /// Parse list value
    fn parse_list_value(&mut self) -> Result<Value, QueryParseError> {
        self.expect_token(&Token::LeftBracket)?;
        let mut values = Vec::new();

        while !self.is_current_token(&Token::RightBracket) {
            values.push(self.parse_value()?);
        }

        self.expect_token(&Token::RightBracket)?;
        Ok(Value::List(values))
    }

    /// Parse object value
    fn parse_object_value(&mut self) -> Result<Value, QueryParseError> {
        self.expect_token(&Token::LeftBrace)?;
        let mut object = HashMap::new();

        while !self.is_current_token(&Token::RightBrace) {
            let key = self.parse_name()?;
            self.expect_token(&Token::Colon)?;
            let value = self.parse_value()?;
            object.insert(key, value);
        }

        self.expect_token(&Token::RightBrace)?;
        Ok(Value::Object(object))
    }

    /// Parse name token
    fn parse_name(&mut self) -> Result<String, QueryParseError> {
        match self.lexer.current_token() {
            Some(Token::Name(name)) => {
                let name = name.clone();
                self.lexer.advance();
                Ok(name)
            },
            Some(token) => Err(QueryParseError::UnexpectedToken {
                expected: "name".to_string(),
                found: format!("{:?}", token),
                position: self.lexer.position(),
            }),
            None => Err(QueryParseError::UnexpectedEof {
                expected: "name".to_string(),
            }),
        }
    }

    /// Check if current token matches expected
    fn is_current_token(&self, expected: &Token) -> bool {
        if let Some(current) = self.lexer.current_token() {
            std::mem::discriminant(current) == std::mem::discriminant(expected)
        } else {
            false
        }
    }

    /// Expect a specific token and advance
    fn expect_token(&mut self, expected: &Token) -> Result<(), QueryParseError> {
        if self.is_current_token(expected) {
            self.lexer.advance();
            Ok(())
        } else {
            match self.lexer.current_token() {
                Some(token) => Err(QueryParseError::UnexpectedToken {
                    expected: format!("{:?}", expected),
                    found: format!("{:?}", token),
                    position: self.lexer.position(),
                }),
                None => Err(QueryParseError::UnexpectedEof {
                    expected: format!("{:?}", expected),
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let input = r#"
        query GetUser {
            user {
                id
                name
            }
        }
        "#;

        let mut parser = QueryParser::new(input);
        let document = parser.parse_document().unwrap();

        assert_eq!(document.definitions.len(), 1);

        if let Definition::Operation(op) = &document.definitions[0] {
            assert_eq!(op.operation_type, OperationType::Query);
            assert_eq!(op.name, Some("GetUser".to_string()));
            assert_eq!(op.selection_set.selections.len(), 1);
        } else {
            panic!("Expected operation definition");
        }
    }

    #[test]
    fn test_parse_query_with_variables() {
        let input = r#"
        query GetUser($id: ID!, $name: String = "default") {
            user(id: $id, name: $name) {
                id
                name
            }
        }
        "#;

        let mut parser = QueryParser::new(input);
        let document = parser.parse_document().unwrap();

        if let Definition::Operation(op) = &document.definitions[0] {
            assert_eq!(op.variable_definitions.len(), 2);
            assert_eq!(op.variable_definitions[0].variable, "id");
            assert_eq!(op.variable_definitions[1].variable, "name");
        } else {
            panic!("Expected operation definition");
        }
    }

    #[test]
    fn test_parse_mutation() {
        let input = r#"
        mutation CreateUser($userData: UserData!) {
            createUser(userData: $userData) {
                id
                name
            }
        }
        "#;

        let mut parser = QueryParser::new(input);
        let document = parser.parse_document().unwrap();

        if let Definition::Operation(op) = &document.definitions[0] {
            assert_eq!(op.operation_type, OperationType::Mutation);
            assert_eq!(op.name, Some("CreateUser".to_string()));
        } else {
            panic!("Expected operation definition");
        }
    }

    #[test]
    fn test_parse_anonymous_query() {
        let input = r#"
        {
            user {
                id
                name
            }
        }
        "#;

        let mut parser = QueryParser::new(input);
        let document = parser.parse_document().unwrap();

        if let Definition::Operation(op) = &document.definitions[0] {
            assert_eq!(op.operation_type, OperationType::Query);
            assert_eq!(op.name, None);
        } else {
            panic!("Expected operation definition");
        }
    }

    #[test]
    fn test_parse_field_with_alias() {
        let input = r#"
        {
            currentUser: user {
                id
                displayName: name
            }
        }
        "#;

        let mut parser = QueryParser::new(input);
        let document = parser.parse_document().unwrap();

        if let Definition::Operation(op) = &document.definitions[0] {
            if let Selection::Field(field) = &op.selection_set.selections[0] {
                assert_eq!(field.alias, Some("currentUser".to_string()));
                assert_eq!(field.name, "user");
            }
        }
    }
}
