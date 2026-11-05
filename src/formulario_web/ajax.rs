//! Consulta AJAX que uma tela servida declara em JavaScript (o
//! `infraAjaxAutoCompletar` da tela "Enviar Processo"): POST ao
//! `controlador_ajax.php` da instalação, para a URL assinada que a tela
//! publicou, com o corpo que o `prepararExecucao` da tela monta. Sem o
//! formulário: os pares vêm da operação, na ordem do script. Só consultas
//! com efeito avaliado (`ConsultaSemEfeitoObservado`).
use super::leitura::UrlHttps;
use super::{
    EfeitoAvaliado, ErroSubmissaoFormulario, ResultadoDoEnvio, SubmissaoDerivada, TelaServida,
    codificar_corpo,
};
use crate::erro_html::ErroHtml;
use crate::instalacao::Instalacao;
use crate::transporte::{FalhaTransporte, conferir_chave_de_consulta, conferir_valor_de_consulta};

/// A consulta declarada pela operação (sem catálogo central).
#[derive(Debug)]
pub(crate) struct ConsultaAjax {
    pub(crate) acao_ajax: &'static str,
    pub(crate) campos: &'static [&'static str],
    pub(crate) efeito: EfeitoAvaliado,
}

/// Deriva o POST da consulta para `url_declarada` (relativa à tela, como o
/// script a publica), com `campos` exatamente na ordem dada.
pub(crate) fn derivar_consulta_ajax(
    consulta: &ConsultaAjax,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    url_declarada: &str,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    use ErroSubmissaoFormulario as E;
    if !matches!(
        consulta.efeito,
        EfeitoAvaliado::ConsultaSemEfeitoObservado { .. }
    ) {
        return Err(E::EfeitoDaOperacaoNaoAvaliado);
    }
    let servida = UrlHttps::interpretar(tela.url).ok_or(E::Transporte(FalhaTransporte::Html(
        ErroHtml::valor("url_invalida").com_mensagem("URL assinada da caixa inválida"),
    )))?;
    if !instalacao.autoriza_controlador(servida.host, servida.caminho) {
        return Err(E::Transporte(FalhaTransporte::OrigemNaoAutorizada));
    }
    // Só a forma servida: relativa à pasta da tela, sem esquema nem `..`.
    if url_declarada.contains(':')
        || url_declarada.starts_with('/')
        || url_declarada.split(['/', '?']).any(|s| s == "..")
    {
        return Err(ErroHtml::valor("acao_do_formulario_fora_da_instalacao").into());
    }
    let absoluta = match servida.caminho.rsplit_once('/') {
        Some((pasta, _)) => format!("https://{}/{pasta}/{url_declarada}", servida.host),
        None => format!("https://{}/{url_declarada}", servida.host),
    };
    let alvo = UrlHttps::interpretar(&absoluta).ok_or(E::AcaoDoFormularioNaoEhUrlHttps)?;
    if !instalacao.autoriza_controlador_ajax(alvo.host, alvo.caminho) {
        return Err(ErroHtml::valor("acao_do_formulario_fora_da_instalacao").into());
    }
    conferir_consulta_ajax(alvo.consulta.unwrap_or_default(), consulta.acao_ajax)?;
    for (i, (nome, _)) in campos.iter().enumerate() {
        let campo = (*nome).to_owned();
        if !consulta.campos.contains(nome) {
            return Err(E::CampoForaDaListaDaOperacao { campo });
        }
        if campos[..i].iter().any(|(anterior, _)| anterior == nome) {
            return Err(E::CampoInformadoMaisDeUmaVez { campo });
        }
    }
    let corpo = codificar_corpo(campos, tela.codificacao)?;
    Ok(SubmissaoDerivada {
        url: absoluta.clone(),
        host: alvo.host.to_owned(),
        caminho: alvo.caminho.to_owned(),
        acao: consulta.acao_ajax,
        referer: tela.url.to_owned(),
        nomes: campos.iter().map(|(n, _)| (*n).to_owned()).collect(),
        corpo: corpo.into_bytes(),
        efeito: consulta.efeito,
        resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    })
}

/// Uma `acao_ajax` igual à da consulta, um `infra_hash` de 64 hex, nenhuma
/// `acao`; chaves e valores canônicos.
fn conferir_consulta_ajax(consulta: &str, acao_ajax: &str) -> Result<(), ErroSubmissaoFormulario> {
    use ErroSubmissaoFormulario as E;
    let (mut acoes, mut hashes) = (0, 0);
    for item in consulta.split('&').filter(|item| !item.is_empty()) {
        let (nome, valor) = item
            .split_once('=')
            .ok_or(E::Transporte(FalhaTransporte::Html(
                ErroHtml::valor("url_invalida").com_mensagem("URL assinada da caixa inválida"),
            )))?;
        conferir_chave_de_consulta(nome)?;
        conferir_valor_de_consulta(valor)?;
        match nome {
            "acao_ajax" if valor == acao_ajax => acoes += 1,
            "acao_ajax" | "acao" => return Err(E::AcaoDoFormularioDiferenteDaOperacao),
            "infra_hash" => {
                if valor.len() != 64 || !valor.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err(E::AcaoDoFormularioSemInfraHashUnico);
                }
                hashes += 1;
            }
            _ => {}
        }
    }
    if acoes != 1 {
        return Err(E::AcaoDoFormularioDiferenteDaOperacao);
    }
    if hashes != 1 {
        return Err(E::AcaoDoFormularioSemInfraHashUnico);
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests_ajax.rs"]
mod tests;
