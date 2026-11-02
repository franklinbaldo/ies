//! Leitura da tela `procedimento_configurar_detalhe` (GET, somente leitura).
//! Nada é salvo: `sbmSalvar` não é enviado por este módulo.
use super::{ColunaDoDetalhe, NivelDeDetalhe};
use crate::html_atributos::ErroAtributoHtml;

/// Formulário observado (o `id` é o publicado pelo SEI, apesar do nome).
pub(crate) const FORMULARIO_DO_NIVEL_DE_DETALHE: &str = "frmAcessoExternoCancelar";
pub(crate) const ACAO_DO_NIVEL_DE_DETALHE: &str = "procedimento_configurar_detalhe";

/// Uma variante por fato verificável da tela de configuração.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroNivelDeDetalhe {
    HtmlNaoInterpretado,
    FormularioDoNivelDeDetalheAusente,
    FormulariosDoNivelDeDetalheRepetidos,
    AcaoDoFormularioDoNivelDeDetalheDiferente,
    OpcaoDoDetalheAusente { coluna: ColunaDoDetalhe },
    OpcaoDoDetalheRepetida { coluna: ColunaDoDetalhe },
    OpcaoDoDetalheNaoPrevista,
    RotuloDaOpcaoDoDetalheDiferente { coluna: ColunaDoDetalhe },
    AtributoHtml(ErroAtributoHtml),
}

impl ErroNivelDeDetalhe {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlNaoInterpretado => "html_do_nivel_de_detalhe_nao_interpretado",
            Self::FormularioDoNivelDeDetalheAusente => "formulario_do_nivel_de_detalhe_ausente",
            Self::FormulariosDoNivelDeDetalheRepetidos => {
                "formularios_do_nivel_de_detalhe_repetidos"
            }
            Self::AcaoDoFormularioDoNivelDeDetalheDiferente => {
                "acao_do_formulario_do_nivel_de_detalhe_diferente"
            }
            Self::OpcaoDoDetalheAusente { .. } => "opcao_do_nivel_de_detalhe_ausente",
            Self::OpcaoDoDetalheRepetida { .. } => "opcao_do_nivel_de_detalhe_repetida",
            Self::OpcaoDoDetalheNaoPrevista => "opcao_do_nivel_de_detalhe_nao_prevista",
            Self::RotuloDaOpcaoDoDetalheDiferente { .. } => {
                "rotulo_da_opcao_do_nivel_de_detalhe_diferente"
            }
            Self::AtributoHtml(falha) => falha.subcodigo(),
        }
    }
}

/// As 12 opções e as marcadas, exatamente como a tela as publica.
pub(crate) fn ler_nivel_de_detalhe(html: &str) -> Result<NivelDeDetalhe, ErroNivelDeDetalhe> {
    let _ = html;
    Err(ErroNivelDeDetalhe::FormularioDoNivelDeDetalheAusente)
}
