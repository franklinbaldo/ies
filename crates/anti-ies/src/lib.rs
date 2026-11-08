//! Servidor experimental. O Controle R usa somente observações verificadas.
pub mod documento_arvore;
pub mod documento_versoes;
use iesei::banco_cassettes::{ControleResumido, Linha, ler};
use iesei::instalacao::Instalacao;
use iesei::observacao::TabelaObservada;
use iesei::retornos_cassettes::{Mes, Situacao, ler as ler_retornos};
use rusqlite::Connection;
use std::fmt::Write as _;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;

/// Estado tipado no limite HTTP, sem classificar erros pelo texto.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Estado {
    Ok,
    RequisicaoInvalida,
    MetodoNaoPermitido,
    NaoEncontrado,
    NaoImplementado,
    DadosInsuficientes,
}

impl Estado {
    #[must_use]
    pub const fn codigo_http(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::RequisicaoInvalida => 400,
            Self::MetodoNaoPermitido => 405,
            Self::NaoEncontrado => 404,
            Self::NaoImplementado => 501,
            Self::DadosInsuficientes => 503,
        }
    }

    #[must_use]
    pub const fn motivo_http(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::RequisicaoInvalida => "Bad Request",
            Self::MetodoNaoPermitido => "Method Not Allowed",
            Self::NaoEncontrado => "Not Found",
            Self::NaoImplementado => "Not Implemented",
            Self::DadosInsuficientes => "Service Unavailable",
        }
    }
}

/// Resposta HTTP sem URLs assinadas ou sessão no corpo.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resposta {
    pub estado: Estado,
    pub tipo_conteudo: &'static str,
    pub corpo: String,
}

impl Resposta {
    #[must_use]
    pub fn nova(estado: Estado) -> Self {
        let corpo = match estado {
            Estado::Ok => r#"{"status":"ok","servidor":"anti-ies"}"#,
            Estado::RequisicaoInvalida => r#"{"erro":"requisicao_invalida"}"#,
            Estado::MetodoNaoPermitido => r#"{"erro":"metodo_nao_permitido"}"#,
            Estado::NaoEncontrado => r#"{"erro":"rota_nao_encontrada"}"#,
            Estado::NaoImplementado => r#"{"erro":"operacao_sei_nao_implementada"}"#,
            Estado::DadosInsuficientes => r#"{"erro":"observacao_insuficiente"}"#,
        };
        Self {
            estado,
            tipo_conteudo: "application/json; charset=utf-8",
            corpo: corpo.to_owned(),
        }
    }
}

fn eh_controlador_sei(alvo: &str) -> bool {
    let caminho = alvo.split_once('?').map_or(alvo, |(caminho, _)| caminho);
    caminho.ends_with("/controlador.php")
}

/// Rotas sem dados não simulam respostas SEI.
#[must_use]
pub fn despachar(metodo: &str, alvo: &str) -> Resposta {
    let estado = if alvo == "/__anti_ies/health" {
        if metodo == "GET" {
            Estado::Ok
        } else {
            Estado::MetodoNaoPermitido
        }
    } else if eh_controlador_sei(alvo) {
        Estado::NaoImplementado
    } else {
        Estado::NaoEncontrado
    };
    Resposta::nova(estado)
}

