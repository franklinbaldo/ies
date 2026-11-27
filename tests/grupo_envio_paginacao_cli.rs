//! CLI real sobre hipótese sintética de paginação de grupos, não replay PEN1.
#![cfg(unix)]
use native_tls::TlsAcceptor;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Output};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[path = "aies_documento_criar_cli/fixture_tls.rs"]
mod fixture_tls;
use fixture_tls::{Pasta, identidade};
#[path = "support/falha.rs"]
mod support;
use support::falha;

const ORIGEM: &str = "https://sei.test/sei";

fn aceitar(escuta: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
    loop {
        match escuta.accept() {
            Ok((cliente, endereco)) => {
                assert!(endereco.ip().is_loopback());
                cliente
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                cliente
                    .set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                return Some(cliente);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if parar.load(Ordering::SeqCst) {
                    return None;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("aceitar: {e}"),
        }
    }
}

fn pedido_http(fonte: &mut impl Read) -> (Vec<u8>, String) {
    let mut cabecalho = Vec::new();
    let mut byte = [0u8];
    while !cabecalho.ends_with(b"\r\n\r\n") {
        assert!(cabecalho.len() < 16_384);
        fonte.read_exact(&mut byte).unwrap();
        cabecalho.push(byte[0]);
    }
    let texto = std::str::from_utf8(&cabecalho).unwrap();
    let primeira = texto.lines().next().unwrap();
    let metodo = primeira.split_whitespace().next().unwrap();
    let acao = primeira
        .split_once('?')
        .and_then(|(_, query)| query.split('&').find_map(|s| s.strip_prefix("acao=")))
        .unwrap_or("sem_acao");
    let evento = format!("{metodo} {acao}");
    let tamanho = texto
        .lines()
        .find_map(|linha| {
            let (chave, valor) = linha.split_once(':')?;
            chave
                .eq_ignore_ascii_case("content-length")
                .then(|| valor.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    assert!(tamanho < 64_000);
    let mut corpo = vec![0; tamanho];
    fonte.read_exact(&mut corpo).unwrap();
    cabecalho.extend(corpo);
    (cabecalho, evento)
}

fn executar_cli(
    pasta: &Pasta,
    arquivos: [&std::path::Path; 4],
    porta_proxy: u16,
    argumentos: &[&str],
) -> Output {
    let [controle, cookie, origem, cert] = arquivos;
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(argumentos)
        .env("IES_URL_CONTROLE_FILE", controle)
        .env("IES_COOKIE_FILE", cookie)
        .env("IES_ORIGEM_FILE", origem)
        .env("SSL_CERT_FILE", cert)
        .env("https_proxy", format!("http://127.0.0.1:{porta_proxy}"))
        .env("HOME", &pasta.0)
        .env_remove("HTTPS_PROXY")
        .env_remove("all_proxy")
        .env_remove("ALL_PROXY")
        .env_remove("NO_PROXY")
        .env_remove("no_proxy")
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_CREDENCIAIS_FILE")
        .output()
        .unwrap()
}

fn preparar_sessao() -> (
    Pasta,
    std::path::PathBuf,
    native_tls::Identity,
    std::path::PathBuf,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let pasta = Pasta::nova();
    let (cert, tls_id) = identidade(&pasta.0, "sei.test");
    let origem = pasta.privado("origem", ORIGEM);
    let hash_chave = ["infra", "hash"].join("_");
    let controle = pasta.privado("url", &format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_sistema=100000100&infra_unidade_atual=110000001&{hash_chave}={}",
        "a".repeat(64)
    ));
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    (pasta, cert, tls_id, origem, controle, cookie)
}

fn controle() -> String {
    format!(
        "<html><body><a id='lnkInfraUnidade'>TESTE</a>\
         <input id='hdnTipoVisualizacao' value='R'>\
         <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
         <table id='tblProcessosGerados'><caption>0 registros:</caption></table>\
         <a href='controlador.php?acao=grupo_unidade_listar&amp;infra_hash={}'>Grupos</a>\
         </body></html>",
        "a".repeat(64)
    )
}

fn pagina(indice: usize) -> String {
    let pos = indice + 1;
    let id = ["004", "6"][indice];
    let selecionado_0 = if indice == 0 { " selected" } else { "" };
    let selecionado_1 = if indice == 1 { " selected" } else { "" };
    let hash = "a".repeat(64);
    format!(
        "<html><body><a id='lnkInfraUnidade'>TESTE</a>\
         <form id='frmGrupoUnidadeLista' method='post' \
         action='controlador.php?acao=grupo_unidade_listar&amp;infra_hash={hash}'>\
         <input type='hidden' name='hdnInfraPaginaAtual' value='{indice}'>\
         <select id='selInfraPaginacaoSuperior' name='selInfraPaginacaoSuperior'>\
         <option value='0'{selecionado_0}>1</option><option value='1'{selecionado_1}>2</option></select>\
         <table><caption>Grupos de Envio (2 registros - {pos} a {pos}):</caption>\
         <tr><th></th><th>Nome</th><th>Descrição</th><th>Ações</th></tr>\
         <tr><td><input type='checkbox' value='{id}'></td><td>Mesmo</td>\
         <td>Descrição {id}</td><td><a href='controlador.php?acao=grupo_unidade_consultar&amp;id_grupo_unidade={id}&amp;infra_hash={hash}'>Consultar</a></td></tr>\
         </table>\
         <select id='selInfraPaginacaoInferior' name='selInfraPaginacaoInferior'>\
         <option value='0'{selecionado_0}>1</option><option value='1'{selecionado_1}>2</option></select>\
         </form></body></html>"
    )
}

fn consulta() -> &'static str {
    "<form id='frmGrupoCadastro'><input name='hdnIdGrupoUnidade' value='004'>\
     <input name='txtNome' value='Mesmo'><textarea name='txaDescricao'>Descrição 004</textarea>\
     <input name='hdnUnidades' value=''><select name='selUnidades'>\
     <option value='0011'>Homônimo</option><option value='12'>Homônimo</option>\
     </select></form>"
}

fn conferir_pagina(saida: &Output, pos: usize, id: &str) {
    assert_eq!(saida.status.code(), Some(0), "{saida:?}");
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "unidade": "TESTE",
            "origem": "web",
            "total_declarado": 2,
            "faixa_observada": {"inicio": pos, "fim": pos},
            "itens": [{"id": id, "nome": "Mesmo", "descricao": format!("Descrição {id}")}]
        })
    );
}

