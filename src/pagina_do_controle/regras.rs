//! Controles não ocultos de `frmProcedimentoControlar`, lidos como o
//! navegador os enviaria num `submit()` por script. Inventário estrutural de
//! 02/11/2026 (PEN1, PEN3, PEN4, Goiás, UnB): `txtBloco`, caixas de seleção
//! `chk{T}Item*` desmarcadas e, quando há várias páginas, `sel{T}Paginacao*`.
//! Nada de JavaScript é executado ou interpretado aqui.
use crate::html_atributos::ErroAtributoHtml;

/// Únicos controles de usuário que o POST do paginador repete.
pub(super) const CAMPOS_USUARIO: &[&str] = &[
    "txtBloco",
    "selRecebidosPaginacaoSuperior",
    "selRecebidosPaginacaoInferior",
    "selGeradosPaginacaoSuperior",
    "selGeradosPaginacaoInferior",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroControleDoFormulario {
    ControleNaoPrevistoNoFormularioDoControle { campo: String },
    CaixaDeSelecaoMarcadaNoFormularioDoControle { campo: String },
    PaginadorSemOpcaoSelecionada { campo: String },
    ControleRepetidoNoFormularioDoControle { campo: String },
    EntidadeHtmlDesconhecidaNoControle { campo: String },
    AtributoHtml(ErroAtributoHtml),
}

impl ErroControleDoFormulario {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::ControleNaoPrevistoNoFormularioDoControle { .. } => {
                "controle_nao_previsto_no_formulario_do_controle"
            }
            Self::CaixaDeSelecaoMarcadaNoFormularioDoControle { .. } => {
                "caixa_de_selecao_marcada_no_formulario_do_controle"
            }
            Self::PaginadorSemOpcaoSelecionada { .. } => "paginador_sem_opcao_selecionada",
            Self::ControleRepetidoNoFormularioDoControle { .. } => {
                "controle_repetido_no_formulario_do_controle"
            }
            Self::EntidadeHtmlDesconhecidaNoControle { .. } => {
                "entidade_html_desconhecida_no_controle"
            }
            Self::AtributoHtml(falha) => falha.subcodigo(),
        }
    }
}

/// Pares (nome, valor) dos controles de usuário, na ordem do documento.
/// Shim CONTRACT.
pub(super) fn campos_do_usuario(
    _form: &tl::HTMLTag<'_>,
    _parser: &tl::Parser<'_>,
) -> Result<Vec<(String, String)>, ErroControleDoFormulario> {
    Ok(Vec::new())
}