fn escapar(texto: &str) -> String {
    texto
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn tabela_html(id: &str, tabela: &TabelaObservada<Linha>, destino: &mut String) -> Option<()> {
    if !tabela.pagina_verificavel() {
        return None;
    }
    // O parser do Controle precisa de um total publicado. Ausência não é zero.
    let total = tabela.total_declarado?;
    let mut legenda = format!("{total} registros");
    if let Some(faixa) = tabela.faixa_observada {
        write!(legenda, " - {} a {}", faixa.inicio(), faixa.fim()).ok()?;
    }
    write!(destino, "<table id='{id}'><caption>{legenda}:</caption>").ok()?;
    for linha in &tabela.linhas {
        let classe = match (linha.visualizado, linha.sigiloso) {
            (true, false) => "processoVisualizado",
            (false, false) => "processoNaoVisualizado",
            (true, true) => "processoVisualizadoSigiloso",
            (false, true) => "processoNaoVisualizadoSigiloso",
        };
        write!(
            destino,
            "<tr id='P{}'><td><a class='{classe}'>{}</a></td></tr>",
            escapar(&linha.id),
            escapar(&linha.numero)
        )
        .ok()?;
    }
    destino.push_str("</table>");
    Some(())
}

fn controle_html(controle: &ControleResumido) -> Option<String> {
    let mut html = format!(
        "<html><body><a id='lnkInfraUnidade'>{}</a><input id='hdnTipoVisualizacao' value='R'>",
        escapar(controle.unidade.as_str())
    );
    tabela_html("tblProcessosRecebidos", &controle.recebidos, &mut html)?;
    tabela_html("tblProcessosGerados", &controle.gerados, &mut html)?;
    html.push_str("</body></html>");
    Some(html)
}

/// Renderiza dados já observados. Ausência de linhas só significa vazio
/// quando a cassette declarou o mês e o leitor do IES o aceitou.
fn html_de_retornos(mes: &Mes) -> String {
    let mut html = String::from("<html><body>");
    for (situacao, id, resumo) in [
        (
            Situacao::Aguardando,
            "divAguardandoAreaTabela",
            "Aguardando Retorno de Outras Unidades",
        ),
        (Situacao::Devolver, "divDevolverAreaTabela", "Para Devolver"),
    ] {
        let itens: Vec<_> = mes
            .itens
            .iter()
            .filter(|r| r.situacao == situacao)
            .collect();
        if itens.is_empty() {
            write!(html, "<div id='{id}'>Nenhum registro encontrado.</div>")
                .expect("formatação String");
            continue;
        }
        write!(
            html,
            "<div id='{id}'><table summary='{resumo}'><caption>{} registros:</caption>\
             <thead><tr><th>Processo</th><th>Unidade</th>\
             <th>Data Programada</th><th>Data de Retorno Efetiva</th>\
             <th>Prazo Restante</th></tr></thead><tbody>",
            itens.len()
        )
        .expect("formatação String");
        for item in itens {
            write!(
                html,
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escapar(&item.numero),
                escapar(&item.unidade),
                escapar(&item.data_programada),
                escapar(item.retornado_em.as_deref().unwrap_or("")),
                escapar(item.prazo_restante.as_deref().unwrap_or(""))
            )
            .expect("formatação String");
        }
        html.push_str("</tbody></table></div>");
    }
    html.push_str("</body></html>");
    html
}

/// Valor único de parâmetro observado na URL; repetição não seleciona mês.
fn parametro_unico<'a>(alvo: &'a str, nome: &str) -> Option<&'a str> {
    let (_, query) = alvo.split_once('?')?;
    let mut valores = query
        .split('&')
        .filter_map(|parte| parte.split_once('='))
        .filter(|(chave, _)| *chave == nome)
        .map(|(_, valor)| valor);
    let valor = valores.next()?;
    (valores.next().is_none() && !valor.is_empty()).then_some(valor)
}

/// Nova rota: lista mensal de retornos, sem comunicar com o SEI.
///
/// Um mês ausente do banco recebe 503, não uma página de zero retornos.
#[must_use]
pub fn retorno_programado(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !eh_controlador_sei(alvo)
        || parametro_unico(alvo, "acao") != Some("retorno_programado_listar")
    {
        return despachar(metodo, alvo);
    }
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let (Some(inicio), Some(fim)) = (
        parametro_unico(alvo, "data_inicial"),
        parametro_unico(alvo, "data_final"),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Ok(Some(mes)) = ler_retornos(banco, instalacao, cenario, inicio, fim) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    Resposta {
        estado: Estado::Ok,
        tipo_conteudo: "text/html; charset=utf-8",
        corpo: html_de_retornos(&mes),
    }
}

/// Atende somente um GET de Controle R observado no banco efêmero.
/// Não utiliza URLs assinadas, cookies nem dados da sessão operacional.
#[must_use]
pub fn controlar(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !eh_controlador_sei(alvo) || !alvo.ends_with("?acao=procedimento_controlar") {
        return despachar(metodo, alvo);
    }
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Ok(Some(snapshot)) = ler(banco, instalacao, cenario) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let Some(corpo) = controle_html(&snapshot) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    Resposta {
        estado: Estado::Ok,
        tipo_conteudo: "text/html; charset=utf-8",
        corpo,
    }
}

/// Um atendimento HTTP real, com fonte optional e sem expor a sessão.
/// O banco é aberto pelo servidor exclusivamente em modo de leitura.
///
/// # Errors
/// Propaga falhas de I/O da conexão local.
pub fn atender_http(
    fluxo: &mut TcpStream,
    fonte: Option<(&Connection, &Instalacao, &str)>,
) -> io::Result<()> {
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
                if let Some((banco, instalacao, cenario)) = fonte {
                    if parametro_unico(alvo, "acao") == Some("retorno_programado_listar") {
                        retorno_programado(metodo, alvo, banco, instalacao, cenario)
                    } else if matches!(
                        parametro_unico(alvo, "acao"),
                        Some("documento_versao_listar" | "documento_visualizar")
                    ) {
                        documento_versoes::responder(metodo, alvo, banco, instalacao, cenario)
                    } else if matches!(
                        parametro_unico(alvo, "acao"),
                        Some("procedimento_visualizar" | "procedimento_trabalhar")
                    ) {
                        documento_arvore::responder(metodo, alvo, banco, instalacao, cenario)
                    } else {
                        controlar(metodo, alvo, banco, instalacao, cenario)
                    }
                } else {
                    despachar(metodo, alvo)
                }
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
