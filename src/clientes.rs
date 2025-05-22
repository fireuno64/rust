// src/clientes.rs

use serde::{Deserialize, Serialize};
use mysql::{prelude::Queryable, prelude::FromRow, Pool, params};
use rust_decimal::Decimal; // IMPORTANTE: Adicionado para usar o tipo Decimal

// Struct para representar um cliente do banco de dados
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)] // Clone para permitir passar para Tera Context
pub struct Cliente {
    pub id: u32,
    pub nome_cliente: String,
    pub telefone_cliente: Option<String>,
    pub nome_crianca: String,
    pub endereco_cliente: String,
    pub valor_mensalidade: Decimal, // CORRIGIDO: Agora é Decimal!
    pub escola_destino: String,
    pub endereco_escola: String,
    pub motorista_perua: String,
}

// Struct para o formulário de cliente (recebido do cliente)
#[derive(Debug, Deserialize)]
pub struct FormCliente {
    pub nome_cliente: String,
    pub telefone_cliente: Option<String>, // CORRIGIDO: Pode ser Option<String> se o formulário permitir vazio
    pub nome_crianca: String,
    pub endereco_cliente: String,
    pub valor_mensalidade: Decimal, // CORRIGIDO: Agora é Decimal! Serde fará o parse da string para Decimal.
    pub escola_destino: String,
    pub endereco_escola: String,
    pub motorista_perua: String,
}

pub fn listar_clientes(pool: &Pool) -> mysql::Result<Vec<Cliente>> {
    let mut conn = pool.get_conn()?;
    // A trait `FromRow` para `Cliente` (derivada) deve ser capaz de mapear diretamente `DECIMAL` do MySQL para `rust_decimal::Decimal`.
    let clientes = conn.query_map(
        "SELECT id, nome_cliente, telefone_cliente, nome_crianca, endereco_cliente, valor_mensalidade, escola_destino, endereco_escola, motorista_perua FROM clientes",
        |(id, nome_cliente, telefone_cliente, nome_crianca, endereco_cliente, valor_mensalidade, escola_destino, endereco_escola, motorista_perua): (u32, String, Option<String>, String, String, Decimal, String, String, String)| { // Especifica os tipos para garantir o Decimal
            Cliente {
                id,
                nome_cliente,
                telefone_cliente,
                nome_crianca,
                endereco_cliente,
                valor_mensalidade, // Já é Decimal aqui
                escola_destino,
                endereco_escola,
                motorista_perua,
            }
        },
    )?;
    Ok(clientes)
}

pub fn adicionar_cliente(pool: &Pool, form: &FormCliente) -> mysql::Result<()> {
    let mut conn = pool.get_conn()?;
    // Não é mais necessário parsear String para f64, o Decimal já vem do formulário
    conn.exec_drop(
        "INSERT INTO clientes (nome_cliente, telefone_cliente, nome_crianca, endereco_cliente, valor_mensalidade, escola_destino, endereco_escola, motorista_perua)
         VALUES (:nome_cliente, :telefone_cliente, :nome_crianca, :endereco_cliente, :valor_mensalidade, :escola_destino, :endereco_escola, :motorista_perua)",
        params! {
            "nome_cliente" => &form.nome_cliente,
            "telefone_cliente" => &form.telefone_cliente,
            "nome_crianca" => &form.nome_crianca,
            "endereco_cliente" => &form.endereco_cliente,
            "valor_mensalidade" => &form.valor_mensalidade, // Passe o Decimal diretamente
            "escola_destino" => &form.escola_destino,
            "endereco_escola" => &form.endereco_escola,
            "motorista_perua" => &form.motorista_perua,
        },
    )?;
    Ok(())
}

pub fn buscar_cliente_por_id(pool: &Pool, id: u32) -> mysql::Result<Option<Cliente>> {
    let mut conn = pool.get_conn()?;
    // O FromRow derivado deve funcionar com Decimal aqui também
    conn.exec_first(
        "SELECT id, nome_cliente, telefone_cliente, nome_crianca, endereco_cliente, valor_mensalidade, escola_destino, endereco_escola, motorista_perua
         FROM clientes WHERE id = :id",
        params! { "id" => id },
    )
}

pub fn atualizar_cliente(pool: &Pool, id: u32, form: &FormCliente) -> mysql::Result<()> {
    let mut conn = pool.get_conn()?;
    // Não é mais necessário parsear String para f64, o Decimal já vem do formulário
    conn.exec_drop(
        "UPDATE clientes SET
             nome_cliente = :nome_cliente,
             telefone_cliente = :telefone_cliente,
             nome_crianca = :nome_crianca,
             endereco_cliente = :endereco_cliente,
             valor_mensalidade = :valor_mensalidade,
             escola_destino = :escola_destino,
             endereco_escola = :endereco_escola,
             motorista_perua = :motorista_perua
           WHERE id = :id",
        params! {
            "id" => id,
            "nome_cliente" => &form.nome_cliente,
            "telefone_cliente" => &form.telefone_cliente,
            "nome_crianca" => &form.nome_crianca,
            "endereco_cliente" => &form.endereco_cliente,
            "valor_mensalidade" => &form.valor_mensalidade, // Passe o Decimal diretamente
            "escola_destino" => &form.escola_destino,
            "endereco_escola" => &form.endereco_escola,
            "motorista_perua" => &form.motorista_perua,
        },
    )?;
    Ok(())
}

pub fn remover_cliente(pool: &Pool, id: u32) -> mysql::Result<()> {
    let mut conn = pool.get_conn()?;
    conn.exec_drop("DELETE FROM clientes WHERE id = :id", params! { "id" => id })?;
    Ok(())
}