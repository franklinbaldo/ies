//! Contraprova sintética da identidade da unidade na CLI real (#880).
use super::*;

fn controle() -> String {
    format!(
        "<html><body><a id='lnkInfraUnidade'>TESTE</a>\
         <input id='hdnTipoVisualizacao' value='R'>\
         <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
         <table id='tblProcessosGerados'><caption>0 registros:</caption></table>\
         <a href='controlador.php?acao=marcador_listar&amp;infra_hash={}'>Marcadores</a>\
         </body></html>",
        "a".repeat(64)
    )
}

fn catalogo(unidade: Option<&str>, legenda: &str) -> String {
    let barra = unidade.map_or_else(String::new, |u| format!("<a id='lnkInfraUnidade'>{u}</a>"));
    format!(
        "<html><body>{barra}<table summary='Tabela de Marcadores.'>\
         <caption>Lista de Marcadores ({legenda}):</caption>\
         <tr><th>ID</th><th>Nome</th><th>Ícone</th><th>Ações</th></tr>\
         <tr><td>501</td><td>Revisão</td><td><img title='Branco'></td>\
         <td><img title='Desativar Marcador'></td></tr></table></body></html>"
    )
}

fn executar_catalogo(
    unidade: Option<&'static str>,
    legenda: &'static str,
    argumentos: &[&str],
) -> (Output, Vec<String>) {
    let (pasta, cert, tls_id, origem, controle_arquivo, cookie) = preparar_sessao();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let sinal = Arc::clone(&parar);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(tls_id).unwrap();
        let mut pedidos = Vec::new();
        while let Some(mut tcp) = aceitar(&proxy, &sinal) {
            let (connect, _) = pedido_http(&mut tcp);
            assert!(connect.starts_with(b"CONNECT"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (_, evento) = pedido_http(&mut canal);
            let html = match evento.as_str() {
                "GET procedimento_controlar" => controle(),
                "GET marcador_listar" => catalogo(unidade, legenda),
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
        pedidos
    });
    let saida = executar_cli(
        &pasta,
        [
            controle_arquivo.as_path(),
            cookie.as_path(),
            origem.as_path(),
            cert.as_path(),
        ],
        porta,
        argumentos,
    );
    parar.store(true, Ordering::SeqCst);
    (saida, tunel.join().unwrap())
}

#[test]
fn catalogo_publicado_de_outra_unidade_nao_sai_como_unidade_do_controle() {
    // A mesma tabela válida, mudando exclusivamente a barra de unidade.
    for unidade in [None, Some("TESTE"), Some("OUTRA")] {
        let (saida, pedidos) =
            executar_catalogo(unidade, "1 registro", &["--json", "marcador", "listar"]);
        assert!(pedidos.iter().any(|p| p == "GET procedimento_controlar"));
        assert!(pedidos.iter().any(|p| p == "GET marcador_listar"));
        assert!(!pedidos.iter().any(|p| p.starts_with("POST ")));
        if unidade == Some("OUTRA") {
            falha(&saida, "UnidadeInesperada");
        } else {
            assert_eq!(saida.status.code(), Some(0));
            let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
            assert_eq!(json["unidade"], "TESTE");
            assert_eq!(json["total_declarado"], 1);
            assert_eq!(json["total_observado"], 1);
            assert_eq!(
                json["faixa_observada"],
                serde_json::json!({"inicio": 1, "fim": 1})
            );
            assert_eq!(json["estado"], "ativo");
            assert_eq!(
                json["itens"],
                serde_json::json!([{
                    "id": "501", "nome": "Revisão", "cor": "Branco", "estado": "ativo"
                }])
            );
        }
    }
}

#[test]
fn texto_distingue_faixa_e_estado_mesmo_sem_itens_selecionados() {
    for (legenda, faixa, argumentos, estado) in [
        (
            "4 registros - 2 a 2",
            "2 a 2",
            &["marcador", "listar", "--inativos"][..],
            "inativo",
        ),
        (
            "4 registros - 3 a 3",
            "3 a 3",
            &["marcador", "listar", "--inativos"][..],
            "inativo",
        ),
        (
            "4 registros - 2 a 2",
            "2 a 2",
            &["marcador", "listar"][..],
            "ativo",
        ),
    ] {
        let (saida, pedidos) = executar_catalogo(None, legenda, argumentos);
        assert_eq!(saida.status.code(), Some(0));
        assert!(!pedidos.iter().any(|p| p.starts_with("POST ")));
        let texto = String::from_utf8(saida.stdout).unwrap();
        assert!(texto.contains(&format!("Estado solicitado: {estado}")));
        assert!(texto.contains(&format!("Faixa observada: {faixa}")));
        assert_eq!(texto.contains("501#Revisão"), estado == "ativo");
    }
}

/// Hipótese sintética complementar: binário real, rota HTTP e banco compartilhado.
/// As páginas originais são confrontadas separadamente no replay cassette_.
#[test]
fn catalogo_sqlite_selecao_explicita_chega_ao_binario() {
    use iesei::gestao_marcadores::{self as gestao, Estado, Marcador, Pagina};

    for ordinal in [1, 2] {
        let (pasta, cert, tls_id, origem, controle, cookie) = preparar_sessao();
        let backend = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        backend.set_nonblocking(true).unwrap();
        let endereco = backend.local_addr().unwrap();
        let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        proxy.set_nonblocking(true).unwrap();
        let porta = proxy.local_addr().unwrap().port();
        let parar = Arc::new(AtomicBool::new(false));
        let sinal = Arc::clone(&parar);
        let servidor = thread::spawn(move || {
            let (mut banco, instalacao) = estado(false, false);
            banco
                .execute_batch(include_str!(
                    "../../crates/iesei/migrations/0043_gestao_marcadores.sql"
                ))
                .unwrap();
            let mista = Pagina {
                total: 4,
                faixa: Some((2, 3)),
                itens: vec![
                    Marcador {
                        id: "01".into(),
                        nome: "A & <B>".into(),
                        cor: "Branco".into(),
                        estado: Estado::Ativo,
                    },
                    Marcador {
                        id: "02".into(),
                        nome: "A & <B>".into(),
                        cor: "Amarelo".into(),
                        estado: Estado::Inativo,
                    },
                ],
            };
            let vazia = Pagina {
                total: 0,
                faixa: None,
                itens: vec![],
            };
            gestao::registrar(&mut banco, &instalacao, CENARIO, "TESTE", 1, &mista).unwrap();
            gestao::registrar(&mut banco, &instalacao, CENARIO, "TESTE", 2, &vazia).unwrap();
            assert!(gestao::selecionar(&banco, &instalacao, CENARIO, "TESTE", ordinal).unwrap());
            while let Some(mut cliente) = aceitar(&backend, &sinal) {
                atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
            }
            assert_eq!(
                gestao::ler(&banco, &instalacao, CENARIO, "TESTE", 1).unwrap(),
                Some(mista.clone())
            );
            assert_eq!(
                gestao::ler(&banco, &instalacao, CENARIO, "TESTE", 2).unwrap(),
                Some(vazia.clone())
            );
            assert_eq!(
                gestao::ler_selecionada(&banco, &instalacao, CENARIO, "TESTE").unwrap(),
                Some(if ordinal == 1 { mista } else { vazia })
            );
        });
        let sinal = Arc::clone(&parar);
        let tunel = thread::spawn(move || {
            let tls = TlsAcceptor::new(tls_id).unwrap();
            let mut pedidos = Vec::new();
            while let Some(mut tcp) = aceitar(&proxy, &sinal) {
                let (connect, _) = pedido_http(&mut tcp);
                assert!(connect.starts_with(b"CONNECT"));
                tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .unwrap();
                let mut canal = tls.accept(tcp).unwrap();
                let (req, evento) = pedido_http(&mut canal);
                pedidos.push(evento);
                let mut destino = TcpStream::connect(endereco).unwrap();
                destino.write_all(&req).unwrap();
                destino.shutdown(Shutdown::Write).unwrap();
                std::io::copy(&mut destino, &mut canal).unwrap();
                canal.flush().unwrap();
            }
            pedidos
        });
        let arquivos = [
            controle.as_path(),
            cookie.as_path(),
            origem.as_path(),
            cert.as_path(),
        ];
        let ativo = executar_cli(&pasta, arquivos, porta, &["--json", "marcador", "listar"]);
        let inativo = executar_cli(
            &pasta,
            arquivos,
            porta,
            &["--json", "marcador", "listar", "--inativos"],
        );
        parar.store(true, Ordering::SeqCst);
        let pedidos = tunel.join().unwrap();
        servidor.join().unwrap();
        assert_eq!(
            pedidos,
            [
                "GET procedimento_controlar",
                "GET marcador_listar",
                "GET procedimento_controlar",
                "GET marcador_listar"
            ]
        );
        for (saida, estado, id, cor) in [
            (ativo, "ativo", "01", "Branco"),
            (inativo, "inativo", "02", "Amarelo"),
        ] {
            assert_eq!(saida.status.code(), Some(0));
            let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
            assert_eq!(json["unidade"], "TESTE");
            assert_eq!(json["estado"], estado);
            if ordinal == 1 {
                assert_eq!(json["total_declarado"], 4);
                assert_eq!(json["total_observado"], 2);
                assert_eq!(
                    json["faixa_observada"],
                    serde_json::json!({"inicio": 2, "fim": 3})
                );
                assert_eq!(
                    json["itens"],
                    serde_json::json!([{
                        "id": id, "nome": "A & <B>", "cor": cor, "estado": estado
                    }])
                );
            } else {
                assert_eq!(json["total_declarado"], 0);
                assert_eq!(json["total_observado"], 0);
                assert!(json.get("faixa_observada").is_none());
                assert_eq!(json["itens"], serde_json::json!([]));
            }
        }
    }
}
