//! Servidor experimental. Leitura de cassettes e hipótese de documento simulado.
pub mod documento_arvore;
pub mod documento_versoes;
mod documentos;
pub mod pesquisa_documento;
pub mod processo_envio;
mod rotas;
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
    Redirecionamento,
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
            Self::Redirecionamento => 302,
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
            Self::Redirecionamento => "Found",
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
    pub localizacao: Option<String>,
}

impl Resposta {
    #[must_use]
    pub fn nova(estado: Estado) -> Self {
        let corpo = match estado {
            Estado::Ok => r#"{"status":"ok","servidor":"anti-ies"}"#,
            Estado::Redirecionamento => "",
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
            localizacao: None,
        }
    }

    #[must_use]
    pub fn html(corpo: String) -> Self {
        Self {
            estado: Estado::Ok,
            tipo_conteudo: "text/html; charset=utf-8",
            corpo,
            localizacao: None,
        }
    }

    #[must_use]
    pub fn redirecionar(localizacao: String) -> Self {
        Self {
            estado: Estado::Redirecionamento,
            tipo_conteudo: "text/html; charset=utf-8",
            corpo: String::new(),
            localizacao: Some(localizacao),
        }
    }
}

pub(crate) fn eh_controlador_sei(alvo: &str) -> bool {
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

pub(crate) fn escapar(texto: &str) -> String {
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
    // A pesquisa e o unico POST da leitura de documento, sem criar objeto.
    // Assinaturas aqui sao placeholders locais, nunca tokens de SEI.
    let formulario = format!(
        r#"<form id="frmProtocoloPesquisaRapida" method="post" action="controlador.php?acao=protocolo_pesquisa_rapida&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}"><input type="text" id="txtPesquisaRapida" name="txtPesquisaRapida" /><img src="svg/pesquisa_rapida.svg" onclick="document.getElementById('frmProtocoloPesquisaRapida').submit();" /></form>"#,
        "b".repeat(64)
    );
    let mut html = format!(
        "<html><body>{formulario}<a id='lnkInfraUnidade'>{}</a><input id='hdnTipoVisualizacao' value='R'>",
        escapar(controle.unidade.as_str())
    );
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
pub(crate) fn parametro_unico<'a>(alvo: &'a str, nome: &str) -> Option<&'a str> {
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
    Resposta::html(html_de_retornos(&mes))
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
    if !eh_controlador_sei(alvo) || parametro_unico(alvo, "acao") != Some("procedimento_controlar")
    {
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
    Resposta::html(corpo)
}

/// Pedido HTTP local. Não preservar URL, headers nem corpo além do atendimento.
struct PedidoHttp {
    metodo: String,
    alvo: String,
    corpo: String,
}

/// Leitura limitada da requisição para suportar o POST do simulador.
/// Sem cookies, proxy, streaming, chunked ou acesso à rede externa.
fn ler_pedido(fluxo: &mut TcpStream) -> io::Result<Option<Result<PedidoHttp, Estado>>> {
    let mut leitor = BufReader::new(fluxo);
    let mut linha = String::new();
    if leitor.read_line(&mut linha)? == 0 {
        return Ok(None);
    }
    if linha.len() > 4096 || !linha.ends_with('\n') {
        return Ok(Some(Err(Estado::RequisicaoInvalida)));
    }
    let mut campos = linha.split_ascii_whitespace();
    let (Some(metodo), Some(alvo), Some("HTTP/1.1" | "HTTP/1.0"), None) =
        (campos.next(), campos.next(), campos.next(), campos.next())
    else {
        return Ok(Some(Err(Estado::RequisicaoInvalida)));
    };
    let (metodo, alvo) = (metodo.to_owned(), alvo.to_owned());
    if alvo.len() > 2048 || !alvo.starts_with('/') {
        return Ok(Some(Err(Estado::RequisicaoInvalida)));
    }
    let mut tamanho: Option<usize> = None;
    let mut terminado = false;
    for _ in 0..64 {
        linha.clear();
        if leitor.read_line(&mut linha)? == 0 || linha.len() > 4096 {
            return Ok(Some(Err(Estado::RequisicaoInvalida)));
        }
        if linha == "\r\n" || linha == "\n" {
            terminado = true;
            break;
        }
        let Some((nome, valor)) = linha.split_once(':') else {
            return Ok(Some(Err(Estado::RequisicaoInvalida)));
        };
        if nome.eq_ignore_ascii_case("Transfer-Encoding") {
            return Ok(Some(Err(Estado::RequisicaoInvalida)));
        }
        if nome.eq_ignore_ascii_case("Content-Length") {
            if tamanho.is_some() {
                return Ok(Some(Err(Estado::RequisicaoInvalida)));
            }
            tamanho = valor.trim().parse::<usize>().ok();
            if tamanho.is_none() {
                return Ok(Some(Err(Estado::RequisicaoInvalida)));
            }
        }
    }
    if !terminado || tamanho.unwrap_or(0) > 8192 || (metodo == "POST" && tamanho.is_none()) {
        return Ok(Some(Err(Estado::RequisicaoInvalida)));
    }
    let mut corpo = vec![0; tamanho.unwrap_or(0)];
    leitor.read_exact(&mut corpo)?;
    let Ok(corpo) = String::from_utf8(corpo) else {
        return Ok(Some(Err(Estado::RequisicaoInvalida)));
    };
    Ok(Some(Ok(PedidoHttp {
        metodo,
        alvo,
        corpo,
    })))
}

/// Atende a sessão HTTP local sequencialmente, com estado SQLite compartilhado.
/// Um POST de criação é consumido uma única vez; nenhuma escrita remota existe.
///
/// # Errors
/// Propaga falhas de I/O na conexão local.
pub fn atender_http(
    fluxo: &mut TcpStream,
    fonte: Option<(&Connection, &Instalacao, &str)>,
) -> io::Result<()> {
    let Some(pedido) = ler_pedido(fluxo)? else {
        return Ok(());
    };
    let resposta = match pedido {
        Err(estado) => Resposta::nova(estado),
        Ok(pedido) => {
            if let Some((banco, instalacao, cenario)) = fonte {
                rotas::responder(
                    &pedido.metodo,
                    &pedido.alvo,
                    &pedido.corpo,
                    banco,
                    instalacao,
                    cenario,
                )
            } else {
                despachar(&pedido.metodo, &pedido.alvo)
            }
        }
    };
    let location = resposta
        .localizacao
        .as_ref()
        .map_or_else(String::new, |valor| format!("Location: {valor}\r\n"));
    write!(
        fluxo,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n{}Connection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n\r\n{}",
        resposta.estado.codigo_http(),
        resposta.estado.motivo_http(),
        resposta.tipo_conteudo,
        resposta.corpo.len(),
        location,
        resposta.corpo
    )?;
    fluxo.flush()
}
