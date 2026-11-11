//! Recusas tipadas de documento cancelar. Nenhum tipo armazena URL ou segredo.
use super::ErroLinkCancelar;
use crate::arvore_processo::Cancelamento;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_obter::ErroDocumentoObter;
use crate::transporte::FalhaTransporte;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FalhaDoCancelamento {
    Envio(ErroSubmissaoFormulario),
    RespostaSemRedirecionamento,
    RedirecionamentoNaoReconhecido,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum VerificacaoDoCancelamento {
    Lida {
        cancelamento: Cancelamento,
    },
    LeituraFalhou(ErroDocumentoObter),
    /// A releitura abriu outro documento: a escrita não pode ser atribuída ao pedido.
    DocumentoDiferenteNaReleitura,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCancelar {
    /// O cookie privado da sessão não pôde ser lido.
    Cookie(#[source] CookieIlegivel),
    Abertura(#[source] ErroDocumentoObter),
    Link(#[source] ErroLinkCancelar),
    CancelamentoNaoOferecido,
    DocumentoJaCancelado,
    Tela(#[source] FalhaTransporte),
    TelaRedirecionada,
    SessaoEncerrada,
    CharsetNaoSuportado,
    Formulario(#[source] ErroSubmissaoFormulario),
    FormularioDeOutroDocumento,
    /// O endereço do formulário não aponta para o processo aberto.
    EnderecoDoProcessoRecusado(#[source] crate::arvore_processo::endereco::EnderecoRecusado),
    /// O endereço do formulário não aponta para o documento pedido.
    EnderecoDoDocumentoRecusado(#[source] crate::arvore_processo::endereco::EnderecoRecusado),
    MotivoNaoPublicadoNaTela,
    /// A unidade da escrita não foi publicada pela árvore nem pela sessão.
    UnidadeNaoPublicada,
    /// A leitura nova confirma ausência inequívoca do cancelamento.
    CancelamentoNaoAplicado {
        falha_do_envio: Option<FalhaDoCancelamento>,
    },
    /// Depois da escrita houve mudança ambígua ou a leitura não pôde decidir.
    NaoConfirmado {
        falha_do_envio: Option<FalhaDoCancelamento>,
        verificacao: Box<VerificacaoDoCancelamento>,
    },
}

impl From<CookieIlegivel> for ErroCancelar {
    fn from(erro: CookieIlegivel) -> Self {
        Self::Cookie(erro)
    }
}

#[cfg(test)]
mod testes_cadeia_de_causas {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn erros_de_cancelamento_retêm_as_causas_tipadas() {
        let atributo =
            crate::html_atributos::ErroAtributoHtml::AtributoRepetidoNaTag { atributo: "href" };
        let exemplos = [
            ErroCancelar::Cookie(CookieIlegivel::Arquivo(FalhaTransporte::Rede)),
            ErroCancelar::Abertura(ErroDocumentoObter::Processo(
                crate::processo_obter::ErroProcessoObter::Transporte(FalhaTransporte::Rede),
            )),
            ErroCancelar::Link(ErroLinkCancelar::Atributo(atributo)),
            ErroCancelar::Tela(FalhaTransporte::Rede),
            ErroCancelar::Formulario(ErroSubmissaoFormulario::AtributoHtml(atributo)),
            ErroCancelar::EnderecoDoProcessoRecusado(
                crate::arvore_processo::endereco::EnderecoRecusado::NaoReconhecido,
            ),
            ErroCancelar::EnderecoDoDocumentoRecusado(
                crate::arvore_processo::endereco::EnderecoRecusado::DeOutroProcesso,
            ),
        ];
        for erro in exemplos {
            assert!(erro.source().is_some(), "perdeu a causa: {erro:?}");
        }
        let link = ErroLinkCancelar::Atributo(atributo);
        assert!(
            link.source()
                .and_then(|fonte| fonte.downcast_ref::<crate::html_atributos::ErroAtributoHtml>())
                .is_some(),
            "a falha da tag deve sobreviver também ao leitor do link"
        );
    }
}
