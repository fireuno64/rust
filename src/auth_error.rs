// src/auth_error.rs
use thiserror::Error;
use bcrypt::BcryptError;
use mysql::Error as MySqlError;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] BcryptError),
    #[error("Database error: {0}")]
    Database(#[from] MySqlError),
    // Adicione outras variantes de erro específicas de autenticação aqui
}