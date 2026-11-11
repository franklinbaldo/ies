//! Envio para o `action` assinado que uma tela da **mesma sessão** já
//! publicou e o índice local guardou literal (ADR 0003), sem o HTML do
//! formulário. Só serve a operações enviadas por script, sem ocultos
//! exigidos nem botão: o corpo é exatamente o dos campos do usuário.
//! O charset da tela não foi guardado, então só valores ASCII imprimíveis
//! são aceitos (iguais em ISO-8859-1 e UTF-8).
use super::leitura::{UrlHttps, conferir_consulta};
use super::{
    CodificacaoDaTela, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    OperacaoDeFormulario, SubmissaoDerivada, codificar_corpo,
};
use crate::instalacao::Instalacao;
use crate::transporte::FalhaTransporte;

/// Deriva o POST para `url_guardada` com `campos` na ordem dada.
/// `referer` é a tela da sessão que publica o formulário (o Controle).
pub(crate) fn derivar_submissao_guardada(
    operacao: &OperacaoDeFormulario,
    instalacao: &Instalacao,
    url_guardada: &str,
    referer: &str,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    if operacao.efeito == EfeitoAvaliado::Desconhecido {
        return Err(ErroSubmissaoFormulario::EfeitoDaOperacaoNaoAvaliado);
    }
    // Sem o formulário não há ocultos nem botão para reproduzir.
    if operacao.envio != EnvioDoFormulario::SubmitPorScript
        || !operacao.campos_ocultos_exigidos.is_empty()
        || !operacao.campos_ocultos_reescritos.is_empty()
    {
        return Err(ErroSubmissaoFormulario::FormularioAusenteNaTela);
    }
    let alvo = UrlHttps::interpretar(url_guardada)
        .ok_or(ErroSubmissaoFormulario::AcaoDoFormularioNaoEhUrlHttps)?;
    if !instalacao.autoriza_controlador(alvo.host, alvo.caminho) {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao);
    }
    conferir_consulta(alvo.consulta.unwrap_or_default(), operacao.acao)?;
    let servida = UrlHttps::interpretar(referer).ok_or(ErroSubmissaoFormulario::Transporte(
        FalhaTransporte::UrlInvalida,
    ))?;
    if !instalacao.autoriza_controlador(servida.host, servida.caminho) {
        return Err(ErroSubmissaoFormulario::Transporte(
            FalhaTransporte::OrigemNaoAutorizada,
        ));
    }
    for (i, (nome, valor)) in campos.iter().enumerate() {
        let campo = (*nome).to_owned();
        if !operacao.campos_usuario.contains(nome) {
            return Err(ErroSubmissaoFormulario::CampoForaDaListaDaOperacao { campo });
        }
        if campos.iter().take(i).any(|(anterior, _)| anterior == nome) {
            return Err(ErroSubmissaoFormulario::CampoInformadoMaisDeUmaVez { campo });
        }
        if !valor.bytes().all(|b| (32..=126).contains(&b)) {
            return Err(ErroSubmissaoFormulario::ValorNaoRepresentavelNaCodificacao { campo });
        }
    }
    let corpo = codificar_corpo(campos, CodificacaoDaTela::Utf8)?;
    Ok(SubmissaoDerivada {
        url: url_guardada.to_owned(),
        host: alvo.host.to_owned(),
        caminho: alvo.caminho.to_owned(),
        acao: operacao.acao,
        referer: referer.to_owned(),
        nomes: campos.iter().map(|(n, _)| (*n).to_owned()).collect(),
        corpo: corpo.into_bytes(),
        efeito: operacao.efeito,
        resultado: operacao.resultado,
    })
}

#[cfg(test)]
#[path = "tests_guardada.rs"]
mod tests;
