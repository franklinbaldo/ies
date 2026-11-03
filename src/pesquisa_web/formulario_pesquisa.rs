//! Formulário `frmPesquisaProtocolo` observado offline nas instalações `unb` e `pen1`
//! (cassettes `processo-busca-*`, interação GET 1), apenas nomes e estrutura.
use super::redirecionamento::{
    ErroRedirecionamentoDaPesquisa, UrlDeTrabalho, interpretar_redirecionamento,
};
use super::regras_formulario::{ErroRegraDoFormularioDePesquisa, ler_regras_da_tela};
use super::resposta::{ErroRespostaDaPesquisa, RespostaDaPesquisa, ler_resposta_html};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDeSessao, MarcadorDeResultado, OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio,
    SubmissaoDerivada, TelaServida, derivar_submissao, localizar_formulario,
};
#[cfg(test)]
use crate::formulario_web::{RespostaDoPost, enviar_submissao_via};
use crate::instalacao::Instalacao;
use crate::transporte::FalhaTransporte;
#[cfg(test)]
use std::{io::Read, path::Path};

/// Ocultos exigidos na tela da pesquisa e na tela de resultado.
pub(crate) const CAMPOS_OCULTOS_DA_PESQUISA: &[&str] = &[
    "hdnInfraTipoPagina",
    "hdnIdUnidade",
    "hdnIdAssunto",
    "hdnIdAssinante",
    "hdnIdContato",
    "hdnIdUsuarioGerador1",
    "hdnIdUsuarioGerador2",
    "hdnIdUsuarioGerador3",
    "hdnInicio",
];

/// Controles de usuário que a operação preenche (rádio, órgãos e termo).
pub(crate) const CAMPOS_USUARIO_DA_PESQUISA: &[&str] = &[
    "rdoPesquisarEm",
    "selOrgaoPesquisa[]",
    "txtProtocoloPesquisa",
];

/// Respostas 200 reconhecidas: a tabela de resultado ou o aviso de que a
/// pesquisa não devolveu nada (`UnB` 4.0.9, 03/11/2026).
pub(crate) const MARCADORES_DA_RESPOSTA: ResultadoDoEnvio = ResultadoDoEnvio::UmDosMarcadores(&[
    MarcadorDeResultado {
        elemento: "table",
        classe: "pesquisaResultado",
    },
    MarcadorDeResultado {
        elemento: "div",
        classe: "pesquisaSemResultado",
    },
]);

/// O servidor pode lembrar página/filtro da sessão
/// (docs/como-o-sei-funciona/tabelas-e-paginacao.md): não é consulta pura.
pub(crate) const PESQUISAR_PROTOCOLO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "protocolo_pesquisar",
    formulario: "frmPesquisaProtocolo",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmPesquisar"),
    campos_ocultos_exigidos: CAMPOS_OCULTOS_DA_PESQUISA,
    campos_ocultos_reescritos: &[],
    campos_usuario: CAMPOS_USUARIO_DA_PESQUISA,
    resultado: MARCADORES_DA_RESPOSTA,
    unidade: ConferenciaDaUnidade::NaTelaDoFormulario,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroPesquisaProtocolo {
    RegraDoFormulario(ErroRegraDoFormularioDePesquisa),
    Submissao(ErroSubmissaoFormulario),
    Redirecionamento(ErroRedirecionamentoDaPesquisa),
    Resposta(ErroRespostaDaPesquisa),
}

impl ErroPesquisaProtocolo {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::RegraDoFormulario(erro) => erro.subcodigo(),
            Self::Submissao(erro) => erro.subcodigo(),
            Self::Redirecionamento(erro) => erro.subcodigo(),
            Self::Resposta(erro) => erro.subcodigo(),
        }
    }

    pub(crate) fn status_http(&self) -> Option<&crate::transporte::StatusHttpRecusado> {
        match self {
            Self::Submissao(erro) => erro.status_http(),
            _ => None,
        }
    }
}

/// Deriva o POST da pesquisa por protocolo, sem rede. O rádio e os órgãos
/// vêm da tela servida pelas regras de `regras_formulario`; o chamador só
/// informa o protocolo.
pub(crate) fn derivar_pesquisa_de_protocolo(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade_solicitada: &str,
    protocolo: &str,
) -> Result<SubmissaoDerivada, ErroPesquisaProtocolo> {
    derivar_com(
        &PESQUISAR_PROTOCOLO,
        instalacao,
        tela,
        unidade_solicitada,
        protocolo,
        None,
    )
}

/// Deriva o POST de `navegar(inicio)` a partir da tela de resultado: mesmas
/// regras do `onsubmit`, sem botão, com `hdnInicio = inicio`.
pub(crate) fn derivar_pagina_da_pesquisa(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade_solicitada: &str,
    protocolo: &str,
    inicio: u64,
) -> Result<SubmissaoDerivada, ErroPesquisaProtocolo> {
    let inicio = inicio.to_string();
    derivar_com(
        &super::paginacao::PAGINAR_PESQUISA_DE_PROTOCOLO,
        instalacao,
        tela,
        unidade_solicitada,
        protocolo,
        Some(&inicio),
    )
}

