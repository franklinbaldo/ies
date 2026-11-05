//! Navegacao somente GET na tela Retorno Programado publicada pelo Controle.
//! Oraculo: Todos `retorno_programado.py` @323eb01, cassette unidade-listar-retornos-unb.
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::valor_do_atributo;
use crate::instalacao::Instalacao;
use crate::transporte::{self, FalhaTransporte, StatusHttpRecusado};
use crate::unidade_retornos::{self, ErroRetornos, Retorno};
use crate::url_observada::{UrlHttps, pasta};
use std::collections::HashSet;

#[derive(Debug)]
pub(crate) enum ErroLeituraRetornos {
    Transporte(FalhaTransporte),
    Controle(crate::erro_web::ErroCaixa),
    Tabela(ErroRetornos),
    LinkDeRetornosAusente,
    LinkDeRetornosAmbiguo,
    LinkDeRetornosForaDaInstalacao,
    CalendarioDosRetornosDesconhecido,
    MesDosRetornosInacessivel,
    MesDaCliInvalido,
    UnidadeDiferenteDaSolicitada,
}
impl ErroLeituraRetornos {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Transporte(e) => e.subcodigo(),
            Self::Controle(e) => e.subcodigo(),
            Self::Tabela(e) => e.subcodigo(),
            Self::LinkDeRetornosAusente => "link_de_retornos_ausente",
            Self::LinkDeRetornosAmbiguo => "link_de_retornos_ambiguo",
            Self::LinkDeRetornosForaDaInstalacao => "link_de_retornos_fora_da_instalacao",
            Self::CalendarioDosRetornosDesconhecido => "calendario_dos_retornos_desconhecido",
            Self::MesDosRetornosInacessivel => "mes_dos_retornos_inacessivel",
            Self::MesDaCliInvalido => "mes_dos_retornos_invalido",
            Self::UnidadeDiferenteDaSolicitada => "unidade_diferente_da_solicitada",
        }
    }
    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Transporte(e) => e.mensagem(),
            Self::Controle(_) => "Controle de Processos nao reconhecido",
            Self::Tabela(e) => e.mensagem(),
            Self::LinkDeRetornosAusente => "link Retorno Programado ausente no Controle",
            Self::LinkDeRetornosAmbiguo => "varios links Retorno Programado na mesma tela",
            Self::LinkDeRetornosForaDaInstalacao => {
                "link de retornos invalido ou fora da instalacao"
            }
            Self::CalendarioDosRetornosDesconhecido => {
                "calendario ou link Ver todo o mes nao reconhecido"
            }
            Self::MesDosRetornosInacessivel => "o calendario nao chega ao mes solicitado",
            Self::MesDaCliInvalido => "mes deve ser AAAA-MM valido",
            Self::UnidadeDiferenteDaSolicitada => "Controle mostra outra unidade",
        }
    }
}
impl crate::relogin_cli::FalhaPublicavel for ErroLeituraRetornos {
    fn subcodigo(&self) -> &'static str {
        self.subcodigo()
    }
    fn mensagem(&self) -> &'static str {
        self.mensagem()
    }
    fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte(e) => e.status_http(),
            _ => None,
        }
    }
}

/// Somente numeros e limites do mes, sem depender do fuso da maquina.
pub(crate) fn mes_pedido(valor: Option<&str>) -> Result<Option<(u16, u8)>, ErroLeituraRetornos> {
    let Some(s) = valor else { return Ok(None) };
    let (Some((a, m)), true) = (s.split_once('-'), s.len() == 7) else {
        return Err(ErroLeituraRetornos::MesDaCliInvalido);
    };
    if !a.bytes().all(|b| b.is_ascii_digit()) || !m.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroLeituraRetornos::MesDaCliInvalido);
    }
    let (Ok(a), Ok(m)) = (a.parse::<u16>(), m.parse::<u8>()) else {
        return Err(ErroLeituraRetornos::MesDaCliInvalido);
    };
    if unidade_retornos::ultimo_dia(a, m).is_none() {
        return Err(ErroLeituraRetornos::MesDaCliInvalido);
    }
    Ok(Some((a, m)))
}

