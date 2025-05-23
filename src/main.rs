use actix_web::{web, App, HttpServer, HttpResponse, Responder, Result};
use actix_session::{SessionMiddleware, Session, storage::CookieSessionStore};
use actix_web::cookie::Key;
use tera::{Tera, Context};
use mysql::Pool;
use serde::Deserialize;

use rust_decimal::Decimal;

mod auth;
use auth::{LoginForm, autenticar_usuario};

mod clientes;
use clientes::{listar_clientes, adicionar_cliente, buscar_cliente_por_id, atualizar_cliente, remover_cliente};

mod auth_error;

mod middleware;
use middleware::{AuthAdmin, AuthMiddleware};

mod models;
use models::{FormCliente, Cliente};

use actix_files as fs; // <-- ADICIONE ESTA LINHA NOVAMENTE!

#[derive(serde::Serialize)]
struct UserTemplate {
    username: String,
    is_admin: bool,
}

// Struct para pegar o parâmetro de pesquisa da URL
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    search: Option<String>,
}

// *******************************************************************
// FUNÇÃO DE FILTRO PERSONALIZADA PARA O TERA (mantida inalterada)
// *******************************************************************
fn format_currency_filter(value: &tera::Value, _: &std::collections::HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    let decimal_num = match value {
        tera::Value::Number(n) => {
            let s = n.to_string();
            s.parse::<Decimal>()
                .map_err(|e| tera::Error::msg(format!("Failed to parse Decimal from number value: {}", e)))?
        },
        tera::Value::String(s) => {
            s.parse::<Decimal>()
                .map_err(|e| tera::Error::msg(format!("Failed to parse Decimal from string: {}", e)))?
        },
        _ => return Err(tera::Error::msg(format!("Filter `format_currency` only works on numbers or string representations of numbers, got {:?}", value))),
    };

    let formatted_value = format!("R$ {}", decimal_num.to_string().replace(".", ","));
    
    let parts: Vec<&str> = formatted_value.split(',').collect();
    let integer_part = parts[0].replace("R$ ", "");
    let mut formatted_integer_part_rev = String::new();
    let mut count = 0;
    for c in integer_part.chars().rev() {
        if count == 3 {
            formatted_integer_part_rev.push('.');
            count = 0;
        }
        formatted_integer_part_rev.push(c);
        count += 1;
    }
    let final_integer_part: String = formatted_integer_part_rev.chars().rev().collect();
    
    let final_formatted_value = if parts.len() > 1 {
        format!("R$ {},{}", final_integer_part, parts[1])
    } else {
        format!("R$ {}", final_integer_part)
    };
    
    Ok(tera::to_value(final_formatted_value).unwrap())
}


