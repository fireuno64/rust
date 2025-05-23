// src/auth.rs

use serde::Deserialize; // Removido Serialize pois não é usado aqui
use mysql::PooledConn;
use mysql::params;
use mysql::prelude::{Queryable};
use crate::models::Usuario; // Importa Usuario de models.rs

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

/// Autentica usuário a partir do username e senha (texto plano).
/// Retorna Ok(Some(usuario)) se autenticado, Ok(None) se inválido, Err para erro DB.
pub fn autenticar_usuario(conn: &mut PooledConn, username: &str, password_attempt: &str) -> Result<Option<Usuario>, mysql::Error> {
    let query = "SELECT id, username, password, is_admin FROM usuarios WHERE username = :username LIMIT 1";

    let params = mysql::params! {
        "username" => username,
    };

    let user_opt: Option<Usuario> = conn.exec_first(
        query,
        params
    )?;

    if let Some(user) = user_opt {
        if user.password == password_attempt {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}