//! Formulário `frmPesquisaProtocolo` observado offline nas instalações `unb` e `pen1`
//! (cassettes `processo-busca-*`, interação GET 1), apenas nomes e estrutura.
use super::redirecionamento::{ErroRedirecionamentoDaPesquisa, interpretar_redirecionamento};
use super::{ErroResultadoPesquisa, ResultadoPesquisa};
use crate::dominio::IdProcedimento;
use crate::formulario_web::{
    EfeitoAvaliado, ErroSubmissaoFormulario, EstadoDeSessao, OperacaoDeFormulario, RespostaDoPost,
    SubmissaoDerivada,
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
    campos_usuario: &["rdoPesquisarEm", "txtProtocoloPesquisa"],
    marcador_de_resultado: "table.pesquisaResultado",
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
    Submissao(ErroSubmissaoFormulario),
    Redirecionamento(ErroRedirecionamentoDaPesquisa),
    Resultado(ErroResultadoPesquisa),
}

impl ErroPesquisaProtocolo {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Submissao(erro) => erro.subcodigo(),
            Self::Redirecionamento(erro) => erro.subcodigo(),
            Self::Resultado(erro) => erro.subcodigo(),
        }
    }
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
    let _ = (submissao, instalacao, cookie_file, enviar);
    let _ = interpretar_redirecionamento;
    unimplemented!("CONTRACT shim: pesquisa de protocolo")
}

#[cfg(test)]
#[path = "tests_formulario_pesquisa.rs"]
mod tests;

#[cfg(test)]
#[path = "tests_pesquisa_envio.rs"]
mod tests_envio;
