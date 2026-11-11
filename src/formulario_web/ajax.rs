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
use crate::acao_do_sei::{AcaoAjaxDoSei, AcaoDaSubmissao};
use crate::instalacao::Instalacao;
use crate::transporte::{FalhaTransporte, conferir_chave_de_consulta, conferir_valor_de_consulta};

/// A consulta declarada pela operação (sem catálogo central).
#[derive(Debug)]
pub(crate) struct ConsultaAjax {
    pub(crate) acao_ajax: AcaoAjaxDoSei,
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
    if !matches!(
        consulta.efeito,
        EfeitoAvaliado::ConsultaSemEfeitoObservado { .. }
    ) {
        return Err(ErroSubmissaoFormulario::EfeitoDaOperacaoNaoAvaliado);
    }
    let servida = UrlHttps::interpretar(tela.url).ok_or(ErroSubmissaoFormulario::Transporte(
        FalhaTransporte::UrlInvalida,
    ))?;
    if !instalacao.autoriza_controlador(servida.host, servida.caminho) {
        return Err(ErroSubmissaoFormulario::Transporte(
            FalhaTransporte::OrigemNaoAutorizada,
        ));
    }
    // Só a forma servida: relativa à pasta da tela, sem esquema nem `..`.
    if url_declarada.contains(':')
        || url_declarada.starts_with('/')
        || url_declarada.split(['/', '?']).any(|s| s == "..")
    {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao);
    }
    let absoluta = match servida.caminho.rsplit_once('/') {
        Some((pasta, _)) => format!("https://{}/{pasta}/{url_declarada}", servida.host),
        None => format!("https://{}/{url_declarada}", servida.host),
    };
    let alvo = UrlHttps::interpretar(&absoluta)
        .ok_or(ErroSubmissaoFormulario::AcaoDoFormularioNaoEhUrlHttps)?;
    if !instalacao.autoriza_controlador_ajax(alvo.host, alvo.caminho) {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao);
    }
    conferir_consulta_ajax(alvo.consulta.unwrap_or_default(), consulta.acao_ajax)?;
    for (i, (nome, _)) in campos.iter().enumerate() {
        let campo = (*nome).to_owned();
        if !consulta.campos.contains(nome) {
            return Err(ErroSubmissaoFormulario::CampoForaDaListaDaOperacao { campo });
        }
        if campos.iter().take(i).any(|(anterior, _)| anterior == nome) {
            return Err(ErroSubmissaoFormulario::CampoInformadoMaisDeUmaVez { campo });
        }
    }
    let corpo = codificar_corpo(campos, tela.codificacao)?;
    Ok(SubmissaoDerivada {
        url: absoluta.clone(),
        host: alvo.host.to_owned(),
        caminho: alvo.caminho.to_owned(),
        acao: AcaoDaSubmissao::Ajax(consulta.acao_ajax),
        referer: tela.url.to_owned(),
        nomes: campos.iter().map(|(n, _)| (*n).to_owned()).collect(),
        corpo: corpo.into_bytes(),
        efeito: consulta.efeito,
        resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    })
}

/// Uma `acao_ajax` igual à da consulta, um `infra_hash` de 64 hex, nenhuma
/// `acao`; chaves e valores canônicos.
fn conferir_consulta_ajax(
    consulta: &str,
    acao_ajax: AcaoAjaxDoSei,
) -> Result<(), ErroSubmissaoFormulario> {
    let (mut acoes, mut hashes) = (0, 0);
    for item in consulta.split('&').filter(|item| !item.is_empty()) {
        let (nome, valor) = item
            .split_once('=')
            .ok_or(ErroSubmissaoFormulario::Transporte(
                FalhaTransporte::UrlInvalida,
            ))?;
        conferir_chave_de_consulta(nome)?;
        conferir_valor_de_consulta(valor)?;
        match nome {
            "acao_ajax" if AcaoAjaxDoSei::ler(valor) == Some(acao_ajax) => acoes += 1,
            "acao_ajax" | "acao" => {
                return Err(ErroSubmissaoFormulario::AcaoDoFormularioDiferenteDaOperacao);
            }
            "infra_hash" => {
                if valor.len() != 64 || !valor.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err(ErroSubmissaoFormulario::AcaoDoFormularioSemInfraHashUnico);
                }
                hashes += 1;
            }
            _ => {}
        }
    }
    if acoes != 1 {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioDiferenteDaOperacao);
    }
    if hashes != 1 {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioSemInfraHashUnico);
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests_ajax.rs"]
mod tests;