fn conferir_post_paginacao(texto: &str) {
    let (_, corpo) = texto.split_once("\r\n\r\n").unwrap();
    let mut campos: Vec<_> = corpo.split('&').collect();
    campos.sort_unstable();
    assert_eq!(
        campos,
        [
            "hdnInfraPaginaAtual=0",
            "selInfraPaginacaoInferior=0",
            "selInfraPaginacaoSuperior=0",
        ]
    );
}

#[test]
fn pagina_observada_opt_in_e_estado_lembrado_sem_restaurar() {
    let (pasta, cert, tls_id, origem, controle_arquivo, cookie) = preparar_sessao();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let sinal = Arc::clone(&parar);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(tls_id).unwrap();
        let mut pedidos = Vec::new();
        let mut indice = 1;
        while let Some(mut tcp) = aceitar(&proxy, &sinal) {
            let (connect, _) = pedido_http(&mut tcp);
            assert!(connect.starts_with(b"CONNECT sei.test:443 "));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (req, evento) = pedido_http(&mut canal);
            let texto = std::str::from_utf8(&req).unwrap();
            let html = match evento.as_str() {
                "GET procedimento_controlar" => controle(),
                "GET grupo_unidade_listar" => pagina(indice),
                "POST grupo_unidade_listar" => {
                    conferir_post_paginacao(texto);
                    indice = 0;
                    pagina(indice)
                }
                "GET grupo_unidade_consultar" => {
                    assert_eq!(indice, 0);
                    assert!(
                        texto
                            .lines()
                            .next()
                            .unwrap()
                            .contains("id_grupo_unidade=004")
                    );
                    consulta().to_owned()
                }
                _ => panic!("pedido inesperado: {evento}"),
            };
            pedidos.push(evento);
            write!(
                canal,
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{html}",
                html.len()
            )
            .unwrap();
            canal.flush().unwrap();
        }
        (pedidos, indice)
    });
    let arquivos = [
        controle_arquivo.as_path(),
        cookie.as_path(),
        origem.as_path(),
        cert.as_path(),
    ];
    let executar = |args: &[&str]| {
        let mut cmd = vec!["--json", "grupo-envio"];
        cmd.extend_from_slice(args);
        executar_cli(&pasta, arquivos, porta, &cmd)
    };
    let observada = executar(&["listar"]);
    let mesma = executar(&["listar", "--pagina", "2"]);
    let fora = executar(&["listar", "--pagina", "3"]);
    let oculto = executar(&["obter", "004"]);
    let primeira = executar(&["listar", "--pagina", "1"]);
    let lembrada = executar(&["listar"]);
    let grupo = executar(&["obter", "004"]);
    parar.store(true, Ordering::SeqCst);
    let (pedidos, indice) = tunel.join().unwrap();

    conferir_pagina(&observada, 2, "6");
    conferir_pagina(&mesma, 2, "6");
    falha(&fora, "PaginaNaoOferecida");
    falha(&oculto, "GrupoNaoEncontradoNaPagina");
    conferir_pagina(&primeira, 1, "004");
    conferir_pagina(&lembrada, 1, "004");
    assert_eq!(grupo.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&grupo.stdout).unwrap();
    assert_eq!(json["grupo"]["id"], "004");
    assert_eq!(
        json["grupo"]["unidades"],
        serde_json::json!([{"id":"0011","nome":"Homônimo"},{"id":"12","nome":"Homônimo"}])
    );
    let mut esperados = Vec::new();
    for i in 0..7 {
        esperados.extend(["GET procedimento_controlar", "GET grupo_unidade_listar"]);
        if i == 4 {
            esperados.push("POST grupo_unidade_listar");
        } else if i == 6 {
            esperados.push("GET grupo_unidade_consultar");
        }
    }
    assert_eq!(pedidos, esperados);
    assert_eq!(indice, 0);
}
