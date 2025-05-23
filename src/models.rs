// src/models.rs
use serde::{Deserialize, Serialize};
use mysql::prelude::FromRow;
use rust_decimal::Decimal;

// Definição Canônica da Struct Cliente
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, FromRow)]
pub struct Cliente {
    pub id: u32,
    pub nome_cliente: String,
    pub telefone_cliente: Option<String>,
    pub nome_crianca: String,
    pub endereco_cliente: String,
    pub valor_mensalidade: Decimal, // Usando Decimal para valores monetários
    pub escola_destino: String,
    pub endereco_escola: String,
    pub motorista_perua: String,
}

// Struct para o formulário de cliente (dados de entrada para criar/atualizar)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FormCliente {
    pub nome_cliente: String,
    pub telefone_cliente: Option<String>,
    pub nome_crianca: String,
    pub endereco_cliente: String,
    pub valor_mensalidade: Decimal,
    pub escola_destino: String,
    pub endereco_escola: String,
    pub motorista_perua: String,
}

// Definição Canônica da Struct Usuario
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Usuario {
    pub id: i32,
    pub username: String,
    pub password: String, // Mantido como String para texto plano
    pub is_admin: bool,
}