//! Servidor local de compatibilidade. Fonte: SQLite efêmero de cassettes.
use anti_ies::atender_http;
use iesei::instalacao::Instalacao;
use rusqlite::{Connection, OpenFlags};
use std::net::TcpListener;
use std::time::Duration;

type Fonte = (Connection, Instalacao, String);

fn fonte_da_cassette() -> Result<Option<Fonte>, Box<dyn std::error::Error>> {
    let caminho = std::env::var_os("ANTI_IES_CASSETTE_DB");
    let origem = std::env::var("ANTI_IES_CASSETTE_ORIGEM").ok();
    let cenario = std::env::var("ANTI_IES_CASSETTE_CENARIO").ok();
    match (caminho, origem, cenario) {
        (None, None, None) => Ok(None),
        (Some(caminho), Some(origem), Some(cenario)) if !cenario.is_empty() => {
            // Nunca usar diretamente o índice operacional com URLs de sessão.
            if std::path::Path::new(&caminho).file_name()
                == Some(std::ffi::OsStr::new("ies.sqlite3"))
            {
                return Err("o banco operacional do IES não é uma cassette".into());
            }
            let instalacao =
                Instalacao::interpretar(&origem).ok_or("origem de cassette inválida")?;
            let banco = Connection::open_with_flags(
                caminho,
                OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
            )?;
            Ok(Some((banco, instalacao, cenario)))
        }
        _ => Err("configuração incompleta da fonte anti-IES".into()),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut argumentos = std::env::args().skip(1);
    let porta = match (argumentos.next(), argumentos.next()) {
        (None, None) => 8765,
        (Some(texto), None) => texto.parse::<u16>()?,
        _ => return Err("uso: anti-ies [porta]".into()),
    };
    let fonte = fonte_da_cassette()?;
    let servidor = TcpListener::bind(("127.0.0.1", porta))?;
    eprintln!(
        "anti-IES experimental: http://{}/__anti_ies/health",
        servidor.local_addr()?
    );
    for conexao in servidor.incoming() {
        match conexao {
            Ok(mut fluxo) => {
                fluxo.set_read_timeout(Some(Duration::from_secs(3)))?;
                fluxo.set_write_timeout(Some(Duration::from_secs(3)))?;
                let acesso = fonte
                    .as_ref()
                    .map(|(banco, instalacao, cenario)| (banco, instalacao, cenario.as_str()));
                if let Err(erro) = atender_http(&mut fluxo, acesso) {
                    // Não registrar a requisição: pode conter URL assinada.
                    eprintln!("conexao encerrada: {erro}");
                }
            }
            Err(erro) => eprintln!("falha ao aceitar conexao: {erro}"),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use iesei::banco_cassettes::{ControleResumido, guardar, iniciar};
    use iesei::observacao::{TabelaObservada, UnidadeExibida};
    use std::io::{Read, Write};
    use std::net::TcpStream;

    #[derive(Clone, Copy)]
    enum BancoDoTeste {
        Nenhum,
        Vazio,
        Observado,
    }

    fn pedir(alvo: &str, tipo: BancoDoTeste) -> String {
        let escuta = TcpListener::bind(("127.0.0.1", 0)).expect("bind local");
        let endereco = escuta.local_addr().expect("endereco local");
        let servidor = std::thread::spawn(move || {
            let (mut fluxo, _) = escuta.accept().expect("aceitar conexao");
            let mut banco = Connection::open_in_memory().unwrap();
            iniciar(&banco).unwrap();
            let instalacao = Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
            if matches!(tipo, BancoDoTeste::Observado) {
                guardar(
                    &mut banco,
                    "ensaio",
                    &ControleResumido {
                        instalacao: instalacao.clone(),
                        unidade: UnidadeExibida::nova("TESTE").unwrap(),
                        recebidos: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
                        gerados: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
                    },
                )
                .unwrap();
            }
            let acesso =
                (!matches!(tipo, BancoDoTeste::Nenhum)).then_some((&banco, &instalacao, "ensaio"));
            atender_http(&mut fluxo, acesso).expect("responder");
        });
        let mut cliente = TcpStream::connect(endereco).expect("conectar");
        write!(cliente, "GET {alvo} HTTP/1.1\r\nHost: localhost\r\n\r\n").expect("enviar GET");
        cliente
            .shutdown(std::net::Shutdown::Write)
            .expect("encerrar escrita");
        let mut resposta = String::new();
        cliente.read_to_string(&mut resposta).expect("ler resposta");
        servidor.join().expect("servidor terminou");
        resposta
    }

    #[test]
    fn saude_responde_no_protocolo_http_real() {
        let resposta = pedir("/__anti_ies/health", BancoDoTeste::Nenhum);
        assert!(resposta.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(resposta.contains(r#"{"status":"ok","servidor":"anti-ies"}"#));
    }

    #[test]
    fn controlador_nao_implementado_responde_501_no_fio() {
        let resposta = pedir(
            "/sei/controlador.php?acao=procedimento_trabalhar",
            BancoDoTeste::Observado,
        );
        assert!(resposta.starts_with("HTTP/1.1 501 Not Implemented\r\n"));
    }

    #[test]
    fn controle_r_servido_pelo_banco_no_http_real() {
        let resposta = pedir(
            "/sei/controlador.php?acao=procedimento_controlar",
            BancoDoTeste::Observado,
        );
        assert!(resposta.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(resposta.contains("<a id='lnkInfraUnidade'>TESTE</a>"));
        assert!(resposta.contains("<caption>0 registros:</caption>"));
        assert!(resposta.contains("Content-Type: text/html; charset=utf-8"));
    }

    #[test]
    fn controle_sem_dados_nao_finge_lista_vazia() {
        let resposta = pedir(
            "/sei/controlador.php?acao=procedimento_controlar",
            BancoDoTeste::Vazio,
        );
        assert!(resposta.starts_with("HTTP/1.1 503 Service Unavailable\r\n"));
    }
}
