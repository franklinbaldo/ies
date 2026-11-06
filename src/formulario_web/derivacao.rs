//! Derivação de POST da tela servida, com destino original ou reescrito
//! por script observado (sem executar JavaScript). Refs #254.
use super::{
    CodificacaoDaTela, ConferenciaDaUnidade, EfeitoAvaliado, ErroSubmissaoFormulario,
    OperacaoDeFormulario, SubmissaoDerivada, TelaServida, codificar_corpo,
    conferir_unidade, ordenar_campos,
};
use super::leitura::{
    self, UrlHttps, conferir_metodo_e_codificacao, ler_controles,
    localizar_formulario, resolver_action,
};
use crate::html_atributos::nos_com_id;
use crate::instalacao::Instalacao;
use crate::transporte::FalhaTransporte;

/// Deriva a submissão do formulário servido, sem rede.
pub(crate) fn derivar_submissao(
    operacao: &OperacaoDeFormulario,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade_solicitada: &str,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    derivar_submissao_interna(operacao, instalacao, tela, unidade_solicitada, campos, None)
}

/// A operação provou, pelo JS servido e NÃO executado, que o navegador
/// reescreve a action do form. Confere TAMBÉM a action original, o destino
/// HTTPS observado e sua origem, antes de derivar os controles do form real.
pub(crate) fn derivar_submissao_com_action_do_script(
    operacao: &OperacaoDeFormulario,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade_solicitada: &str,
    acao_original: &str,
    destino_do_script: &str,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    derivar_submissao_interna(
        operacao,
        instalacao,
        tela,
        unidade_solicitada,
        campos,
        Some((acao_original, destino_do_script)),
    )
}

fn derivar_submissao_interna(
    operacao: &OperacaoDeFormulario,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade_solicitada: &str,
    campos: &[(&str, &str)],
    action_do_script: Option<(&str, &str)>,
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    use ErroSubmissaoFormulario as E;
    if operacao.efeito == EfeitoAvaliado::Desconhecido {
        return Err(E::EfeitoDaOperacaoNaoAvaliado);
    }
    if tela.html.len() > crate::MAX_HTML_BYTES {
        return Err(E::Transporte(FalhaTransporte::RespostaExcessiva));
    }
    let servida =
        UrlHttps::interpretar(tela.url).ok_or(E::Transporte(FalhaTransporte::UrlInvalida))?;
    if !instalacao.autoriza_controlador(servida.host, servida.caminho) {
        return Err(E::Transporte(FalhaTransporte::OrigemNaoAutorizada));
    }
    // tl só recusa entradas acima de u32::MAX bytes, já barradas acima.
    let dom = tl::parse(tela.html, tl::ParserOptions::default())
        .map_err(|_| E::Transporte(FalhaTransporte::RespostaExcessiva))?;
    match operacao.unidade {
        ConferenciaDaUnidade::NaTelaDoFormulario => conferir_unidade(&dom, unidade_solicitada)?,
        ConferenciaDaUnidade::NaTelaDeOrigemDoLink => {
            if !nos_com_id(&dom, "lnkInfraUnidade")?.is_empty() {
                conferir_unidade(&dom, unidade_solicitada)?;
            }
        }
    }
    let form = localizar_formulario(&dom, operacao.formulario)?;
    conferir_metodo_e_codificacao(form)?;
    let url = if let Some((acao_original, destino)) = action_do_script {
        let original = leitura::resolver_action_para_acao(
            form, &servida, instalacao, acao_original,
        )?;
        let original = UrlHttps::interpretar(&original)
            .ok_or(E::AcaoDoFormularioForaDaInstalacao)?;
        let alvo = UrlHttps::interpretar(destino)
            .ok_or(E::AcaoDoFormularioNaoEhUrlHttps)?;
        if !instalacao.autoriza_controlador(alvo.host, alvo.caminho)
            || !alvo.host.eq_ignore_ascii_case(servida.host)
            || alvo.caminho != original.caminho
        {
            return Err(E::AcaoDoFormularioForaDaInstalacao);
        }
        leitura::conferir_consulta(alvo.consulta.unwrap_or_default(), operacao.acao)?;
        destino.to_owned()
    } else {
        resolver_action(form, &servida, instalacao, operacao)?
    };
    let controles = ler_controles(form, dom.parser(), operacao)?;
    let ordenados = ordenar_campos(&controles, operacao, campos)?;
    let corpo = codificar_corpo(&ordenados, tela.codificacao)?;
    Ok(SubmissaoDerivada {
        url,
        host: servida.host.to_owned(),
        caminho: servida.caminho.to_owned(),
        acao: operacao.acao,
        referer: tela.url.to_owned(),
        nomes: ordenados.iter().map(|(n, _)| (*n).to_owned()).collect(),
        corpo: corpo.into_bytes(),
        efeito: operacao.efeito,
        resultado: operacao.resultado,
    })
}

