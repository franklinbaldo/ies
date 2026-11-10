//! Resultado e recusas de `ies unidade colunas --definir`
//! (contracts/cli/unidade-definir-colunas.md). Um erro por fato.
use super::ErroVisaoDoControle;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::nivel_de_detalhe::{ColunaDoDetalhe, PlanoDasColunas};

/// Por que a resposta do POST de salvar não é a esperada (HTML 200 sem
/// tela de login, com o sinal de gravação). O pedido saiu: o efeito é
/// incerto até a leitura.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaDoSalvar {
    /// Rede, tempo esgotado, 5xx, outro status, MIME, charset, tamanho, login.
    Envio(#[from] ErroSubmissaoFormulario),
    /// 302 recebido e não seguido.
    Redirecionamento302,
    /// HTML 200 com a tela do Nível sem o sinal de gravação (`''=='1'`).
    NivelDevolvidoSemSinalDeGravacao,
    /// HTML 200 sem o `inicializar()` observado da tela do Nível.
    RespostaDoSalvarNaoReconhecida,
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

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroDefinirColunas {
    /// Falha antes do POST de salvar: nada foi salvo.
    Leitura(#[from] ErroVisaoDoControle),
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