fn derivar_com(
    operacao: &OperacaoDeFormulario,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade_solicitada: &str,
    protocolo: &str,
    inicio: Option<&str>,
) -> Result<SubmissaoDerivada, ErroPesquisaProtocolo> {
    use ErroPesquisaProtocolo as E;
    let excessiva = || {
        E::Submissao(ErroSubmissaoFormulario::Transporte(
            FalhaTransporte::RespostaExcessiva,
        ))
    };
    if tela.html.len() > crate::MAX_HTML_BYTES {
        return Err(excessiva());
    }
    // As recusas de instalação e unidade vêm de `derivar_submissao`, depois
    // destas regras; ler a tela em memória não tem efeito.
    let dom = tl::parse(tela.html, tl::ParserOptions::default()).map_err(|_| excessiva())?;
    let form = localizar_formulario(&dom, operacao.formulario).map_err(E::Submissao)?;
    let regras = ler_regras_da_tela(form, dom.parser()).map_err(E::RegraDoFormulario)?;
    // O navegador conta `maxlength` em unidades UTF-16; termo maior não
    // seria digitável na tela servida e não sai em POST.
    if let Some(maxlength) = regras.maxlength_do_termo
        && protocolo.encode_utf16().count() > maxlength as usize
    {
        return Err(E::RegraDoFormulario(
            ErroRegraDoFormularioDePesquisa::TermoAcimaDoMaxlengthDaTela { maxlength },
        ));
    }
    let mut campos = vec![
        ("rdoPesquisarEm", regras.opcao_de_processos.as_str()),
        ("txtProtocoloPesquisa", protocolo),
    ];
    campos.extend(
        regras
            .orgaos_selecionados
            .iter()
            .map(|valor| ("selOrgaoPesquisa[]", valor.as_str())),
    );
    if let Some(inicio) = inicio {
        campos.push(("hdnInicio", inicio));
    }
    derivar_submissao(operacao, instalacao, tela, unidade_solicitada, &campos).map_err(E::Submissao)
}

/// Resposta classificada, o HTML 200 (para a paginação) e, no 302, a URL
/// de trabalho conferida (só `ies processo NUP` a abre).
pub(crate) struct EnvioInterpretado {
    pub(crate) resposta: RespostaDaPesquisa,
    pub(crate) html: Option<String>,
    pub(crate) trabalho: Option<UrlDeTrabalho>,
}

/// A resposta do envio interpretada: o 302 vira o identificador e a URL
/// conferida (nunca seguida aqui) e o 200 é classificado.
pub(crate) fn interpretar_envio(
    resposta: RespostaDoEnvio,
    instalacao: &Instalacao,
) -> Result<EnvioInterpretado, ErroPesquisaProtocolo> {
    match resposta {
        RespostaDoEnvio::Html(html) => ler_resposta_html(&html)
            .map(|resposta| EnvioInterpretado {
                resposta,
                html: Some(html),
                trabalho: None,
            })
            .map_err(ErroPesquisaProtocolo::Resposta),
        RespostaDoEnvio::Redirecionamento(redirecionamento) => {
            interpretar_redirecionamento(redirecionamento, instalacao)
                .map(|p| EnvioInterpretado {
                    resposta: RespostaDaPesquisa::ProcessoUnico { id: p.id },
                    html: None,
                    trabalho: Some(p.trabalho),
                })
                .map_err(ErroPesquisaProtocolo::Redirecionamento)
        }
    }
}

/// Falha do envio; sem marcador reconhecido (nem tabela, nem aviso de
/// "sem resultado", nem validação) é `resposta_da_pesquisa_nao_reconhecida`.
pub(crate) fn falha_do_envio(erro: ErroSubmissaoFormulario) -> ErroPesquisaProtocolo {
    match erro {
        ErroSubmissaoFormulario::MarcadorDeResultadoAusenteNaResposta => {
            ErroPesquisaProtocolo::Resposta(
                ErroRespostaDaPesquisa::RespostaDaPesquisaNaoReconhecida,
            )
        }
        outro => ErroPesquisaProtocolo::Submissao(outro),
    }
}

/// Envia uma vez e interpreta a resposta; o 302 nunca é seguido. O fluxo do
/// comando usa as mesmas peças (`interpretar_envio`) guardando o HTML.
#[cfg(test)]
pub(crate) fn pesquisar_protocolo_via<R: Read, F>(
    submissao: SubmissaoDerivada,
    instalacao: &Instalacao,
    cookie_file: &Path,
    enviar: F,
) -> Result<RespostaDaPesquisa, ErroPesquisaProtocolo>
where
    F: FnOnce(SubmissaoDerivada, &str) -> Result<RespostaDoPost<R>, FalhaTransporte>,
{
    let resposta = enviar_submissao_via(submissao, cookie_file, enviar).map_err(falha_do_envio)?;
    interpretar_envio(resposta, instalacao).map(|e| e.resposta)
}

#[cfg(test)]
#[path = "tests_formulario_pesquisa.rs"]
mod tests;

#[cfg(test)]
#[path = "tests_pesquisa_envio.rs"]
mod tests_envio;
