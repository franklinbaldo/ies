//! Listener HTTP local, exclusivo para ensaios offline de compatibilidade.
//! Não possui autenticação, persistência nem operações SEI implementadas.
use anti_ies::{Estado, Resposta, despachar};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut argumentos = std::env::args().skip(1);
    let porta = match (argumentos.next(), argumentos.next()) {
        (None, None) => 8765,
        (Some(texto), None) => texto.parse::<u16>()?,
        _ => return Err("uso: anti-ies [porta]".into()),
    };

    // A fatia inicial nunca expõe o listener em uma interface pública.
    let servidor = TcpListener::bind(("127.0.0.1", porta))?;
    eprintln!(
        "anti-IES experimental: http://{}/__anti_ies/health",
        servidor.local_addr()?
    );

    for conexao in servidor.incoming() {
        match conexao {
            Ok(mut fluxo) => {
                if let Err(erro) = atender(&mut fluxo) {
                    // Não registrar o pedido: pode conter URLs assinadas.
                    eprintln!("conexao encerrada: {erro}");
                }
            }
            Err(erro) => eprintln!("falha ao aceitar conexao: {erro}"),
        }
    }

    Ok(())
}

fn atender(fluxo: &mut TcpStream) -> io::Result<()> {
    fluxo.set_read_timeout(Some(Duration::from_secs(3)))?;
    fluxo.set_write_timeout(Some(Duration::from_secs(3)))?;

    // Apenas a linha inicial, limitada em bytes. Nenhuma ação usa headers,
    // cookies, corpo, sessão ou estado do cliente nesta fatia.
    let mut primeira_linha = Vec::new();
    let lidos = BufReader::new(&mut *fluxo)
        .take(4096)
        .read_until(b'\n', &mut primeira_linha)?;
    if lidos == 0 {
        return Ok(());
    }

    let resposta = if primeira_linha.ends_with(b"\n") {
        let texto = std::str::from_utf8(&primeira_linha).unwrap_or_default();
        let mut partes = texto.split_ascii_whitespace();
        match (partes.next(), partes.next(), partes.next(), partes.next()) {
            (Some(metodo), Some(alvo), Some("HTTP/1.1" | "HTTP/1.0"), None) => {
                despachar(metodo, alvo)
            }
            _ => Resposta::nova(Estado::RequisicaoInvalida),
        }
    } else {
        Resposta::nova(Estado::RequisicaoInvalida)
    };

    write!(
        fluxo,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n\r\n{}",
        resposta.estado.codigo_http(),
        resposta.estado.motivo_http(),
        resposta.tipo_conteudo,
        resposta.corpo.len(),
        resposta.corpo
    )?;
    fluxo.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pedir(alvo: &str) -> String {
        let escuta = TcpListener::bind(("127.0.0.1", 0)).expect("bind local");
        let endereco = escuta.local_addr().expect("endereco local");
        let servidor = std::thread::spawn(move || {
            let (mut fluxo, _) = escuta.accept().expect("aceitar conexao");
            atender(&mut fluxo).expect("responder");
        });

        let mut cliente = TcpStream::connect(endereco).expect("conectar");
        write!(cliente, "GET {alvo} HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .expect("enviar GET");
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
        let resposta = pedir("/__anti_ies/health");
        assert!(resposta.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(resposta.contains(r#"{"status":"ok","servidor":"anti-ies"}"#));
    }

    #[test]
    fn controlador_sei_nao_implementado_responde_501_no_fio() {
        let resposta = pedir("/sei/controlador.php?acao=procedimento_trabalhar");
        assert!(resposta.starts_with("HTTP/1.1 501 Not Implemented\r\n"));
        assert!(resposta.contains("operacao_sei_nao_implementada"));
    }
}
