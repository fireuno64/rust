// src/clientes.rs

// Removidos Deserialize e Serialize, pois FormCliente e Cliente já estão em models.rs e serializados/desserializados lá.
use mysql::{prelude::Queryable, Pool, params, Error as MySqlError};
use rust_decimal::Decimal;

use crate::models::{Cliente, FormCliente}; // Importa Cliente e FormCliente de models.rs

// Funções de CRUD agora focadas apenas na interação com o banco de dados.
// A verificação de permissão será feita pelo middleware antes destas funções serem chamadas.

pub fn adicionar_cliente(pool: &Pool, form: &FormCliente) -> Result<(), MySqlError> {
    let mut conn = pool.get_conn()?;
    conn.exec_drop(
        "INSERT INTO clientes (nome_cliente, telefone_cliente, nome_crianca, endereco_cliente, valor_mensalidade, escola_destino, endereco_escola, motorista_perua)
         VALUES (:nome_cliente, :telefone_cliente, :nome_crianca, :endereco_cliente, :valor_mensalidade, :escola_destino, :endereco_escola, :motorista_perua)",
        params! {
            "nome_cliente" => &form.nome_cliente,
            "telefone_cliente" => &form.telefone_cliente,
            "nome_crianca" => &form.nome_crianca,
            "endereco_cliente" => &form.endereco_cliente,
            "valor_mensalidade" => &form.valor_mensalidade,
            "escola_destino" => &form.escola_destino,
            "endereco_escola" => &form.endereco_escola,
            "motorista_perua" => &form.motorista_perua,
        },
    )?;
    Ok(())
}

pub fn atualizar_cliente(pool: &Pool, id: u32, form: &FormCliente) -> Result<(), MySqlError> {
    let mut conn = pool.get_conn()?;
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
            "valor_mensalidade" => &form.valor_mensalidade,
            "escola_destino" => &form.escola_destino,
            "endereco_escola" => &form.endereco_escola,
            "motorista_perua" => &form.motorista_perua,
        },
    )?;
    Ok(())
}

pub fn remover_cliente(pool: &Pool, id: u32) -> Result<(), MySqlError> {
    let mut conn = pool.get_conn()?;
    conn.exec_drop("DELETE FROM clientes WHERE id = :id", params! { "id" => id })?;
    Ok(())
}

pub fn listar_clientes(pool: &Pool) -> Result<Vec<Cliente>, MySqlError> {
    let mut conn = pool.get_conn()?;
    let clientes = conn.query_map(
        "SELECT id, nome_cliente, telefone_cliente, nome_crianca, endereco_cliente, valor_mensalidade, escola_destino, endereco_escola, motorista_perua FROM clientes",
        |(id, nome_cliente, telefone_cliente, nome_crianca, endereco_cliente, valor_mensalidade, escola_destino, endereco_escola, motorista_perua): (u32, String, Option<String>, String, String, Decimal, String, String, String)| {
            Cliente {
                id,
                nome_cliente,
                telefone_cliente,
                nome_crianca,
                endereco_cliente,
                valor_mensalidade,
                escola_destino,
                endereco_escola,
                motorista_perua,
            }
        },
    )?;
    Ok(clientes)
}

pub fn buscar_cliente_por_id(pool: &Pool, id: u32) -> Result<Option<Cliente>, MySqlError> {
    let mut conn = pool.get_conn()?;
    conn.exec_first(
        "SELECT id, nome_cliente, telefone_cliente, nome_crianca, endereco_cliente, valor_mensalidade, escola_destino, endereco_escola, motorista_perua
         FROM clientes WHERE id = :id",
        params! { "id" => id },
    )
}