/// Nao pode aparecer em logs: contem URL assinada.
struct Link {
    url: String,
    pares: Vec<(String, String)>,
}
impl Link {
    fn valor(&self, chave: &str) -> Option<&str> {
        self.pares
            .iter()
            .find(|(k, _)| k == chave)
            .map(|(_, v)| v.as_str())
    }
}

fn decodificar_data(valor: &str) -> Option<String> {
    let mut out = Vec::new();
    let bytes = valor.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let d = |b: u8| -> Option<u8> {
                match b {
                    b'0'..=b'9' => Some(b - b'0'),
                    b'A'..=b'F' => Some(b - b'A' + 10),
                    b'a'..=b'f' => Some(b - b'a' + 10),
                    _ => None,
                }
            };
            let hi = d(*bytes.get(i + 1)?)?;
            let lo = d(*bytes.get(i + 2)?)?;
            out.push(hi * 16 + lo);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn data_da_consulta(valor: &str) -> Option<(u16, u8, u8)> {
    let s = decodificar_data(valor)?;
    if !unidade_retornos::data_valida(&s) {
        return None;
    }
    let mut partes = s.split('/');
    let d = partes.next()?.parse().ok()?;
    let m = partes.next()?.parse().ok()?;
    let a = partes.next()?.parse().ok()?;
    Some((a, m, d))
}

/// Aceita apenas URLs assinadas de `retorno_programado_listar` no controlador
/// da instalacao. Nunca envia cookie a outro host nem segue redirects.
fn conferir_link(
    bruto: &str,
    url_servida: &str,
    instalacao: &Instalacao,
) -> Result<Link, ErroLeituraRetornos> {
    use ErroLeituraRetornos as E;
    let nao = E::LinkDeRetornosForaDaInstalacao;
    let href = decodificar_atributo(bruto).ok_or(E::LinkDeRetornosForaDaInstalacao)?;
    if !href.bytes().all(|b| (33..=126).contains(&b))
        || href.contains(['#', '\\', '\''])
        || href.contains("..")
    {
        return Err(nao);
    }
    let servida = UrlHttps::interpretar(url_servida).ok_or(E::LinkDeRetornosForaDaInstalacao)?;
    let url = if href.starts_with("https://") {
        href
    } else if href.starts_with('/') {
        format!("https://{}{href}", servida.host)
    } else if href.starts_with("controlador.php?") {
        let dir = pasta(servida.caminho).ok_or(E::LinkDeRetornosForaDaInstalacao)?;
        format!("https://{}/{dir}/{href}", servida.host)
    } else {
        return Err(nao);
    };
    let alvo = UrlHttps::interpretar(&url).ok_or(E::LinkDeRetornosForaDaInstalacao)?;
    if !instalacao.autoriza_controlador(alvo.host, alvo.caminho) {
        return Err(nao);
    }
    let consulta = alvo.consulta.ok_or(E::LinkDeRetornosForaDaInstalacao)?;
    let mut vistos = HashSet::new();
    let mut pares = Vec::new();
    for par in consulta.split('&') {
        let (k, v) = par
            .split_once('=')
            .ok_or(E::LinkDeRetornosForaDaInstalacao)?;
        transporte::conferir_chave_de_consulta(k).map_err(|_| E::LinkDeRetornosForaDaInstalacao)?;
        transporte::conferir_valor_de_consulta(v).map_err(|_| E::LinkDeRetornosForaDaInstalacao)?;
        if !vistos.insert(k) || k == "acao_ajax" {
            return Err(nao);
        }
        pares.push((k.to_owned(), v.to_owned()));
    }
    if !pares
        .iter()
        .any(|(k, v)| k == "acao" && v == "retorno_programado_listar")
        || !pares.iter().any(|(k, v)| {
            k == "infra_hash" && v.len() == 64 && v.bytes().all(|b| b.is_ascii_hexdigit())
        })
    {
        return Err(nao);
    }
    Ok(Link { url, pares })
}

/// Somente links de retorno da propria pagina servida. Outros links de menu
/// sao ignorados; links suspeitos da operacao sao erro, nao alternativas.
fn links(
    html: &str,
    servida: &str,
    instalacao: &Instalacao,
) -> Result<Vec<Link>, ErroLeituraRetornos> {
    use ErroLeituraRetornos as E;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| E::CalendarioDosRetornosDesconhecido)?;
    let parser = dom.parser();
    let mut saida = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let bruto =
            valor_do_atributo(tag, "href").map_err(|_| E::LinkDeRetornosForaDaInstalacao)?;
        let Some(bruto) = bruto else { continue };
        if bruto.contains("retorno_programado_listar") {
            saida.push(conferir_link(&bruto, servida, instalacao)?);
        }
    }
    Ok(saida)
}

