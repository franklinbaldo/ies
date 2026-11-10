//! Servidor experimental. Leitura de cassettes e hipótese de documento simulado.
mod arvore_compartilhada;
mod bloco_detalhe;
mod blocos;
mod controle_detalhado;
mod controle_resumido;
pub mod documento_arvore;
pub mod documento_cancelar;
mod documento_ciencia;
pub mod documento_versoes;
mod documentos;
pub mod editor_documento;
pub mod pdf_externo;
pub mod pesquisa_documento;
mod prazo_processo;
mod processo_acompanhar;
mod processo_andamento;
mod processo_anexacao;
mod processo_atribuicao;
mod processo_barra;
mod processo_ciencia;
mod processo_comentarios;
mod processo_criacao;
pub mod processo_envio;
mod processo_marcadores;
mod processo_relacionar;
mod processo_sobrestamento;
mod protocolo_sintetico;
mod rotas;
use iesei::banco_cassettes::ler;
use iesei::instalacao::Instalacao;
use iesei::retornos_cassettes::{Mes, PeriodoMensal, Situacao, ler as ler_retornos};
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
    AcessoNegado,
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
            Self::AcessoNegado => 403,
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
            Self::AcessoNegado => "Forbidden",
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
    /// Bytes do PDF quando o corpo HTTP não é texto UTF-8.
    pub corpo_binario: Option<Vec<u8>>,
}

impl Resposta {
    #[must_use]
    pub fn nova(estado: Estado) -> Self {
        let corpo = match estado {
            Estado::Ok => r#"{"status":"ok","servidor":"anti-ies"}"#,
            Estado::Redirecionamento => "",
            Estado::RequisicaoInvalida => r#"{"erro":"requisicao_invalida"}"#,
            Estado::MetodoNaoPermitido => r#"{"erro":"metodo_nao_permitido"}"#,
            Estado::AcessoNegado => r#"{"erro":"acesso_recusado"}"#,
            Estado::NaoEncontrado => r#"{"erro":"rota_nao_encontrada"}"#,
            Estado::NaoImplementado => r#"{"erro":"operacao_sei_nao_implementada"}"#,
            Estado::DadosInsuficientes => r#"{"erro":"observacao_insuficiente"}"#,
        };
        Self {
            estado,
            tipo_conteudo: "application/json; charset=utf-8",
            corpo: corpo.to_owned(),
            localizacao: None,
            corpo_binario: None,
        }
    }

    #[must_use]
    pub fn html(corpo: String) -> Self {
        Self {
            estado: Estado::Ok,
            tipo_conteudo: "text/html; charset=utf-8",
            corpo,
            localizacao: None,
            corpo_binario: None,
        }
    }

