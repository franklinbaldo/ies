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
         <td><img title='Desativar Marcador'></td></tr></table>\
         <button type='button' id='btnNovo' onclick=\"location.href='controlador.php?acao=marcador_cadastrar&amp;infra_hash={hash}';\">Novo</button>\
         <button type='button' id='btnNovo' onclick=\"location.href='controlador.php?acao=marcador_cadastrar&amp;infra_hash={hash}';\">Novo</button></body></html>",
        hash = "b".repeat(64)
    )
}

fn executar_catalogo(
    unidade: Option<&'static str>,
    legenda: &'static str,
    argumentos: &[&str],
) -> (Output, Vec<String>) {
    executar_com_formulario(unidade, legenda, "TESTE", "", true, argumentos)
}

fn executar_com_formulario(
    unidade: Option<&'static str>,
    legenda: &'static str,
    unidade_formulario: &'static str,
    opcoes_adicionais: &'static str,
    sessao_presente: bool,
    argumentos: &[&str],
) -> (Output, Vec<String>) {
    let (pasta, cert, tls_id, origem, controle_arquivo, cookie) = preparar_sessao();
    if !sessao_presente {
        for arquivo in [&origem, &controle_arquivo, &cookie] {
            std::fs::remove_file(arquivo).unwrap();
            assert!(!arquivo.exists());
        }
    }
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
                "GET marcador_cadastrar" => formulario_novo(unidade_formulario, opcoes_adicionais),
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

fn formulario_novo(unidade: &str, opcoes_adicionais: &str) -> String {
    format!(
        "<a id='lnkInfraUnidade'>{unidade}</a>\
         <form id='frmMarcadorCadastro' method='post' action='controlador.php?acao=marcador_cadastrar&amp;infra_hash={}'>\
         <input name='txtNome'><textarea name='txaDescricao'></textarea>\
         <input type='hidden' name='hdnStaIcone'>\
         <select name='selStaIcone'><option value='null'> </option>\
         <option value='2'>Cinza</option><option value='2'>Cinza Escuro</option>\
         <option value='0'>Preto</option>{opcoes_adicionais}</select>\
         <button name='sbmCadastrarMarcador' type='submit'>Salvar</button></form>",
        "c".repeat(64)
    )
}

#[test]
fn previa_criar_marcador_le_formulario_sem_post() {
    for (cor, codigo) in [("Cinza Escuro", "2"), ("Preto", "0")] {
        let (saida, pedidos) = executar_catalogo(
            Some("TESTE"),
            "1 registro",
            &[
                "--json",
                "marcador",
                "criar",
                "--nome",
                "Novo",
                "--cor",
                cor,
                "--descricao",
                "Conferência",
            ],
        );
        assert_eq!(saida.status.code(), Some(0), "{:?}", saida.stderr);
        assert_eq!(
            pedidos,
            [
                "GET procedimento_controlar",
                "GET marcador_listar",
                "GET marcador_cadastrar",
            ]
        );
        let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "unidade": "TESTE", "nome": "Novo", "cor": cor,
                "codigo_cor": codigo, "descricao": "Conferência",
                "previa": true, "criado": false, "requisicoes": 3,
            })
        );
    }
}

#[test]
fn previa_criar_marcador_recusa_confirmacao_e_cor_ausente() {
    // A recusa deve anteceder até a leitura das referências inexistentes.
    let (saida, pedidos) = executar_com_formulario(
        Some("TESTE"),
        "1 registro",
        "TESTE",
        "",
        false,
        &[
            "--json",
            "marcador",
            "criar",
            "--nome",
            "Novo",
            "--cor",
            "Preto",
            "--confirmar",
        ],
    );
    falha(&saida, "ConfirmacaoIndisponivel");
    assert!(pedidos.is_empty());
    let (saida, pedidos) = executar_catalogo(
        Some("TESTE"),
        "1 registro",
        &[
            "--json",
            "marcador",
            "criar",
            "--nome",
            "Novo",
            "--cor",
            "Inexistente",
        ],
    );
    falha(&saida, "CorNaoOferecidaOuAmbigua");
    assert_eq!(
        pedidos,
        [
            "GET procedimento_controlar",
            "GET marcador_listar",
            "GET marcador_cadastrar",
        ]
    );

    let (saida, pedidos) = executar_com_formulario(
        Some("TESTE"),
        "1 registro",
        "OUTRA",
        "",
        true,
        &[
            "--json", "marcador", "criar", "--nome", "Novo", "--cor", "Preto",
        ],
    );
    falha(&saida, "UnidadeInesperada");
    assert_eq!(
        pedidos,
        [
            "GET procedimento_controlar",
            "GET marcador_listar",
            "GET marcador_cadastrar",
        ]
    );
}

#[test]
fn previa_criar_marcador_recusa_rotulo_duplicado_mesmo_com_valor_invalido() {
    for opcao in [
        "<option>Preto</option>",
        "<option value=''>Preto</option>",
        "<option value='null'>Preto</option>",
        "<option value='0'>Preto</option>",
    ] {
        let (saida, pedidos) = executar_com_formulario(
            Some("TESTE"),
            "1 registro",
            "TESTE",
            opcao,
            true,
            &[
                "--json", "marcador", "criar", "--nome", "Novo", "--cor", "Preto",
            ],
        );
        falha(&saida, "CorNaoOferecidaOuAmbigua");
        assert_eq!(
            pedidos,
            [
                "GET procedimento_controlar",
                "GET marcador_listar",
                "GET marcador_cadastrar",
            ]
        );
    }
}

#[test]
fn previa_criar_marcador_texto_nao_promete_escrita() {
    let (saida, pedidos) = executar_catalogo(
        Some("TESTE"),
        "1 registro",
        &[
            "marcador",
            "criar",
            "--nome",
            "Novo",
            "--cor",
            "Preto",
            "--descricao",
            "Conferência",
        ],
    );
    assert_eq!(saida.status.code(), Some(0), "{:?}", saida.stderr);
    assert_eq!(
        pedidos,
        [
            "GET procedimento_controlar",
            "GET marcador_listar",
            "GET marcador_cadastrar",
        ]
    );
    let texto = String::from_utf8(saida.stdout).unwrap();
    assert!(texto.contains("Prévia: criar marcador Novo na unidade TESTE"));
    assert!(texto.contains("cor Preto (código 0)"));
    assert!(texto.contains("Descrição: Conferência"));
    assert!(texto.contains("Nenhum marcador criado; confirmação indisponível."));
}
