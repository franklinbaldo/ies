//! Navegacao somente GET na tela Retorno Programado publicada pelo Controle.
//! Oraculo: Todos `retorno_programado.py` @323eb01, cassette unidade-listar-retornos-unb.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroAcaoDaUrl, ErroConsultaDoControlador, ErroParametroDaUrl,
    menciona_acao,
};
use crate::instalacao::Instalacao;
use crate::transporte::FalhaTransporte;
use crate::unidade_retornos::{self, DataCivil, ErroRetornos, MesAno, Retorno};
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
    ChaveDoLinkIlegivel(#[source] ErroParametroDaUrl),
    AcaoDoLinkIlegivel(#[source] ErroAcaoDaUrl),
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
            | Self::ChaveDoLinkIlegivel(_)
            | Self::AcaoDoLinkIlegivel(_) => false,
        }
    }
}

/// `--mes AAAA-MM` (parser do clap: saída 2): somente numeros e limites do
/// mes, sem depender do fuso da maquina.
pub(crate) fn mes_da_cli(s: &str) -> Result<MesAno, ErroLeituraRetornos> {
    MesAno::ler_cli(s).ok_or(ErroLeituraRetornos::MesDaCliInvalido)
}

/// Nao pode aparecer em logs: contem URL assinada.
struct Link {
    url: String,
    data_inicial: Option<DataCivil>,
    data_final: Option<DataCivil>,
    data_atual: Option<DataCivil>,
}

fn data_da_consulta(valor: &str) -> Option<DataCivil> {
    DataCivil::ler(&valor.replace("%2F", "/").replace("%2f", "/"))
}

/// Aceita apenas URLs assinadas de `retorno_programado_listar` no controlador
/// da instalacao. Nunca envia cookie a outro host nem segue redirects.
fn conferir_link(
    bruto: &str,
    url_servida: &str,
    instalacao: &Instalacao,
) -> Result<Link, ErroLeituraRetornos> {
    let nao = || ErroLeituraRetornos::LinkDeRetornosForaDaInstalacao;
    if !bruto.bytes().all(|b| (33..=126).contains(&b))
        || bruto.contains(['#', '\\', '\''])
        || bruto.contains("..")
    {
        return Err(nao());
    }
    let servida = UrlHttps::interpretar(url_servida).ok_or_else(nao)?;
    let url = if bruto.starts_with("https://") {
        bruto.to_owned()
    } else if bruto.starts_with('/') {
        format!("https://{}{bruto}", servida.host)
    } else if bruto.starts_with("controlador.php?") {
        let dir = pasta(servida.caminho).ok_or_else(nao)?;
        format!("https://{}/{dir}/{bruto}", servida.host)
    } else {
        return Err(nao());
    };
    let alvo = UrlHttps::interpretar(&url).ok_or_else(nao)?;
    if !instalacao.autoriza_controlador(alvo.host, alvo.caminho) {
        return Err(nao());
    }
    let consulta = alvo.consulta.ok_or_else(nao)?;
    let leitura = ConsultaDoControlador::ler(consulta)
        .map_err(ErroLeituraRetornos::ConsultaDoLinkIlegivel)?;
    let acao = leitura
        .acao()
        .map_err(ErroLeituraRetornos::AcaoDoLinkIlegivel)?;
    let hash = leitura
        .unico("infra_hash")
        .map_err(ErroLeituraRetornos::ChaveDoLinkIlegivel)?;
    if acao != AcaoDoSei::RetornoProgramadoListar.como_na_url()
        || leitura.valores("acao_ajax").next().is_some()
        || hash.len() != 64
        || !hash.bytes().all(|b| b.is_ascii_hexdigit())
        || !leitura.sem_chaves_repetidas()
    {
        return Err(nao());
    }
    let ler_data = |chave| leitura.valores(chave).next().and_then(data_da_consulta);
    let (data_inicial, data_final, data_atual) = (
        ler_data("data_inicial"),
        ler_data("data_final"),
        ler_data("data_atual"),
    );
    Ok(Link {
        url,
        data_inicial,
        data_final,
        data_atual,
    })
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
        let Some(href) = no.atributo("href") else {
            continue;
        };
        // Seleção tolerante pela ação exata (decodificada antes); a conferência
        // estrita fica em `conferir_link`.
        if menciona_acao(href, AcaoDoSei::RetornoProgramadoListar) {
            saida.push(conferir_link(href, servida, instalacao)?);
        }
    }
    Ok(saida)
}

fn menu(
    html: &crate::tela_html::HtmlServido,
    servida: &str,
    instalacao: &Instalacao,
) -> Result<String, ErroLeituraRetornos> {
    let mut todos = links(html, servida, instalacao)?;
    match todos.len() {
        0 => Err(ErroLeituraRetornos::LinkDeRetornosAusente),
        1 => Ok(todos.swap_remove(0).url),
        _ => Err(ErroLeituraRetornos::LinkDeRetornosAmbiguo),
    }
}