fn menu(html: &str, servida: &str, instalacao: &Instalacao) -> Result<String, ErroLeituraRetornos> {
    let todos = links(html, servida, instalacao)?;
    match todos.len() {
        0 => Err(ErroLeituraRetornos::LinkDeRetornosAusente),
        1 => Ok(todos.into_iter().next().map_or_else(String::new, |l| l.url)),
        _ => Err(ErroLeituraRetornos::LinkDeRetornosAmbiguo),
    }
}

/// Link unico de "Ver todo o mes": o primeiro dia e o ultimo dia do mesmo mes.
fn mes_inteiro(links: &[Link]) -> Result<((u16, u8), &str), ErroLeituraRetornos> {
    let mut encontrado = None;
    for link in links {
        let (Some(inicio), Some(fim)) = (link.valor("data_inicial"), link.valor("data_final"))
        else {
            continue;
        };
        let (Some((ano, mes, 1)), Some((ano_fim, mes_fim, dia_fim))) =
            (data_da_consulta(inicio), data_da_consulta(fim))
        else {
            continue;
        };
        if ano != ano_fim
            || mes != mes_fim
            || Some(dia_fim) != unidade_retornos::ultimo_dia(ano, mes)
        {
            continue;
        }
        // O calendario real publica duas vezes o mesmo "Ver todo o mes"
        // (cabecalho/rodape) nas sete instalacoes. Duplicata identica
        // nao e uma segunda escolha; URL assinada diferente continua ambigua.
        match encontrado {
            Some((anterior, url)) if anterior == (ano, mes) && url == link.url => {}
            Some(_) => return Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido),
            None => encontrado = Some(((ano, mes), link.url.as_str())),
        }
    }
    encontrado.ok_or(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido)
}

fn hoje_na_tela(
    html: &str,
    servida: &str,
    instalacao: &Instalacao,
) -> Result<(u16, u8), ErroLeituraRetornos> {
    use ErroLeituraRetornos as E;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| E::CalendarioDosRetornosDesconhecido)?;
    let parser = dom.parser();
    let mut meses = HashSet::new();
    for no in dom.query_selector("td").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let classe =
            valor_do_atributo(tag, "class").map_err(|_| E::CalendarioDosRetornosDesconhecido)?;
        if !classe
            .as_deref()
            .is_some_and(|c| c.split_whitespace().any(|v| v == "diaHoje"))
        {
            continue;
        }
        for filho in tag.query_selector(parser, "a").into_iter().flatten() {
            let Some(a) = filho.get(parser).and_then(tl::Node::as_tag) else {
                continue;
            };
            let Some(href) =
                valor_do_atributo(a, "href").map_err(|_| E::CalendarioDosRetornosDesconhecido)?
            else {
                continue;
            };
            let link = conferir_link(&href, servida, instalacao)?;
            if let Some((ano, mes, _)) = link.valor("data_atual").and_then(data_da_consulta) {
                meses.insert((ano, mes));
            }
        }
    }
    if meses.len() == 1 {
        meses
            .into_iter()
            .next()
            .ok_or(E::CalendarioDosRetornosDesconhecido)
    } else {
        Err(E::CalendarioDosRetornosDesconhecido)
    }
}

