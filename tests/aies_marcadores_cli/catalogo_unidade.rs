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

fn catalogo(unidade: Option<&str>) -> String {
    let barra = unidade.map_or_else(String::new, |u| {
        format!("<a id='lnkInfraUnidade'>{u}</a>")
    });
    format!(
        "<html><body>{barra}<table summary='Tabela de Marcadores.'>\
         <caption>Lista de Marcadores (1 registro):</caption>\
         <tr><th>ID</th><th>Nome</th><th>Ícone</th><th>Ações</th></tr>\
         <tr><td>501</td><td>Revisão</td><td><img title='Branco'></td>\
         <td><img title='Desativar Marcador'></td></tr></table></body></html>"
    )
}

fn executar_catalogo(unidade: Option<&'static str>) -> (Output, Vec<String>) {
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
                "GET marcador_listar" => catalogo(unidade),
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
        &["--json", "marcador", "listar"],
    );
    parar.store(true, Ordering::SeqCst);
    (saida, tunel.join().unwrap())
}

#[test]
fn catalogo_publicado_de_outra_unidade_nao_sai_como_unidade_do_controle() {
    // A mesma tabela válida, mudando exclusivamente a barra de unidade.
    for unidade in [None, Some("TESTE"), Some("OUTRA")] {
        let (saida, pedidos) = executar_catalogo(unidade);
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
            assert_eq!(json["itens"][0]["id"], "501");
            assert_eq!(json["itens"][0]["estado"], "ativo");
        }
    }
}
