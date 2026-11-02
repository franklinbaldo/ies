//! Formulário `frmPesquisaProtocolo` observado offline nas instalações `unb` e `pen1`
//! (cassettes `processo-busca-*`, interação GET 1), apenas nomes e estrutura.
use super::redirecionamento::{ErroRedirecionamentoDaPesquisa, interpretar_redirecionamento};
use super::regras_formulario::{ErroRegraDoFormularioDePesquisa, ler_regras_da_tela};
use super::{ErroResultadoPesquisa, ResultadoPesquisa, ler_resultados};
use crate::dominio::IdProcedimento;
use crate::formulario_web::{
    EfeitoAvaliado, ErroSubmissaoFormulario, EstadoDeSessao, MarcadorDeResultado,
    OperacaoDeFormulario, RespostaDoEnvio, RespostaDoPost, SubmissaoDerivada, TelaServida,
    derivar_submissao, enviar_submissao_via, localizar_formulario,
};
use crate::instalacao::Instalacao;
use crate::transporte::FalhaTransporte;
use std::io::Read;
use std::path::Path;

/// O servidor pode lembrar página/filtro da sessão
/// (docs/como-o-sei-funciona/tabelas-e-paginacao.md): não é consulta pura.
pub(crate) const PESQUISAR_PROTOCOLO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "protocolo_pesquisar",
    formulario: "frmPesquisaProtocolo",
    botao_de_envio: "sbmPesquisar",
    campos_ocultos_exigidos: &[
        "hdnInfraTipoPagina",
        "hdnIdUnidade",
        "hdnIdAssunto",
        "hdnIdAssinante",
        "hdnIdContato",
        "hdnIdUsuarioGerador1",
        "hdnIdUsuarioGerador2",
        "hdnIdUsuarioGerador3",
        "hdnInicio",
    ],
    campos_usuario: &[
        "rdoPesquisarEm",
        "selOrgaoPesquisa[]",
        "txtProtocoloPesquisa",
    ],
    marcador_de_resultado: MarcadorDeResultado {
        elemento: "table",
        classe: "pesquisaResultado",
    },
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

/// Resultado tipado do POST de pesquisa: tabela de um 200 ou 302 interpretado.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ResultadoPesquisaProtocolo {
    Tabela(Vec<ResultadoPesquisa>),
    RedirecionadoParaProcessoUnico { id: IdProcedimento },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroPesquisaProtocolo {
    RegraDoFormulario(ErroRegraDoFormularioDePesquisa),
    Submissao(ErroSubmissaoFormulario),
    Redirecionamento(ErroRedirecionamentoDaPesquisa),
    Resultado(ErroResultadoPesquisa),
}

impl ErroPesquisaProtocolo {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::RegraDoFormulario(erro) => erro.subcodigo(),
            Self::Submissao(erro) => erro.subcodigo(),
            Self::Redirecionamento(erro) => erro.subcodigo(),
            Self::Resultado(erro) => erro.subcodigo(),
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
    let form = localizar_formulario(&dom, PESQUISAR_PROTOCOLO.formulario).map_err(E::Submissao)?;
    let regras = ler_regras_da_tela(form, dom.parser()).map_err(E::RegraDoFormulario)?;
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
    derivar_submissao(
        &PESQUISAR_PROTOCOLO,
        instalacao,
        tela,
        unidade_solicitada,
        &campos,
    )
    .map_err(E::Submissao)
}

/// Envia uma vez e interpreta a resposta; o 302 nunca é seguido.
pub(crate) fn pesquisar_protocolo_via<R: Read, F>(
    submissao: SubmissaoDerivada,
    instalacao: &Instalacao,
    cookie_file: &Path,
    enviar: F,
) -> Result<ResultadoPesquisaProtocolo, ErroPesquisaProtocolo>
where
    F: FnOnce(SubmissaoDerivada, &str) -> Result<RespostaDoPost<R>, FalhaTransporte>,
{
    match enviar_submissao_via(submissao, cookie_file, enviar)
        .map_err(ErroPesquisaProtocolo::Submissao)?
    {
        RespostaDoEnvio::Html(html) => ler_resultados(&html)
            .map(ResultadoPesquisaProtocolo::Tabela)
            .map_err(ErroPesquisaProtocolo::Resultado),
        RespostaDoEnvio::Redirecionamento(redirecionamento) => {
            interpretar_redirecionamento(redirecionamento, instalacao)
                .map(|id| ResultadoPesquisaProtocolo::RedirecionadoParaProcessoUnico { id })
                .map_err(ErroPesquisaProtocolo::Redirecionamento)
        }
    }
}

#[cfg(test)]
#[path = "tests_formulario_pesquisa.rs"]
mod tests;

#[cfg(test)]
#[path = "tests_pesquisa_envio.rs"]
mod tests_envio;
