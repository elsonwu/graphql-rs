/// Simple test to verify our lexer works correctly
fn main() {
    use graphql_rs::infrastructure::lexer::{Lexer, Token};

    println!("Testing GraphQL Lexer...");

    let input = r#"type User { id: ID! name: String }"#;
    let mut lexer = Lexer::new(input);

    let mut tokens = Vec::new();
    while let Some(token) = lexer.advance() {
        tokens.push(token);
    }

    println!("Tokenized {} tokens from: {}", tokens.len(), input);

    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token);
    }

    // Verify we got the expected tokens
    assert!(tokens.contains(&Token::Type));
    assert!(tokens.contains(&Token::Name("User".to_string())));
    assert!(tokens.contains(&Token::LeftBrace));
    assert!(tokens.contains(&Token::Name("id".to_string())));
    assert!(tokens.contains(&Token::Colon));
    assert!(tokens.contains(&Token::Name("ID".to_string())));
    assert!(tokens.contains(&Token::Bang));

    println!("\n✅ Lexer test passed! All expected tokens found.");
}
