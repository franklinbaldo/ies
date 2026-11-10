//! Projeções das falhas de assinatura; nenhuma operação ou classificação nova.
use crate::arvore_processo::Assinante;
use crate::documento_assinar::{ErroAssinar, FalhaDoEnvio, VerificacaoDaAssinatura};
use crate::processo_concluir_cli::falha::FalhaDaSituacao;

#[derive(serde::Serialize)]
pub(super) struct AssinantePublico<'a> {
    cargo: &'a str,
    nome: &'a str,
    unidade: &'a str,
}

pub(super) fn assinaturas_publicas(lista: &[Assinante]) -> Vec<AssinantePublico<'_>> {
    lista
        .iter()
        .map(|a| AssinantePublico {
            cargo: &a.cargo,
            nome: &a.nome,
            unidade: &a.unidade,
        })
        .collect()
}

pub(super) fn falha_do_envio_publica(falha: &FalhaDoEnvio) -> crate::saida_erro::FalhaPublica {
    crate::saida_erro::FalhaPublica::nova(falha.status_http())
}

/// Os campos próprios precedem `codigo` na ordem histórica do contrato.
#[derive(serde::Serialize)]
pub(super) struct FalhaAssinatura<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    assinaturas: Option<Vec<AssinantePublico<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assinaturas_depois: Option<Vec<AssinantePublico<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cargos: Option<&'a [String]>,
    #[serde(flatten)]
    dados: FalhaDaSituacao<'a>,
}

pub(super) fn json_de_falha(erro: &ErroAssinar) -> FalhaAssinatura<'_> {
    let mut valor = FalhaAssinatura {
        assinaturas: None,
        assinaturas_depois: None,
        cargos: None,
        dados: FalhaDaSituacao::nova(erro.codigo(), erro.mensagem(), erro.status_http()),
    };
    match erro {
        ErroAssinar::CargoNaoOferecidoPelaTela { cargos } => valor.cargos = Some(cargos),
        ErroAssinar::DocumentoAssinadoPorOutroUsuario { antes } => {
            valor.assinaturas = Some(assinaturas_publicas(antes));
        }
        ErroAssinar::AssinaturaNaoAplicada {
            antes,
            falha_do_envio,
        } => {
            valor.assinaturas = Some(assinaturas_publicas(antes));
            valor.dados.falha_do_envio = falha_do_envio.as_ref().map(falha_do_envio_publica);
        }
        ErroAssinar::EfeitoIncertoNaAssinatura {
            antes,
            falha_do_envio,
            verificacao,
        } => {
            valor.assinaturas = Some(assinaturas_publicas(antes));
            if let VerificacaoDaAssinatura::Lida(depois) = verificacao.as_ref() {
                valor.assinaturas_depois = Some(assinaturas_publicas(depois));
            }
            valor.dados.verificacao = Some(verificacao.motivo());
            valor.dados.falha_do_envio = falha_do_envio.as_ref().map(falha_do_envio_publica);
        }
        _ => {}
    }
    valor
}

/// A saída humana parte do erro original, sem ler sua apresentação JSON.
pub(super) fn linhas_da_falha(erro: &ErroAssinar) -> Vec<String> {
    let mut linhas = vec![format!("IES: {}", erro.mensagem())];
    if let ErroAssinar::CargoNaoOferecidoPelaTela { cargos } = erro {
        linhas.push(format!("IES: cargos da tela: {}", cargos.join("; ")));
    }
    let falha_do_envio = match erro {
        ErroAssinar::AssinaturaNaoAplicada { falha_do_envio, .. }
        | ErroAssinar::EfeitoIncertoNaAssinatura { falha_do_envio, .. } => falha_do_envio.as_ref(),
        _ => None,
    };
    if let Some(falha) = falha_do_envio {
        linhas.push(format!("IES: resposta da escrita: {falha:?}"));
    }
    if let ErroAssinar::EfeitoIncertoNaAssinatura { verificacao, .. } = erro
        && let Some(motivo) = verificacao.motivo()
    {
        linhas.push(format!("IES: verificação: {motivo}"));
    }
    linhas
}