/// Link unico de "Ver todo o mes": o primeiro dia e o ultimo dia do mesmo mes.
fn mes_inteiro(links: &[Link]) -> Result<(MesAno, &str), ErroLeituraRetornos> {
    let mut encontrado: Option<(MesAno, &str)> = None;
    for link in links {
        let (Some(inicio), Some(fim)) = (link.data_inicial, link.data_final) else {
            continue;
        };
        if inicio.dia != 1
            || inicio.mes_ano != fim.mes_ano
            || fim.dia != inicio.mes_ano.ultimo_dia()
        {
            continue;
        }
        // O calendario real publica duas vezes o mesmo "Ver todo o mes"
        // (cabecalho/rodape) nas sete instalacoes. Duplicata identica
        // nao e uma segunda escolha; URL assinada diferente continua ambigua.
        match encontrado {
            Some((anterior, url)) if anterior == inicio.mes_ano && url == link.url => {}
            Some(_) => return Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido),
            None => encontrado = Some((inicio.mes_ano, link.url.as_str())),
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
) -> Result<Option<MesAno>, ErroLeituraRetornos> {
    let dom = crate::tela_html::arvore(html);
    let mut marcado: Option<DataCivil> = None;
    let mut celulas_marcadas = 0;
    for no in dom.elementos("td") {
        let classe = no.atributo("class");
        if !classe.is_some_and(|c| c.split_whitespace().any(|v| v == "diaHoje")) {
            continue;
        }
        celulas_marcadas += 1;
        for filho in no.elementos("a") {
            let Some(href) = filho.atributo("href") else {
                continue;
            };
            let link = conferir_link(href, servida, instalacao)?;
            if let Some(data) = link.data_atual
                && marcado.replace(data).is_some()
            {
                return Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido);
            }
        }
    }
    match (celulas_marcadas, marcado) {
        (0, None) => Ok(None),
        (1, Some(data)) => Ok(Some(data.mes_ano)),
        _ => Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido),
    }
}

/// Data civil UTC, apenas pista de navegacao. O mes publicado na saida
/// vem exclusivamente da celula diaHoje do SEI, nunca deste relogio.
fn data_utc_de_dias(dias: i64) -> Option<DataCivil> {
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
    DataCivil::new(
        u16::try_from(ano).ok()?,
        u8::try_from(mes).ok()?,
        u8::try_from(dia).ok()?,
    )
}

fn data_utc_agora() -> Option<DataCivil> {
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
    alvo: MesAno,
    get: &mut F,
    requisicoes: &mut u8,
) -> Result<String, ErroLeituraRetornos>
where
    F: FnMut(&str) -> Result<crate::tela_html::HtmlServido, ErroLeituraRetornos>,
{
    let mut vistos = HashSet::new();
    for _ in 0..=24 {
        let candidatos = links(tela, servida, instalacao)?;
        let (atual, link_mes) = mes_inteiro(&candidatos)?;
        if atual == alvo {
            return Ok(link_mes.to_owned());
        }
        if !vistos.insert(atual) || *requisicoes >= 26 {
            return Err(ErroLeituraRetornos::MesDosRetornosInacessivel);
        }
        let rumo = if alvo.posicao() > atual.posicao() {
            1
        } else {
            -1
        };
        let proximo = atual
            .vizinho(rumo)
            .ok_or(ErroLeituraRetornos::MesDosRetornosInacessivel)?;
        let mut escolhido: Option<&str> = None;
        for link in candidatos.iter().filter(|link| {
            link.data_atual
                == Some(DataCivil {
                    mes_ano: proximo,
                    dia: 1,
                })
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
    mes: Option<MesAno>,
    get: F,
) -> Result<Leitura, ErroLeituraRetornos>
where
    F: FnMut(&str) -> Result<crate::tela_html::HtmlServido, ErroLeituraRetornos>,
{
    // Nao consulta o relogio quando o pedido ja indicou o mes.
    let hoje_utc = if mes.is_some() {
        DataCivil {
            mes_ano: MesAno { ano: 1970, mes: 1 },
            dia: 1,
        }
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
    mes: Option<MesAno>,
    hoje_utc: DataCivil,
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
        let mut candidatos = vec![hoje_utc.mes_ano];
        if hoje_utc.dia == 1
            && let Some(anterior) = hoje_utc.mes_ano.vizinho(-1)
        {
            candidatos.push(anterior);
        }
        if hoje_utc.dia == hoje_utc.mes_ano.ultimo_dia()
            && let Some(seguinte) = hoje_utc.mes_ano.vizinho(1)
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
    let link_mes = navegar_ate_mes(
        &mut tela,
        &mut servida,
        instalacao,
        alvo,
        &mut get,
        &mut requisicoes,
    )?;
    let html = get(&link_mes)?;
    requisicoes += 1;
    let itens = unidade_retornos::extrair_retornos(&html)?;
    Ok(Leitura {
        mes: alvo.to_string(),
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
    let anterior = mes.vizinho(-1);
    let proximo = mes.vizinho(1);
    let vizinho = candidatos.iter().any(|link| {
        link.data_atual.is_some_and(|data| {
            data.dia == 1 && (Some(data.mes_ano) == anterior || Some(data.mes_ano) == proximo)
        })
    });
    Ok((marcado, vizinho))
}

#[cfg(test)]
#[path = "unidade_retornos_web/tests.rs"]
mod tests;