    #[must_use]
    pub fn redirecionar(localizacao: String) -> Self {
        Self {
            estado: Estado::Redirecionamento,
            tipo_conteudo: "text/html; charset=utf-8",
            corpo: String::new(),
            localizacao: Some(localizacao),
            corpo_binario: None,
        }
    }
    /// Resposta com os bytes originais do PDF, sem conversão para texto.
    #[must_use]
    pub fn pdf(bytes: Vec<u8>) -> Self {
        Self {
            estado: Estado::Ok,
            tipo_conteudo: "application/pdf",
            corpo: String::new(),
            localizacao: None,
            corpo_binario: Some(bytes),
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

/// O menu e o calendário são projeções do banco efêmero, jamais páginas congeladas.
/// A assinatura literal é exclusiva do simulador: nenhuma sessão do SEI é armazenada.
fn link_calendario() -> String {
    format!(
        "controlador.php?acao=retorno_programado_listar&amp;infra_hash={}",
        protocolo_sintetico::INFRA_HASH
    )
}

fn menu_com_retornos(
    html: String,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> String {
    // O Controle válido continua legível mesmo se um mês sintético
    // desconhecido (ex.: "01"/"31") não oferecer calendário navegável.
    let Ok(observados) = iesei::retornos_cassettes::periodos(banco, instalacao, cenario) else {
        return html;
    };
    if observados.is_empty() {
        return html;
    }
    let link = format!("<a href='{}'>Retorno Programado</a>", link_calendario());
    html.replacen("</body>", &format!("{link}</body>"), 1)
}

fn calendario_html(atual: &PeriodoMensal, observados: &[PeriodoMensal]) -> String {
    let base = link_calendario();
    let mut html = format!(
        "<html><body><a href='{base}&amp;data_inicial={}&amp;data_final={}'>Ver todo o mes</a>",
        atual.data_inicial, atual.data_final
    );
    let posicao = i32::from(atual.ano) * 12 + i32::from(atual.mes);
    for periodo in observados {
        let outra = i32::from(periodo.ano) * 12 + i32::from(periodo.mes);
        if (outra - posicao).abs() == 1 {
            write!(
                html,
                "<a href='{base}&amp;data_atual={}'>Mes vizinho</a>",
                periodo.data_inicial
            )
            .expect("String");
        }
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
    match (
        parametro_unico(alvo, "data_inicial"),
        parametro_unico(alvo, "data_final"),
    ) {
        (Some(inicio), Some(fim)) => {
            let Ok(Some(mes)) = ler_retornos(banco, instalacao, cenario, inicio, fim) else {
                return Resposta::nova(Estado::DadosInsuficientes);
            };
            Resposta::html(html_de_retornos(&mes))
        }
        (None, None) => {
            let Ok(observados) = iesei::retornos_cassettes::periodos(banco, instalacao, cenario)
            else {
                return Resposta::nova(Estado::DadosInsuficientes);
            };
            let atual = if let Some(data) = parametro_unico(alvo, "data_atual") {
                observados
                    .iter()
                    .find(|periodo| periodo.data_inicial == data)
            } else {
                observados.first()
            };
            atual.map_or_else(
                || Resposta::nova(Estado::DadosInsuficientes),
                |periodo| Resposta::html(calendario_html(periodo, &observados)),
            )
        }
        _ => Resposta::nova(Estado::RequisicaoInvalida),
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
    if !eh_controlador_sei(alvo) || parametro_unico(alvo, "acao") != Some("procedimento_controlar")
    {
        return despachar(metodo, alvo);
    }
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Ok(detalhe) = iesei::detalhada_cassettes::ler(banco, instalacao, cenario) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    if let Some(observacao) = detalhe {
        return controle_detalhado::html(&observacao)
            .map(|html| menu_com_retornos(html, banco, instalacao, cenario))
            .map_or_else(
                || Resposta::nova(Estado::DadosInsuficientes),
                Resposta::html,
            );
    }
    let Ok(Some(snapshot)) = ler(banco, instalacao, cenario) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let Ok(projetado) =
        iesei::processos::controle_da_unidade(banco, instalacao, cenario, snapshot.clone())
    else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let Some(corpo) = controle_resumido::controle_html(&projetado, &snapshot) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    Resposta::html(menu_com_retornos(corpo, banco, instalacao, cenario))
}

/// Pedido HTTP local. Não preservar URL, headers nem corpo além do atendimento.
struct PedidoHttp {
    metodo: String,
    alvo: String,
    corpo: Vec<u8>,
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
    let limite = match parametro_unico(&alvo, "acao") {
        Some("documento_upload_anexo") => 256 * 1024 * 1024,
        Some("documento_editar" | "editor_salvar") => 24 * 1024 * 1024,
        _ => 8192,
    };
    if !terminado || tamanho.unwrap_or(0) > limite || (metodo == "POST" && tamanho.is_none()) {
        return Ok(Some(Err(Estado::RequisicaoInvalida)));
    }
    let mut corpo = vec![0; tamanho.unwrap_or(0)];
    leitor.read_exact(&mut corpo)?;
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
    let bytes = resposta
        .corpo_binario
        .as_deref()
        .unwrap_or(resposta.corpo.as_bytes());
    write!(
        fluxo,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n{}Connection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n\r\n",
        resposta.estado.codigo_http(),
        resposta.estado.motivo_http(),
        resposta.tipo_conteudo,
        bytes.len(),
        location,
    )?;
    fluxo.write_all(bytes)?;
    fluxo.flush()
}
