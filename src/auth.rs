// src/auth.rs

use serde::{Deserialize, Serialize};
use mysql::{PooledConn, Row}; // Row está sem uso, mas podemos deixar por enquanto.
use mysql::params; // CORRIGIDO: Adicionado import para a macro `params!`
use mysql::prelude::{Queryable, FromRow}; // CORRIGIDO: Adicionado FromRow aqui também para Usuario

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)] // CORRIGIDO: Adicionado FromRow
pub struct Usuario {
    pub id: i32,
    pub username: String,
    pub password: String, // Mantido como texto plano para homologação, como solicitado
    pub is_admin: bool,
}

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

    println!("Resultado da consulta usuário: {:?}", user_opt);

    if let Some(user) = user_opt {
        println!("Senha do DB (texto plano): {}", user.password);
        println!("Senha fornecida (texto plano): {}", password_attempt);
        
        if user.password == password_attempt {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}