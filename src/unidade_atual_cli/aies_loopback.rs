//! #355: hipótese sintética AIES-01/03, HTTP real em loopback e parser da CLI.
//! O transporte TLS de produção permanece intocado; não é E2E do binário.
use super::unidade_validada;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

fn consultar_controle(com_snapshot: bool) -> String {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).expect("porta efêmera local");
    let endereco = escuta.local_addr().expect("endereço local");
    let servidor = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().expect("SQLite efêmero");
        banco_cassettes::iniciar(&banco).expect("esquema versionado");
        let instalacao = Instalacao::interpretar("https://sei.exemplo/sei")
            .expect("instalação somente sintética");
        if com_snapshot {
            banco_cassettes::guardar(
                &mut banco,
                "aies-01",
                &ControleResumido {
                    instalacao: instalacao.clone(),
                    unidade: UnidadeExibida::nova("TESTE").expect("unidade válida"),
                    recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![])
                        .expect("lista declarada vazia"),
                    gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![])
                        .expect("lista declarada vazia"),
                },
            )
            .expect("guardar observação sintética");
        }
        let (mut conexao, _) = escuta.accept().expect("aceitar única conexão");
        anti_ies::atender_http(&mut conexao, Some((&banco, &instalacao, "aies-01")))
            .expect("responder consulta");
    });
    let mut cliente = TcpStream::connect(endereco).expect("conectar ao servidor local");
    // URLs de sessão de Controle carregam parâmetros além de acao.
    // Não incluir credenciais, tokens ou hashes na fixture.
    cliente
        .write_all(
            b"GET /sei/controlador.php?acao=procedimento_controlar&infra_unidade_atual=7 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        )
        .expect("um GET");
    cliente.shutdown(Shutdown::Write).expect("fim do pedido");
    let mut resposta = String::new();
    cliente
        .read_to_string(&mut resposta)
        .expect("resposta única");
    servidor.join().expect("thread HTTP");
    resposta
}

#[test]
fn aies_01_controle_do_sqlite_chega_ao_leitor_da_cli_por_http() {
    let resposta = consultar_controle(true);
    assert!(
        resposta.starts_with("HTTP/1.1 200 OK\r\n"),
        "status inesperado: {}",
        resposta.lines().next().unwrap_or("")
    );
    let (cabecalho, html) = resposta.split_once("\r\n\r\n").expect("resposta HTTP");
    assert!(cabecalho.contains("Content-Type: text/html; charset=utf-8"));
    let unidade = unidade_validada(html).expect("Controle R reconhecido pelo IES");
    assert_eq!(unidade, "TESTE");
    let json = super::dados_publicos_unidade(&unidade);
    assert_eq!(json["unidade_exibida"], "TESTE");
    assert_eq!(json["origem"], "controle_de_processos");
}

#[test]
fn aies_03_snapshot_ausente_nao_vira_unidade_ficticia() {
    let resposta = consultar_controle(false);
    assert!(resposta.starts_with("HTTP/1.1 503 Service Unavailable\r\n"));
    assert!(resposta.contains("observacao_insuficiente"));
    assert!(!resposta.contains("lnkInfraUnidade"));
    assert!(!resposta.contains("TESTE"));
}