fn mes_vizinho((a, m): (u16, u8), rumo: i32) -> Option<(u16, u8)> {
    match (m, rumo) {
        (12, 1) => Some((a.checked_add(1)?, 1)),
        (1, -1) => Some((a.checked_sub(1)?, 12)),
        (1..=11, 1) => Some((a, m + 1)),
        (2..=12, -1) => Some((a, m - 1)),
        _ => None,
    }
}
fn posicao((a, m): (u16, u8)) -> i32 {
    i32::from(a) * 12 + i32::from(m)
}

pub(crate) struct Leitura {
    pub(crate) mes: String,
    pub(crate) itens: Vec<Retorno>,
    pub(crate) requisicoes: u8,
}

/// Recebe Controle ja lido (GET 1) e GET injetavel para cada URL validada.
/// Mes na tela: GET do menu + GET Ver todo o mes = 3 GET contando Controle.
/// Mes adjacente: mais um GET por passo. Sem POST nem retry.
pub(crate) fn ler_com<F>(
    controle_url: &str,
    controle_html: &str,
    instalacao: &Instalacao,
    mes: Option<(u16, u8)>,
    get: F,
) -> Result<Leitura, ErroLeituraRetornos>
where
    F: FnMut(&str) -> Result<String, ErroLeituraRetornos>,
{
    ler_com_data_utc(controle_url, controle_html, instalacao, mes, (1970, 1, 1), get)
}

// Shim de teste: o relogio UTC sera injetado para exercitar viradas de mes
// sem alterar o relogio do CI. O RED comprovara a lacuna de navegacao.
fn ler_com_data_utc<F>(
    controle_url: &str,
    controle_html: &str,
    instalacao: &Instalacao,
    mes: Option<(u16, u8)>,
    _hoje_utc: (u16, u8, u8),
    mut get: F,
) -> Result<Leitura, ErroLeituraRetornos>
where
    F: FnMut(&str) -> Result<String, ErroLeituraRetornos>,
{
    use ErroLeituraRetornos as E;
    let url = menu(controle_html, controle_url, instalacao)?;
    let mut tela = get(&url)?;
    let mut servida = url;
    let mut requisicoes: u8 = 2; // Controle e tela do menu
    let alvo = match mes {
        Some(v) => v,
        None => hoje_na_tela(&tela, &servida, instalacao)?,
    };
    let mut vistos = HashSet::new();
    for _ in 0..=24 {
        let candidatos = links(&tela, &servida, instalacao)?;
        let (atual, link_mes) = mes_inteiro(&candidatos)?;
        if atual == alvo {
            let html = get(link_mes)?;
            requisicoes = requisicoes.saturating_add(1);
            let itens = unidade_retornos::extrair_retornos(&html).map_err(E::Tabela)?;
            return Ok(Leitura {
                mes: format!("{:04}-{:02}", alvo.0, alvo.1),
                itens,
                requisicoes,
            });
        }
        if !vistos.insert(atual) {
            return Err(E::MesDosRetornosInacessivel);
        }
        let rumo = if posicao(alvo) > posicao(atual) {
            1
        } else {
            -1
        };
        let proximo = mes_vizinho(atual, rumo).ok_or(E::MesDosRetornosInacessivel)?;
        let mut dia_primeiro = candidatos.iter().filter(|link| {
            link.valor("data_atual").and_then(data_da_consulta) == Some((proximo.0, proximo.1, 1))
        });
        let link = dia_primeiro.next().ok_or(E::MesDosRetornosInacessivel)?;
        if dia_primeiro.next().is_some() {
            return Err(E::CalendarioDosRetornosDesconhecido);
        }
        tela = get(&link.url)?;
        servida.clone_from(&link.url);
        requisicoes = requisicoes.saturating_add(1);
    }
    Err(E::MesDosRetornosInacessivel)
}

#[cfg(test)]
#[path = "unidade_retornos_web/tests.rs"]
mod tests;
