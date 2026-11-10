//! Resultado e recusas de `ies unidade colunas --definir`
//! (contracts/cli/unidade-definir-colunas.md). Uma variante por fato.
use super::ErroVisaoDoControle;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::nivel_de_detalhe::{ColunaDoDetalhe, PlanoDasColunas};
use crate::transporte::StatusHttpRecusado;

/// Por que a resposta do POST de salvar não é a esperada (HTML 200 sem
/// tela de login, com o sinal de gravação). O pedido saiu: o efeito é
/// incerto até a leitura.
#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum FalhaDoSalvar {
    /// Rede, tempo esgotado, 5xx, outro status, MIME, charset, tamanho, login.
    #[causa(interna)]
    Envio(ErroSubmissaoFormulario),
    /// 302 recebido e não seguido.
    Redirecionamento302,
    /// HTML 200 com a tela do Nível sem o sinal de gravação (`''=='1'`).
    NivelDevolvidoSemSinalDeGravacao,
    /// HTML 200 sem o `inicializar()` observado da tela do Nível.
    RespostaDoSalvarNaoReconhecida,
}

impl FalhaDoSalvar {
    /// Status e mensagem do SEI, quando a resposta foi uma recusa HTTP.
    pub(crate) fn status_http(&self) -> Option<StatusHttpRecusado> {
        match self {
            Self::Envio(erro) => erro.status_http().cloned(),
            Self::Redirecionamento302 => Some(StatusHttpRecusado::sem_mensagem(302)),
            Self::NivelDevolvidoSemSinalDeGravacao | Self::RespostaDoSalvarNaoReconhecida => None,
        }
    }
}

/// O que a leitura de verificação, feita uma vez, mostrou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum VerificacaoDasColunas {
    Marcadas(Vec<ColunaDoDetalhe>),
    LeituraFalhou(ErroVisaoDoControle),
}

/// Sucesso: prévia, alvo igual ao atual, ou salvo e conferido.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ColunasDefinidas {
    pub(crate) unidade: String,
    pub(crate) plano: PlanoDasColunas,
    pub(crate) confirmado: bool,
    /// O POST de salvar foi enviado e a leitura mostrou o alvo.
    pub(crate) alterado: bool,
    /// Nomes dos campos do POST, previsto (prévia) ou enviado.
    pub(crate) campos_do_envio: Option<Vec<String>>,
    /// Marcas lidas na verificação (só depois de salvar).
    pub(crate) depois: Option<Vec<ColunaDoDetalhe>>,
    /// A resposta do salvar não foi a esperada, e a leitura mostrou o alvo.
    pub(crate) reconciliado_por: Option<FalhaDoSalvar>,
}

#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroDefinirColunas {
    /// Falha antes do POST de salvar: nada foi salvo.
    #[causa(interna)]
    Leitura(ErroVisaoDoControle),
    ColunaIndisponivelNestaInstalacao {
        coluna: ColunaDoDetalhe,
    },
    /// A leitura de verificação mostrou as marcas de antes.
    ConfiguracaoDeColunasNaoAplicada {
        plano: Box<PlanoDasColunas>,
        falha_do_salvar: Option<FalhaDoSalvar>,
    },
    /// Resposta esperada, mas a leitura mostrou outras marcas (nem o alvo
    /// nem as de antes).
    MarcasDiferentesDoAlvoAposSalvar {
        plano: Box<PlanoDasColunas>,
        observadas: Vec<ColunaDoDetalhe>,
    },
    /// Saída 3: o POST saiu e a leitura não decide o efeito.
    NaoConfirmadoNaConfiguracaoDeColunas {
        plano: Box<PlanoDasColunas>,
        falha_do_salvar: Option<FalhaDoSalvar>,
        verificacao: Box<VerificacaoDasColunas>,
    },
}

impl ErroDefinirColunas {
    /// Categoria pública: o próximo passo de quem chama.
    pub(crate) fn codigo(&self) -> &'static str {
        match self {
            Self::Leitura(_) | Self::ColunaIndisponivelNestaInstalacao { .. } => "fonte_invalida",
            Self::ConfiguracaoDeColunasNaoAplicada { .. } => "escrita_nao_aplicada",
            Self::MarcasDiferentesDoAlvoAposSalvar { .. } => "escrita_diferente_da_solicitada",
            Self::NaoConfirmadoNaConfiguracaoDeColunas { .. } => "nao_confirmado",
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Leitura(erro) => erro.mensagem(),
            Self::ColunaIndisponivelNestaInstalacao { .. } => {
                "a coluna pedida não é publicada pelo Nível de detalhe desta instalação; nada foi salvo"
            }
            Self::ConfiguracaoDeColunasNaoAplicada { .. } => {
                "a configuração lida depois de salvar é a de antes: as colunas não foram alteradas"
            }
            Self::MarcasDiferentesDoAlvoAposSalvar { .. } => {
                "a configuração lida depois de salvar não é a pedida nem a de antes"
            }
            Self::NaoConfirmadoNaConfiguracaoDeColunas { .. } => {
                "não se sabe se as colunas foram alteradas; confira com `ies unidade colunas` antes de repetir"
            }
        }
    }
}
