//! `ies unidade colunas`: GET da tela do Nível de detalhe pelo link que a D
//! publica (`ancNivelDetalhe`). A R não o publica: troca para D e fica nela.
use super::envio::{EnvioDoControle, Sessao, TelaAtual};
use super::passos::{contexto, trocar, unidade_da_tela};
use super::{ErroVisaoDoControle, Visao};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::instalacao::Instalacao;
use crate::nivel_de_detalhe::{NivelDeDetalhe, ler_nivel_de_detalhe};
use crate::transporte::decodificar_resposta;
use crate::transporte::sessao::CookieDaSessao;

/// Unidade exibida e as 12 opções do catálogo com `disponivel` e `marcada`.
/// Orçamento: D: 1 GET + 1 GET; R: 1 GET + POST D + GET.
pub(crate) fn ler_colunas<V: EnvioDoControle>(
    instalacao: &Instalacao,
    tela: &TelaAtual,
    unidade_solicitada: Option<&str>,
    cookie: &CookieDaSessao,
    envio: &mut V,
) -> Result<(String, NivelDeDetalhe), super::ErroVisaoDoControle> {
    let encontrado = contexto(&tela.html)?;
    let unidade = unidade_da_tela(&encontrado, unidade_solicitada)?;
    let mut sessao = Sessao {
        instalacao,
        unidade: &unidade,
        cookie,
        envio,
    };
    if encontrado.eh_detalhada() {
        let nivel = ler_pelo_link(&mut sessao, tela)?;
        return Ok((unidade.clone(), nivel));
    }
    if !encontrado.eh_resumida() {
        return Err(ErroVisaoDoControle::VisaoDoControleNaoReconhecida);
    }
    let detalhada = trocar(&mut sessao, tela, Visao::Detalhada)?;
    let nivel = ler_pelo_link(&mut sessao, &detalhada)?;
    Ok((unidade.clone(), nivel))
}

fn ler_pelo_link<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    tela: &TelaAtual,
) -> Result<NivelDeDetalhe, super::ErroVisaoDoControle> {
    let url = link_do_nivel_de_detalhe(sessao.instalacao, tela)?;
    let servida = obter_tela_do_nivel(sessao, &url)?;
    ler_nivel_de_detalhe(&servida.html).map_err(super::ErroVisaoDoControle::NivelDeDetalhe)
}

/// Um GET da tela do Nível de detalhe já autorizada (link da D ou a mesma
/// URL, na verificação da escrita), com URL e `Content-Type` da resposta.
pub(super) fn obter_tela_do_nivel<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    url: &str,
) -> Result<TelaAtual, super::ErroVisaoDoControle> {
    let cookie = sessao.cookie.valor()?;
    let resposta = sessao.envio.get(url, cookie)?;
    let mime = resposta.mime.clone();
    let html = decodificar_resposta(resposta)?;
    Ok(TelaAtual {
        url: url.to_owned(),
        html,
        mime,
    })
}

/// URL absoluta do `onclick="acaoConfigurarDetalhe('controlador.php?…')"`
/// de `a#ancNivelDetalhe`, resolvida contra a URL da tela servida.
pub(super) fn link_do_nivel_de_detalhe(
    instalacao: &Instalacao,
    tela: &TelaAtual,
) -> Result<String, super::ErroVisaoDoControle> {
    let invalido = super::ErroVisaoDoControle::LinkDoNivelDeDetalheNaoReconhecido;
    let dom = crate::tela_html::arvore(&tela.html);
    let nos = dom.com_id("ancNivelDetalhe").collect::<Vec<_>>();
    let no = match nos.as_slice() {
        [] => return Err(ErroVisaoDoControle::LinkDoNivelDeDetalheAusente),
        [no] => *no,
        _ => return Err(ErroVisaoDoControle::LinksDoNivelDeDetalheRepetidos),
    };
    let onclick = no
        .atributo("onclick")
        .map(str::to_owned)
        .ok_or_else(|| invalido.clone())?;
    let resto = onclick
        .strip_prefix("acaoConfigurarDetalhe('")
        .ok_or_else(|| invalido.clone())?;
    let resto = resto.strip_suffix(';').unwrap_or(resto);
    let relativo = resto.strip_suffix("')").ok_or_else(|| invalido.clone())?;
    let consulta = relativo
        .strip_prefix("controlador.php?")
        .ok_or_else(|| invalido.clone())?;
    if !relativo.bytes().all(|b| (33..=126).contains(&b)) || relativo.contains(['#', '\\', '\'']) {
        return Err(invalido);
    }
    let leitura = ConsultaDoControlador::ler(consulta)
        .map_err(super::ErroVisaoDoControle::ConsultaDoNivelIlegivel)?;
    let acao = leitura
        .acao()
        .map_err(super::ErroVisaoDoControle::AcaoDoNivelIlegivel)?;
    let hash = leitura
        .unico("infra_hash")
        .map_err(super::ErroVisaoDoControle::ChaveDoNivelIlegivel)?;
    if acao != AcaoDoSei::ProcedimentoConfigurarDetalhe.como_na_url()
        || leitura.valores("acao_ajax").next().is_some()
        || hash.len() != 64
        || !hash.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(invalido);
    }
    let base = tela
        .url
        .split_once('?')
        .map_or(tela.url.as_str(), |(antes, _)| antes);
    let (pasta, _) = base.rsplit_once('/').ok_or_else(|| invalido.clone())?;
    let url = format!("{pasta}/{relativo}");
    let (host, caminho) = url
        .strip_prefix("https://")
        .and_then(|r| r.split_once('?'))
        .and_then(|(antes, _)| antes.split_once('/'))
        .ok_or_else(|| invalido.clone())?;
    if !instalacao.autoriza_controlador(host, caminho) {
        return Err(invalido);
    }
    Ok(url)
}