async fn login_form(tmpl: web::Data<Tera>, session: Session) -> Result<HttpResponse> {
    if let Ok(Some(_username)) = session.get::<String>("user") {
        return Ok(HttpResponse::Found().append_header(("Location", "/")).finish());
    }

    let s = tmpl.render("login.html", &Context::new())
        .map_err(|_| actix_web::error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

async fn login(
    pool: web::Data<Pool>,
    form: web::Form<LoginForm>,
    session: Session,
    tmpl: web::Data<Tera>,
) -> Result<HttpResponse> {
    println!("Tentativa de login: username={}", form.username);

    let mut conn = pool.get_conn().map_err(|e| {
        eprintln!("Erro conexão DB: {:?}", e);
        actix_web::error::ErrorInternalServerError("DB error")
    })?;

    match autenticar_usuario(&mut conn, &form.username, &form.password) {
        Ok(Some(user)) => {
            println!("Usuário autenticado: {}", user.username);
            if let Err(e) = session.insert("user", user.username.clone()) {
                eprintln!("Erro ao inserir user na sessão: {:?}", e);
                return Err(actix_web::error::ErrorInternalServerError("Erro de sessão"));
            }
            if let Err(e) = session.insert("is_admin", user.is_admin) {
                eprintln!("Erro ao inserir is_admin na sessão: {:?}", e);
                return Err(actix_web::error::ErrorInternalServerError("Erro de sessão"));
            }
            Ok(HttpResponse::Found().append_header(("Location", "/")).finish())
        }
        Ok(None) => {
            println!("Usuário ou senha inválidos");
            let mut ctx = Context::new();
            ctx.insert("login_error", &true);
            let s = tmpl.render("login.html", &ctx)
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            Ok(HttpResponse::Ok().content_type("text/html").body(s))
        }
        Err(e) => {
            eprintln!("Erro no login: {:?}", e);
            Err(actix_web::error::ErrorInternalServerError("DB error"))
        }
    }
}

async fn logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::Found().append_header(("Location", "/login")).finish()
}

// *Handlers* de rota após a aplicação do middleware
async fn home(
    tmpl: web::Data<Tera>,
    pool: web::Data<Pool>,
    session: Session, // Session ainda é útil para obter dados do usuário logado
    query: web::Query<SearchParams>, // <--- Adicionado para pegar o parâmetro 'search'
) -> Result<HttpResponse> {
    // Esses unwrap são seguros devido ao AuthMiddleware
    let username = session.get::<String>("user")?.unwrap();
    let is_admin = session.get::<bool>("is_admin")?.unwrap_or(false);

    let mut ctx = Context::new();
    ctx.insert("current_user", &UserTemplate {
        username,
        is_admin,
    });

    let search_term = query.search.clone().filter(|s| !s.trim().is_empty()); // Pega o termo de pesquisa e remove espaços em branco

    let clientes: Vec<Cliente> = listar_clientes(&pool, search_term.clone()) // <--- Passa o termo de pesquisa
        .unwrap_or_default();
    ctx.insert("has_clientes", &!clientes.is_empty());
    ctx.insert("clientes", &clientes);
    ctx.insert("search_query", &search_term); // <--- Passa o termo de pesquisa para o template

    let s = tmpl.render("index.html", &ctx)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

async fn form_adicionar(
    tmpl: web::Data<Tera>,
    session: Session, // Session ainda é útil para obter dados do usuário logado
) -> Result<HttpResponse> {
    // Esses unwrap são seguros devido ao AuthAdmin middleware
    let username = session.get::<String>("user")?.unwrap();
    let is_admin = session.get::<bool>("is_admin")?.unwrap_or(false);

    let mut ctx = Context::new();
    ctx.insert("current_user", &UserTemplate {
        username,
        is_admin,
    });
        
    let s = tmpl.render("adicionar.html", &ctx)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

async fn post_adicionar(
    pool: web::Data<Pool>,
    form: web::Form<FormCliente>,
) -> Result<HttpResponse> {
    // A verificação de admin já foi feita pelo middleware
    match adicionar_cliente(&pool, &form) {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/")).finish()),
        Err(e) => {
            eprintln!("Erro ao adicionar cliente: {:?}", e);
            Err(actix_web::error::ErrorInternalServerError("Erro ao adicionar cliente"))
        }
    }
}

async fn form_editar(
    tmpl: web::Data<Tera>,
    pool: web::Data<Pool>,
    path: web::Path<u32>,
    session: Session, // Session ainda é útil para obter dados do usuário logado
) -> Result<HttpResponse> {
    // Esses unwrap são seguros devido ao AuthAdmin middleware
    let username = session.get::<String>("user")?.unwrap();
    let is_admin = session.get::<bool>("is_admin")?.unwrap_or(false);

    let id = path.into_inner();

    match buscar_cliente_por_id(&pool, id) {
        Ok(Some(cliente)) => {
            let mut ctx = Context::new();
            ctx.insert("cliente", &cliente);
            ctx.insert("current_user", &UserTemplate {
                username,
                is_admin,
            });

            let s = tmpl.render("editar.html", &ctx)
                .map_err(|_| actix_web::error::ErrorInternalServerError("Template error"))?;
            Ok(HttpResponse::Ok().content_type("text/html").body(s))
        },
        Ok(None) => Ok(HttpResponse::NotFound().body("Cliente não encontrado")),
        Err(_) => Err(actix_web::error::ErrorInternalServerError("Erro ao buscar cliente")),
    }
}

async fn post_editar(
    pool: web::Data<Pool>,
    path: web::Path<u32>,
    form: web::Form<FormCliente>,
) -> Result<HttpResponse> {
    // A verificação de admin já foi feita pelo middleware
    let id = path.into_inner();

    match atualizar_cliente(&pool, id, &form) {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/")).finish()),
        Err(_) => Err(actix_web::error::ErrorInternalServerError("Erro ao atualizar cliente")),
    }
}

async fn post_remover(
    pool: web::Data<Pool>,
    path: web::Path<u32>,
) -> Result<HttpResponse> {
    // A verificação de admin já foi feita pelo middleware
    let id = path.into_inner();

    match remover_cliente(&pool, id) {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/")).finish()),
        Err(_) => Err(actix_web::error::ErrorInternalServerError("Erro ao remover cliente")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Inicia o logger
    env_logger::init();
    
    // ATENÇÃO: Use dotenv para carregar as variáveis de ambiente em produção!
    // Para homologação, você pode ter um .env
    // dotenv::dotenv().ok(); // Carrega variáveis do arquivo .env

    // let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // let session_secret_key = std::env::var("SESSION_SECRET_KEY").expect("SESSION_SECRET_KEY must be set");
    
    let pool = Pool::new("mysql://root:Gerudo64*@localhost/perua_escolar").expect("Failed to create pool");
    
    let mut tera = Tera::new("templates/**/*").expect("Failed to parse templates");
    tera.autoescape_on(vec![".html", ".htm"]);
    
    tera.register_filter("format_currency", format_currency_filter);

    let secret_key = Key::generate(); // Ou use Key::from(session_secret_key.as_bytes()) se vier de .env

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(tera.clone()))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .service(fs::Files::new("/static", "./static"))		
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .route("/logout", web::post().to(logout))
            // Rotas que exigem apenas que o usuário esteja logado (HOME)
            .service(
                web::scope("/")
                    .wrap(AuthMiddleware) // Aplica o middleware de autenticação
                    .route("", web::get().to(home)) // Rota raiz (home)
            )
            // Rotas que exigem que o usuário seja administrador
            .service(
                web::scope("/admin") // Todas as rotas dentro deste escopo exigirão admin
                    .wrap(AuthAdmin) // Aplica o middleware de autenticação e admin
                    .route("/adicionar", web::get().to(form_adicionar))
                    .route("/adicionar", web::post().to(post_adicionar))
                    .route("/editar/{id}", web::get().to(form_editar))
                    .route("/editar/{id}", web::post().to(post_editar))
                    .route("/remover/{id}", web::post().to(post_remover))
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}