// src/auth_error.rs
use thiserror::Error;
use mysql::Error as MySqlError;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Database error: {0}")]
    Database(#[from] MySqlError),
    #[error("Invalid credentials")]
    InvalidCredentials,       // Para login falho
    #[error("Unauthorized: Please log in")]
    Unauthorized,             // Para quando o usuário não está logado
    #[error("Forbidden: Insufficient permissions")]
    Forbidden,                // Para quando o usuário não é admin
}