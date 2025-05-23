// src/auth_error.rs
use thiserror::Error;
use mysql::Error as MySqlError; // Certifique-se de que 'mysql' está sendo usado em algum lugar para Database(#[from] MySqlError)

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Database error: {0}")]
    Database(#[from] MySqlError),
    // #[error("Invalid credentials")] // Linha removida
    // InvalidCredentials,            // Linha removida
    // #[error("Unauthorized: Please log in")] // Linha removida
    // Unauthorized,                  // Linha removida
    #[error("Forbidden: Insufficient permissions")]
    Forbidden,                       // Para quando o usuário não é admin
}