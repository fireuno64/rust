use mysql::prelude::*;
use mysql::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Cliente {
    pub id: i32,
    pub nome_cliente: String,
    pub nome_crianca: String,
    pub endereco_cliente: String,
    pub valor_mensalidade: f64,
    pub escola_destino: String,
    pub endereco_escola: String,
    pub motorista_perua: String,
    pub telefone_cliente: Option<String>, // Alterado para Option<String>
}

impl FromRow for Cliente {
    fn from_row(row: Row) -> Self {
        let (
            id,
            nome_cliente,
            nome_crianca,
            endereco_cliente,
            valor_mensalidade,
            escola_destino,
            endereco_escola,
            motorista_perua,
            telefone_cliente, // Este campo agora pode ser Option<String>
        ) = from_row(row);
        Cliente {
            id,
            nome_cliente,
            nome_crianca,
            endereco_cliente,
            valor_mensalidade,
            escola_destino,
            endereco_escola,
            motorista_perua,
            telefone_cliente,
        }
    }

    fn from_row_opt(row: Row) -> Result<Self, FromRowError> {
        let result = (
            row.get(0).map(|v: Value| {
                if let Value::Int(i) = v {
                    i as i32
                } else if let Value::UInt(u) = v {
                    u as i32
                } else {
                    0
                }
            }).ok_or(FromRowError(row.clone())),
            row.get(1).map(|v: Value| match v {
                Value::Bytes(b) => String::from_utf8_lossy(&b).into_owned(),
                _ => String::new(),
            }).ok_or(FromRowError(row.clone())),
            row.get(2).map(|v: Value| match v {
                Value::Bytes(b) => String::from_utf8_lossy(&b).into_owned(),
                _ => String::new(),
            }).ok_or(FromRowError(row.clone())),
            row.get(3).map(|v: Value| match v {
                Value::Bytes(b) => String::from_utf8_lossy(&b).into_owned(),
                _ => String::new(),
            }).ok_or(FromRowError(row.clone())),
            row.get(4).map(|v: Value| {
                if let Value::Double(f) = v {
                    f
                } else if let Value::Float(f) = v {
                    f as f64
                } else {
                    0.0
                }
            }).ok_or(FromRowError(row.clone())),
            row.get(5).map(|v: Value| match v {
                Value::Bytes(b) => String::from_utf8_lossy(&b).into_owned(),
                _ => String::new(),
            }).ok_or(FromRowError(row.clone())),
            row.get(6).map(|v: Value| match v {
                Value::Bytes(b) => String::from_utf8_lossy(&b).into_owned(),
                _ => String::new(),
            }).ok_or(FromRowError(row.clone())),
            row.get(7).map(|v: Value| match v {
                Value::Bytes(b) => String::from_utf8_lossy(&b).into_owned(),
                _ => String::new(),
            }).ok_or(FromRowError(row.clone())),
            row.get(8).map(|v: Value| match v { // Tratar Value::Null
                Value::Bytes(b) => Some(String::from_utf8_lossy(&b).into_owned()),
                Value::Null => None,
                _ => Some(String::new()),
            }).ok_or(FromRowError(row.clone())),
        );

        match result {
            (
                Ok(id),
                Ok(nome_cliente),
                Ok(nome_crianca),
                Ok(endereco_cliente),
                Ok(valor_mensalidade),
                Ok(escola_destino),
                Ok(endereco_escola),
                Ok(motorista_perua),
                Ok(telefone_cliente),
            ) => Ok(Cliente {
                id,
                nome_cliente,
                nome_crianca,
                endereco_cliente,
                valor_mensalidade,
                escola_destino,
                endereco_escola,
                motorista_perua,
                telefone_cliente,
            }),
            _ => Err(FromRowError(row)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NovoCliente {
    pub nome_cliente: String,
    pub nome_crianca: String,
    pub endereco_cliente: String,
    pub valor_mensalidade: f64,
    pub escola_destino: String,
    pub endereco_escola: String,
    pub motorista_perua: String,
    pub telefone_cliente: String,
}

pub fn get_connection() -> Result<PooledConn> {
    let url = "mysql://root:Gerudo64*@localhost:3306/perua_escolar";
    let pool = Pool::new(url)?;
    pool.get_conn()
}

pub fn listar_clientes(conn: &mut PooledConn) -> Result<Vec<Cliente>> {
    conn.query_map(
        "SELECT id, nome_cliente, nome_crianca, endereco_cliente, valor_mensalidade, escola_destino, endereco_escola, motorista_perua, telefone_cliente FROM clientes",
        |(id, nome_cliente, nome_crianca, endereco_cliente, valor_mensalidade, escola_destino, endereco_escola, motorista_perua, telefone_cliente_opt: Option<String>)| {
            Cliente {
                id,
                nome_cliente,
                nome_crianca,
                endereco_cliente,
                valor_mensalidade,
                escola_destino,
                endereco_escola,
                motorista_perua,
                telefone_cliente: telefone_cliente_opt,
            }
        },
    )
}

pub fn adicionar_cliente_db(conn: &mut PooledConn, cliente: &NovoCliente) -> Result<()> {
    conn.exec_drop(
        r"INSERT INTO clientes
        (nome_cliente, nome_crianca, endereco_cliente, valor_mensalidade, escola_destino, endereco_escola, motorista_perua, telefone_cliente)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        (
            &cliente.nome_cliente,
            &cliente.nome_crianca,
            &cliente.endereco_cliente,
            &cliente.valor_mensalidade,
            &cliente.escola_destino,
            &cliente.endereco_escola,
            &cliente.motorista_perua,
            &cliente.telefone_cliente,
        ),
    )
}

pub fn atualizar_cliente_db(conn: &mut PooledConn, cliente: &Cliente) -> Result<()> {
    conn.exec_drop(
        r"UPDATE clientes SET
        nome_cliente = ?,
        nome_crianca = ?,
        endereco_cliente = ?,
        valor_mensalidade = ?,
        escola_destino = ?,
        endereco_escola = ?,
        motorista_perua = ?,
        telefone_cliente = ?
        WHERE id = ?",
        (
            &cliente.nome_cliente,
            &cliente.nome_crianca,
            &cliente.endereco_cliente,
            &cliente.valor_mensalidade,
            &cliente.escola_destino,
            &cliente.endereco_escola,
            &cliente.motorista_perua,
            &cliente.telefone_cliente.as_deref().unwrap_or(""), // Trata Option para String
            &cliente.id,
        ),
    )
}

pub fn remover_cliente_db(conn: &mut PooledConn, id: i32) -> Result<()> {
    conn.exec_drop("DELETE FROM clientes WHERE id = ?", (id,))
}

pub fn buscar_cliente_db(conn: &mut PooledConn, id: i32) -> Result<Option<Cliente>> {
    let query = format!(
        "SELECT id, nome_cliente, nome_crianca, endereco_cliente, valor_mensalidade, escola_destino, endereco_escola, motorista_perua, telefone_cliente FROM clientes WHERE id = {}",
        id
    );

    conn.query_first::<(i32, String, String, String, f64, String, String, String, Option<String>)>(query)
        .map(|row_opt| {
            row_opt.map(|(id, nome_cliente, nome_crianca, endereco_cliente, valor_mensalidade, escola_destino, endereco_escola, motorista_perua, telefone_cliente)| {
                Cliente {
                    id,
                    nome_cliente,
                    nome_crianca,
                    endereco_cliente,
                    valor_mensalidade,
                    escola_destino,
                    endereco_escola,
                    motorista_perua,
                    telefone_cliente,
                }
            })
        })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usuario {
    pub id: i32,
    pub nome: String,
}