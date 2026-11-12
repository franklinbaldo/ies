//! Navegacao somente GET na tela Retorno Programado publicada pelo Controle.
//! Oraculo: Todos `retorno_programado.py` @323eb01, cassette unidade-listar-retornos-unb.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroChaveDaConsulta, ErroConsultaDoControlador, menciona_acao,
};
use crate::instalacao::Instalacao;
use crate::transporte::FalhaTransporte;
use crate::unidade_retornos::{self, ErroRetornos, Retorno};
use crate::url_observada::{UrlHttps, pasta};
use std::collections::HashSet;

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLeituraRetornos {
    Transporte(#[from] FalhaTransporte),
    Controle(#[from] crate::erro_web::ErroCaixa),
    Tabela(#[from] ErroRetornos),
    LinkDeRetornosAusente,
    LinkDeRetornosAmbiguo,
    LinkDeRetornosForaDaInstalacao,
    CalendarioDosRetornosDesconhecido,
    MesDosRetornosInacessivel,
    MesDaCliInvalido,
    UnidadeDiferenteDaSolicitada,
    ConsultaDoLinkIlegivel(#[source] ErroConsultaDoControlador),
    ChaveDoLinkIlegivel(#[source] ErroChaveDaConsulta),
}

impl ErroLeituraRetornos {
    /// A sessão do SEI acabou (decide o relogin único): por `match` no tipo, sem
    /// percorrer a cadeia `source()`.
    pub(crate) fn sessao_encerrada(&self) -> bool {
        match self {
            Self::Transporte(erro) => erro.sessao_encerrada(),
            Self::Controle(_)
            | Self::Tabela(_)
            | Self::LinkDeRetornosAusente
            | Self::LinkDeRetornosAmbiguo
            | Self::LinkDeRetornosForaDaInstalacao
            | Self::CalendarioDosRetornosDesconhecido
            | Self::MesDosRetornosInacessivel
            | Self::MesDaCliInvalido
            | Self::UnidadeDiferenteDaSolicitada
            | Self::ConsultaDoLinkIlegivel(_)
            | Self::ChaveDoLinkIlegivel(_) => false,
        }
    }
}

/// `--mes AAAA-MM` (parser do clap: saída 2): somente numeros e limites do
/// mes, sem depender do fuso da maquina.
pub(crate) fn mes_da_cli(s: &str) -> Result<(u16, u8), ErroLeituraRetornos> {
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
    Ok((a, m))
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
    while let Some(&byte) = bytes.get(i) {
        if byte == b'%' {
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
            out.push(byte);
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
    let nao = ErroLeituraRetornos::LinkDeRetornosForaDaInstalacao;
    let href = bruto.to_owned();
    if !href.bytes().all(|b| (33..=126).contains(&b))
        || href.contains(['#', '\\', '\''])
        || href.contains("..")
    {
        return Err(nao);
    }
    let servida = UrlHttps::interpretar(url_servida)
        .ok_or(ErroLeituraRetornos::LinkDeRetornosForaDaInstalacao)?;
    let url = if href.starts_with("https://") {
        href
    } else if href.starts_with('/') {
        format!("https://{}{href}", servida.host)
    } else if href.starts_with("controlador.php?") {
        let dir =
            pasta(servida.caminho).ok_or(ErroLeituraRetornos::LinkDeRetornosForaDaInstalacao)?;
        format!("https://{}/{dir}/{href}", servida.host)
    } else {
        return Err(nao);
    };
    let alvo =
        UrlHttps::interpretar(&url).ok_or(ErroLeituraRetornos::LinkDeRetornosForaDaInstalacao)?;
    if !instalacao.autoriza_controlador(alvo.host, alvo.caminho) {
        return Err(nao);
    }
    let consulta = alvo
        .consulta
        .ok_or(ErroLeituraRetornos::LinkDeRetornosForaDaInstalacao)?;
    let leitura = ConsultaDoControlador::ler(consulta).map_err(|erro| match erro {
        ErroConsultaDoControlador::RecusadaPeloTransporte(falha) => {
            ErroLeituraRetornos::Transporte(falha)
        }
        outro @ ErroConsultaDoControlador::ParSemIgual => {
            ErroLeituraRetornos::ConsultaDoLinkIlegivel(outro)
        }
    })?;
    let acao = leitura
        .unico("acao")
        .map_err(ErroLeituraRetornos::ChaveDoLinkIlegivel)?;
    let hash = leitura
        .unico("infra_hash")
        .map_err(ErroLeituraRetornos::ChaveDoLinkIlegivel)?;
    if AcaoDoSei::ler(acao) != Some(AcaoDoSei::RetornoProgramadoListar)
        || leitura.valores("acao_ajax").next().is_some()
        || hash.len() != 64
        || !hash.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(nao);
    }
    // Nome repetido recusa; o leitor aceita.
    let mut vistos = HashSet::new();
    let mut pares = Vec::new();
    for par in consulta.split('&') {
        let (k, v) = par.split_once('=').unwrap_or((par, ""));
        if !vistos.insert(k) {
            return Err(nao);
        }
        pares.push((k.to_owned(), v.to_owned()));
    }
    Ok(Link { url, pares })
}

/// Somente links de retorno da propria pagina servida. Outros links de menu
/// sao ignorados; links suspeitos da operacao sao erro, nao alternativas.
fn links(
    html: &crate::tela_html::HtmlServido,
    servida: &str,
    instalacao: &Instalacao,
) -> Result<Vec<Link>, ErroLeituraRetornos> {
    let dom = crate::tela_html::arvore(html);
    let mut saida = Vec::new();
    for no in dom.elementos("a") {
        let href = no.atributo("href").map(str::to_owned);
        let Some(href) = href else { continue };
        // Seleção tolerante pela ação exata (decodificada antes); a conferência
        // estrita fica em `conferir_link`.
        if menciona_acao(&href, AcaoDoSei::RetornoProgramadoListar) {
            saida.push(conferir_link(&href, servida, instalacao)?);
        }
    }
    Ok(saida)
}

fn menu(
    html: &crate::tela_html::HtmlServido,
    servida: &str,
    instalacao: &Instalacao,
) -> Result<String, ErroLeituraRetornos> {
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

/// O SEI e a unica fonte da data observada; ausencia de diaHoje nao
/// autoriza usar a data UTC como se fosse data do SEI.
fn hoje_na_tela(
    html: &crate::tela_html::HtmlServido,
    servida: &str,
    instalacao: &Instalacao,
) -> Result<Option<(u16, u8)>, ErroLeituraRetornos> {
    let dom = crate::tela_html::arvore(html);
    let mut marcado = None;
    let mut celulas_marcadas = 0;
    for no in dom.elementos("td") {
        let classe = no.atributo("class");
        if !classe.is_some_and(|c| c.split_whitespace().any(|v| v == "diaHoje")) {
            continue;
        }
        celulas_marcadas += 1;
        for filho in no.elementos("a") {
            let Some(href) = filho.atributo("href").map(str::to_owned) else {
                continue;
            };
            let link = conferir_link(&href, servida, instalacao)?;
            if let Some(data) = link.valor("data_atual").and_then(data_da_consulta)
                && marcado.replace(data).is_some()
            {
                return Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido);
            }
        }
    }
    match (celulas_marcadas, marcado) {
        (0, None) => Ok(None),
        (1, Some((ano, mes, _))) => Ok(Some((ano, mes))),
        _ => Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido),
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

/// Data civil UTC, apenas pista de navegacao. O mes publicado na saida
/// vem exclusivamente da celula diaHoje do SEI, nunca deste relogio.
fn data_utc_de_dias(dias: i64) -> Option<(u16, u8, u8)> {
    let z = dias.checked_add(719_468)?;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dia = doy - (153 * mp + 2) / 5 + 1;
    let mes = mp + if mp < 10 { 3 } else { -9 };
    let ano = y + i64::from(mes <= 2);
    Some((
        u16::try_from(ano).ok()?,
        u8::try_from(mes).ok()?,
        u8::try_from(dia).ok()?,
    ))
}
fn data_utc_agora() -> Option<(u16, u8, u8)> {
    let segundos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    data_utc_de_dias(i64::try_from(segundos / 86_400).ok()?)
}

/// So segue o primeiro dia do mes vizinho se o proprio SEI publicou
/// a URL. Duas ancoras com href identico significam um unico destino.
/// Um GET para cada transicao, sem retry ou restauracao de tela.
fn navegar_ate_mes<F>(
    tela: &mut crate::tela_html::HtmlServido,
    servida: &mut String,
    instalacao: &Instalacao,
    alvo: (u16, u8),
    get: &mut F,
    requisicoes: &mut u8,
) -> Result<(), ErroLeituraRetornos>
where
    F: FnMut(&str) -> Result<crate::tela_html::HtmlServido, ErroLeituraRetornos>,
{
    let mut vistos = HashSet::new();
    for _ in 0..=24 {
        let candidatos = links(tela, servida, instalacao)?;
        let (atual, _) = mes_inteiro(&candidatos)?;
        if atual == alvo {
            return Ok(());
        }
        if !vistos.insert(atual) || *requisicoes >= 26 {
            return Err(ErroLeituraRetornos::MesDosRetornosInacessivel);
        }
        let rumo = if posicao(alvo) > posicao(atual) {
            1
        } else {
            -1
        };
        let proximo =
            mes_vizinho(atual, rumo).ok_or(ErroLeituraRetornos::MesDosRetornosInacessivel)?;
        let mut escolhido: Option<&str> = None;
        for link in candidatos.iter().filter(|link| {
            link.valor("data_atual").and_then(data_da_consulta) == Some((proximo.0, proximo.1, 1))
        }) {
            match escolhido {
                Some(url) if url == link.url => {}
                Some(_) => return Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido),
                None => escolhido = Some(link.url.as_str()),
            }
        }
        let proxima_url = escolhido
            .ok_or(ErroLeituraRetornos::MesDosRetornosInacessivel)?
            .to_owned();
        *tela = get(&proxima_url)?;
        *servida = proxima_url;
        *requisicoes += 1;
    }
    Err(ErroLeituraRetornos::MesDosRetornosInacessivel)
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
    controle_html: &crate::tela_html::HtmlServido,
    instalacao: &Instalacao,
    mes: Option<(u16, u8)>,
    get: F,
) -> Result<Leitura, ErroLeituraRetornos>
where
    F: FnMut(&str) -> Result<crate::tela_html::HtmlServido, ErroLeituraRetornos>,
{
    // Nao consulta o relogio quando o pedido ja indicou o mes.
    let hoje_utc = if mes.is_some() {
        (1970, 1, 1)
    } else {
        data_utc_agora().ok_or(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido)?
    };
    ler_com_data_utc(controle_url, controle_html, instalacao, mes, hoje_utc, get)
}

/// A data UTC injetavel evita dependencia do relogio nos testes. Quando
/// diaHoje nao esta na tela aberta, procura no mes UTC e, apenas na
/// virada do dia, no mes vizinho. Nao atribui o relogio UTC ao SEI.
fn ler_com_data_utc<F>(
    controle_url: &str,
    controle_html: &crate::tela_html::HtmlServido,
    instalacao: &Instalacao,
    mes: Option<(u16, u8)>,
    hoje_utc: (u16, u8, u8),
    mut get: F,
) -> Result<Leitura, ErroLeituraRetornos>
where
    F: FnMut(&str) -> Result<crate::tela_html::HtmlServido, ErroLeituraRetornos>,
{
    let url = menu(controle_html, controle_url, instalacao)?;
    let mut tela = get(&url)?;
    let mut servida = url;
    let mut requisicoes: u8 = 2; // Controle e tela do menu
    let alvo = if let Some(mes) = mes {
        mes
    } else if let Some(observado) = hoje_na_tela(&tela, &servida, instalacao)? {
        observado
    } else {
        let (ano, mes, dia) = hoje_utc;
        if !unidade_retornos::data_valida(&format!("{dia:02}/{mes:02}/{ano:04}")) {
            return Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido);
        }
        let mut candidatos = vec![(ano, mes)];
        if dia == 1
            && let Some(anterior) = mes_vizinho((ano, mes), -1)
        {
            candidatos.push(anterior);
        }
        if Some(dia) == unidade_retornos::ultimo_dia(ano, mes)
            && let Some(seguinte) = mes_vizinho((ano, mes), 1)
        {
            candidatos.push(seguinte);
        }
        let mut hoje_do_sei = None;
        for candidato in candidatos {
            navegar_ate_mes(
                &mut tela,
                &mut servida,
                instalacao,
                candidato,
                &mut get,
                &mut requisicoes,
            )?;
            if let Some(observado) = hoje_na_tela(&tela, &servida, instalacao)? {
                hoje_do_sei = Some(observado);
                break;
            }
        }
        hoje_do_sei.ok_or(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido)?
    };
    navegar_ate_mes(
        &mut tela,
        &mut servida,
        instalacao,
        alvo,
        &mut get,
        &mut requisicoes,
    )?;
    let candidatos = links(&tela, &servida, instalacao)?;
    let (atual, link_mes) = mes_inteiro(&candidatos)?;
    if atual != alvo {
        return Err(ErroLeituraRetornos::MesDosRetornosInacessivel);
    }
    let html = get(link_mes)?;
    requisicoes += 1;
    let itens = unidade_retornos::extrair_retornos(&html)?;
    Ok(Leitura {
        mes: format!("{:04}-{:02}", alvo.0, alvo.1),
        itens,
        requisicoes,
    })
}

/// Confere, no replay offline, as telas reais do calendario com o mesmo
/// parser, origem e URL servida usados na CLI. Nao publica URLs assinadas.
#[cfg(test)]
pub(crate) fn conferir_calendario_gravado(
    html: &str,
    url_servida: &str,
    instalacao: &Instalacao,
) -> Result<(bool, bool), ErroLeituraRetornos> {
    let candidatos = links(
        &crate::tela_html::HtmlServido::de_teste(html),
        url_servida,
        instalacao,
    )?;
    let (mes, _) = mes_inteiro(&candidatos)?;
    let marcado = hoje_na_tela(
        &crate::tela_html::HtmlServido::de_teste(html),
        url_servida,
        instalacao,
    )?
    .is_some();
    let anterior = mes_vizinho(mes, -1);
    let proximo = mes_vizinho(mes, 1);
    let vizinho = candidatos.iter().any(|link| {
        let data = link.valor("data_atual").and_then(data_da_consulta);
        data.is_some_and(|(ano, mes, dia)| {
            dia == 1 && (Some((ano, mes)) == anterior || Some((ano, mes)) == proximo)
        })
    });
    Ok((marcado, vizinho))
}

#[cfg(test)]
#[path = "unidade_retornos_web/tests.rs"]
mod tests;
